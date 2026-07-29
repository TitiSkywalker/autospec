//! Flatten nested for-loops in Rust source by extracting inner loops into
//! separate functions.  Aeneas cannot translate functions that contain a for-loop
//! inside another for-loop, so this tool guarantees every function has at most
//! one level of for-loop nesting.

use anyhow::{Context, Result};
use quote::format_ident;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

// ── Types ────────────────────────────────────────────────────────────────────

struct CapturedVar {
    name: String,
    ty: syn::Type,
}

// ── Entry point ──────────────────────────────────────────────────────────────

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: flatten-loops <file.rs> [--in-place]");
        std::process::exit(1);
    }

    let path = PathBuf::from(&args[1]);
    let in_place = args.get(2).map_or(false, |a| a == "--in-place");

    let src = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {}", path.display()))?;

    let transformed = transform_file(&src)?;

    if in_place {
        fs::write(&path, &transformed)?;
        eprintln!("Wrote {}", path.display());
    } else {
        println!("{}", transformed);
    }
    Ok(())
}

fn transform_file(src: &str) -> Result<String> {
    let mut file: syn::File =
        syn::parse_str(src).with_context(|| "Failed to parse Rust source")?;

    let mut new_items: Vec<syn::Item> = Vec::new();

    for item in &mut file.items {
        if let syn::Item::Fn(func) = item {
            // Extract nested loops from this function iteratively
            let mut iterations = 0;
            const MAX_ITERATIONS: usize = 50;
            loop {
                let mut lifted = Vec::new();
                let extracted = extract_one_nested_loop(func, &mut lifted);
                if extracted.is_none() {
                    break;
                }
                // Lifted inner fns first, then the extracted function
                for lf in lifted {
                    new_items.push(syn::Item::Fn(lf));
                }
                new_items.push(syn::Item::Fn(extracted.unwrap()));
                iterations += 1;
                if iterations >= MAX_ITERATIONS {
                    eprintln!(
                        "flatten-loops: hit iteration limit ({MAX_ITERATIONS}) in fn '{}', stopping",
                        func.sig.ident,
                    );
                    break;
                }
            }
        }
    }

    // Append extracted functions after the last item
    file.items.extend(new_items);

    Ok(pretty_print(&file))
}

// ── Core extraction logic ────────────────────────────────────────────────────

/// Find the innermost nested for-loop in `func` (a for-loop whose body contains
/// another for-loop) and extract it into a new function.  The original function
/// is mutated in-place to replace the loop with a call to the new function.
fn extract_one_nested_loop(
    func: &mut syn::ItemFn,
    lifted: &mut Vec<syn::ItemFn>,
) -> Option<syn::ItemFn> {
    // Collect type information from function params
    let mut known_types = collect_param_types(func);

    // Scan the function body for let bindings (for type info)
    scan_let_bindings_block(&func.block, &mut known_types);

    // Find innermost nested loop
    let (path, for_loop) = find_innermost_nested_loop(&func.block)?;

    // Determine free variables (used in loop body + range expression, not defined there, excluding loop variable)
    let loop_var_names = pat_binding_names(&for_loop.pat);
    let mut captured_names = free_variables(&for_loop.body);
    // Also collect variables from the range expression (e.g., i, nrows in `for k in i+1..nrows`)
    collect_used_in_expr(&for_loop.expr, &mut captured_names);
    for lv in &loop_var_names {
        captured_names.remove(lv);
    }

    // Remove function call targets — these are module-level functions, not
    // variables to be captured (e.g., `get_element(data, n, i, j)`).
    let call_targets = collect_call_targets_in_block(&for_loop.body);

    // If the loop body calls inner functions (defined inside the original
    // function), they need to be lifted to the module level so the extracted
    // function can find them.
    if !call_targets.is_empty() {
        *lifted = lift_inner_functions(func, &call_targets);
    }

    for ct in &call_targets {
        captured_names.remove(ct);
    }

    // Build parameter list for the extracted function.
    // If any captured variable has an unknown type, we cannot safely
    // extract the loop — bail out to avoid generating broken code.
    let mut params: Vec<CapturedVar> = Vec::new();
    for name in &captured_names {
        match known_types.get(name) {
            Some(ty) => params.push(CapturedVar {
                name: name.clone(),
                ty: ty.clone(),
            }),
            None => {
                eprintln!(
                    "flatten-loops: skipping extraction — unknown type for '{}'",
                    name,
                );
                return None;
            }
        }
    }

    // Correct heuristically-guessed types based on actual usage in the loop body.
    // Variables used with indexing (e.g., `var[i]`) cannot be scalar f64/usize.
    let index_targets = collect_index_targets_in_block(&for_loop.body);
    for p in &mut params {
        let ty: &syn::Type = &p.ty;
        let ty_str = quote::quote! { #ty }.to_string();
        if index_targets.contains(&p.name) {
            if ty_str == "f64" {
                p.ty = syn::parse_quote! { &[f64] };
            } else if ty_str == "usize" {
                p.ty = syn::parse_quote! { &[usize] };
            }
        }
        // Variables that have elements assigned (var[i] = ...) need &mut
        if has_index_assign_in_block(&p.name, &for_loop.body) {
            let current_ty: &syn::Type = &p.ty;
            let new_ty_str = quote::quote! { #current_ty }.to_string();
            if new_ty_str == "& [f64]" {
                p.ty = syn::parse_quote! { &mut [f64] };
            }
        }
    }

    // Detect which Copy-type params are mutated in the loop body.
    // These need to be returned from the extracted function and reassigned
    // at the call site, otherwise mutations are lost.
    let mutated_copy: HashSet<String> = params
        .iter()
        .filter(|p| is_copy_type(&p.ty) && is_mutated_in_block(&p.name, &for_loop.body))
        .map(|p| p.name.clone())
        .collect();

    // Generate unique function name
    let fn_name = func.sig.ident.to_string();
    let inner_name = format_ident!("__{}_inner{}", fn_name, next_counter());

    // Build return type for mutated Copy values
    let ret_ty: syn::Type = if mutated_copy.is_empty() {
        syn::parse_quote! { () }
    } else if mutated_copy.len() == 1 {
        let name = mutated_copy.iter().next().unwrap();
        let ty = known_types.get(name).unwrap();
        ty.clone()
    } else {
        // Sort for deterministic output
        let mut sorted: Vec<_> = mutated_copy.iter().collect();
        sorted.sort();
        let types: Vec<&syn::Type> = sorted
            .iter()
            .map(|n| known_types.get(*n).unwrap())
            .collect();
        syn::parse_quote! { (#(#types),*) }
    };

    // Build parameter declarations — borrow non-Copy types; Copy types are by value
    let param_decls: Vec<syn::FnArg> = params
        .iter()
        .map(|p| {
            let name: syn::Ident = format_ident!("{}", p.name);
            let ty = &p.ty;
            if needs_borrow(ty) {
                syn::parse_quote! { #name: &mut #ty }
            } else if mutated_copy.contains(&p.name) {
                // Mutable Copy type — declare as `mut` so body can modify it
                syn::parse_quote! { mut #name: #ty }
            } else {
                syn::parse_quote! { #name: #ty }
            }
        })
        .collect();

    // If there are mutated Copy values, add them to the end of the body as return
    let ret_expr: Option<syn::Expr> = if mutated_copy.is_empty() {
        None
    } else if mutated_copy.len() == 1 {
        let name = mutated_copy.iter().next().unwrap();
        let id: syn::Ident = format_ident!("{name}");
        Some(syn::parse_quote! { #id })
    } else {
        let mut sorted: Vec<_> = mutated_copy.iter().collect();
        sorted.sort();
        let ids: Vec<syn::Ident> = sorted
            .iter()
            .map(|n| format_ident!("{n}"))
            .collect();
        Some(syn::parse_quote! { (#(#ids),*) })
    };

    let extracted_fn: syn::ItemFn = match ret_expr {
        Some(ret) => syn::parse_quote! {
            fn #inner_name(#(#param_decls),*) -> #ret_ty {
                #for_loop
                #ret
            }
        },
        None => syn::parse_quote! {
            fn #inner_name(#(#param_decls),*) {
                #for_loop
            }
        },
    };

    // Build the call statement
    let call_args: Vec<syn::Expr> = params
        .iter()
        .map(|p| {
            let id: syn::Ident = format_ident!("{}", p.name);
            if needs_borrow(&p.ty) {
                syn::parse_quote! { &mut #id }
            } else {
                let ty: &syn::Type = &p.ty;
                let ty_str = quote::quote! { #ty }.to_string();
                // If the parameter is a reference to a slice (e.g., &[f64]),
                // the call-site variable is likely Vec<f64>, which needs `&var`.
                if ty_str.starts_with("& mut [") || ty_str.starts_with("&mut [") {
                    syn::parse_quote! { &mut #id }
                } else if ty_str.starts_with("& [") {
                    syn::parse_quote! { &#id }
                } else {
                    syn::parse_quote! { #id }
                }
            }
        })
        .collect();

    let call_stmt: syn::Stmt = if mutated_copy.is_empty() {
        syn::parse_quote! {
            #inner_name(#(#call_args),*);
        }
    } else if mutated_copy.len() == 1 {
        let name = mutated_copy.iter().next().unwrap();
        let id: syn::Ident = format_ident!("{name}");
        syn::parse_quote! {
            #id = #inner_name(#(#call_args),*);
        }
    } else {
        let mut sorted: Vec<_> = mutated_copy.iter().collect();
        sorted.sort();
        let ids: Vec<syn::Ident> = sorted
            .iter()
            .map(|n| format_ident!("{n}"))
            .collect();
        syn::parse_quote! {
            (#(#ids),*) = #inner_name(#(#call_args),*);
        }
    };

    // Replace the loop with the call in-place
    replace_loop_with_call(&mut func.block, &path, call_stmt);

    Some(extracted_fn)
}

// ── Type collection ─────────────────────────────────────────────────────────

fn collect_param_types(func: &syn::ItemFn) -> HashMap<String, syn::Type> {
    let mut types = HashMap::new();
    for input in &func.sig.inputs {
        if let syn::FnArg::Typed(pat_type) = input {
            collect_pat_types(&pat_type.pat, (*pat_type.ty).clone(), &mut types);
        }
    }
    types
}

fn collect_pat_types(pat: &syn::Pat, ty: syn::Type, out: &mut HashMap<String, syn::Type>) {
    match pat {
        syn::Pat::Ident(pi) => {
            out.insert(pi.ident.to_string(), ty);
        }
        syn::Pat::Type(pt) => {
            collect_pat_types(&pt.pat, (*pt.ty).clone(), out);
        }
        syn::Pat::Reference(pr) => {
            collect_pat_types(&pr.pat, ty, out);
        }
        syn::Pat::Tuple(pt) => {
            for elem in &pt.elems {
                collect_pat_types(elem, ty.clone(), out);
            }
        }
        _ => {}
    }
}

fn scan_let_bindings_block(block: &syn::Block, known: &mut HashMap<String, syn::Type>) {
    for stmt in &block.stmts {
        scan_let_bindings_stmt(stmt, known);
    }
}

fn scan_let_bindings_stmt(stmt: &syn::Stmt, known: &mut HashMap<String, syn::Type>) {
    match stmt {
        syn::Stmt::Local(local) => {
            // Get type from annotation or inference
            let ty: Option<syn::Type> = if let syn::Pat::Type(pt) = &local.pat {
                Some((*pt.ty).clone())
            } else if let Some(init) = &local.init {
                infer_type(&init.expr, known)
            } else {
                None
            };

            // Register binding
            if let Some(ty) = ty {
                collect_pat_types(&local.pat, ty, known);
            }
        }
        syn::Stmt::Expr(expr, _) => {
            scan_let_bindings_expr(expr, known);
        }
        _ => {}
    }
}

fn scan_let_bindings_expr(expr: &syn::Expr, known: &mut HashMap<String, syn::Type>) {
    match expr {
        syn::Expr::ForLoop(fl) => {
            // Infer the loop variable type from the range expression
            let loop_ty = infer_type(&fl.expr, known).or_else(|| {
                // Ranges are typically over usize
                Some(syn::parse_quote! { usize })
            });
            if let Some(ty) = loop_ty {
                add_pat_bindings_with_type(&fl.pat, ty, known);
            }
            scan_let_bindings_block(&fl.body, known);
        }
        syn::Expr::Block(block) => scan_let_bindings_block(&block.block, known),
        syn::Expr::If(if_expr) => {
            scan_let_bindings_block(&if_expr.then_branch, known);
            if let Some((_, else_expr)) = &if_expr.else_branch {
                scan_let_bindings_expr(else_expr, known);
            }
        }
        _ => {}
    }
}

fn add_pat_bindings_with_type(pat: &syn::Pat, ty: syn::Type, known: &mut HashMap<String, syn::Type>) {
    match pat {
        syn::Pat::Ident(pi) => {
            known.insert(pi.ident.to_string(), ty);
        }
        syn::Pat::Type(pt) => add_pat_bindings_with_type(&pt.pat, (*pt.ty).clone(), known),
        syn::Pat::Reference(pr) => add_pat_bindings_with_type(&pr.pat, ty, known),
        syn::Pat::Tuple(pt) => {
            for elem in &pt.elems {
                add_pat_bindings_with_type(elem, ty.clone(), known);
            }
        }
        _ => {}
    }
}

fn infer_type(expr: &syn::Expr, known: &HashMap<String, syn::Type>) -> Option<syn::Type> {
    match expr {
        syn::Expr::Lit(lit) => match &lit.lit {
            syn::Lit::Int(_) => Some(syn::parse_quote! { usize }),
            syn::Lit::Float(_) => Some(syn::parse_quote! { f64 }),
            syn::Lit::Bool(_) => Some(syn::parse_quote! { bool }),
            _ => None,
        },
        syn::Expr::Binary(bin) => {
            infer_type(&bin.left, known).or_else(|| infer_type(&bin.right, known))
        }
        syn::Expr::Unary(un) => {
            // -f64 → f64, !bool → bool, *ref → inner type
            infer_type(&un.expr, known)
        }
        syn::Expr::Call(call) => {
            if let syn::Expr::Path(path) = &*call.func {
                let segments: Vec<String> = path
                    .path
                    .segments
                    .iter()
                    .map(|s| s.ident.to_string())
                    .collect();
                if segments.iter().any(|s| s.contains("f64_helpers")) {
                    return Some(syn::parse_quote! { f64 });
                }
                // Try to infer from the function name heuristics
                let fname = segments.last().map(|s| s.as_str()).unwrap_or("");
                if fname.contains("get_element") || fname.contains("index") {
                    return Some(syn::parse_quote! { f64 });
                }
                if fname.contains("len") || fname.contains("dim") {
                    return Some(syn::parse_quote! { usize });
                }
                if fname.contains("clone") || fname.contains("to_") {
                    // Try to infer from first argument
                    if let Some(first_arg) = call.args.first() {
                        return infer_type(first_arg, known);
                    }
                }
            }
            None
        }
        syn::Expr::Path(path) => {
            let name = path.path.get_ident()?.to_string();
            // First check known types
            if let Some(ty) = known.get(&name) {
                return Some(ty.clone());
            }
            // Heuristics for common names
            if name.contains("nrows") || name.contains("ncols") || name.contains("offset")
                || name.contains("dim") || name.contains("len") || name.contains("size")
                || name.contains("iter") || name.contains("index")
                || name.contains("_idx") || name.contains("col_") || name.contains("row_")
                || name.contains("start") || name.contains("_ix")
            {
                Some(syn::parse_quote! { usize })
            } else if name.contains("diag") || name.contains("norm")
                || name.contains("dot") || name.contains("factor")
                || name.contains("tol") || name.contains("eps")
                || name.contains("scalar") || name.contains("alpha") || name.contains("beta")
                || (name.len() <= 2 && !name.contains("a") && !name.contains("b"))
            {
                Some(syn::parse_quote! { f64 })
            } else {
                None
            }
        }
        syn::Expr::Index(idx) => {
            // If we can infer the base type (e.g., &[f64]), return the element type f64
            match infer_type(&idx.expr, known) {
                Some(ty) => {
                    let ty_str = quote::quote! { #ty }.to_string();
                    if ty_str.contains("f64") {
                        Some(syn::parse_quote! { f64 })
                    } else {
                        // Could be &[usize] etc — return usize for simplicity
                        Some(syn::parse_quote! { usize })
                    }
                }
                None => None,
            }
        }
        syn::Expr::Macro(mac) => {
            // vec![f64_expr; N] or vec![f64_expr, ...] → Vec<f64>
            if mac.mac.path.is_ident("vec") {
                Some(syn::parse_quote! { Vec<f64> })
            } else {
                None
            }
        }
        syn::Expr::Reference(r) => {
            // If it's a &[f64], the result type is &[f64] (for passing to functions)
            infer_type(&r.expr, known)
        }
        syn::Expr::MethodCall(mc) => {
            let method = mc.method.to_string();
            // Methods that return ()
            if method == "swap" {
                return Some(syn::parse_quote! { () });
            }
            // f64 methods that return f64
            if matches!(
                method.as_str(),
                "sqrt" | "abs" | "signum" | "ceil" | "floor" | "round" | "trunc"
                    | "fract" | "recip" | "exp" | "ln" | "log2" | "log10"
                    | "sin" | "cos" | "tan" | "asin" | "acos" | "atan"
                    | "sinh" | "cosh" | "tanh" | "cbrt" | "hypot"
                    | "powf" | "powi" | "to_degrees" | "to_radians"
                    | "max" | "min" | "clamp"
            ) {
                return Some(syn::parse_quote! { f64 });
            }
            // f64 methods that return bool
            if matches!(
                method.as_str(),
                "is_nan" | "is_infinite" | "is_finite"
                    | "is_sign_positive" | "is_sign_negative"
            ) {
                return Some(syn::parse_quote! { bool });
            }
            // usize/vec methods
            if method == "len" || method == "capacity" {
                return Some(syn::parse_quote! { usize });
            }
            // Try to infer from receiver type: f64 methods return f64 by default
            if let Some(receiver_ty) = infer_type(&mc.receiver, known) {
                let ty_str = quote::quote! { #receiver_ty }.to_string();
                if ty_str.contains("f64") || ty_str.contains("f32") {
                    return Some(syn::parse_quote! { f64 });
                }
            }
            None
        }
        syn::Expr::If(if_expr) => {
            // Type is the type of the branches
            // then_branch is a Block, not an Expr — get last expression
            if_expr
                .then_branch
                .stmts
                .last()
                .and_then(|stmt| match stmt {
                    syn::Stmt::Expr(e, _) => infer_type(e, known),
                    _ => None,
                })
                .or_else(|| {
                    if_expr
                        .else_branch
                        .as_ref()
                        .and_then(|(_, e)| infer_type(e, known))
                })
        }
        syn::Expr::Block(block) => {
            // Type is the type of the last expression
            block
                .block
                .stmts
                .last()
                .and_then(|stmt| match stmt {
                    syn::Stmt::Expr(e, _) => infer_type(e, known),
                    _ => None,
                })
        }
        syn::Expr::Paren(p) => infer_type(&p.expr, known),
        syn::Expr::Group(g) => infer_type(&g.expr, known),
        syn::Expr::Cast(cast) => {
            // `x as usize` → usize, `x as f64` → f64, etc.
            let ty_str = quote::quote! { #cast.ty }.to_string();
            match ty_str.as_str() {
                "usize" => Some(syn::parse_quote! { usize }),
                "f64" | "f32" => Some(syn::parse_quote! { f64 }),
                "bool" => Some(syn::parse_quote! { bool }),
                _ => infer_type(&cast.expr, known),
            }
        }
        syn::Expr::Field(field) => {
            // struct.field — try to infer from field name heuristics
            let field_name = field
                .member
                .clone();
            let name = quote::quote! { #field_name }.to_string();
            if name.contains("nrows") || name.contains("ncols") || name.contains("dim") {
                Some(syn::parse_quote! { usize })
            } else if name.contains("diag") || name.contains("val") || name.contains("re")
                || name.contains("im")
            {
                Some(syn::parse_quote! { f64 })
            } else {
                infer_type(&field.base, known)
            }
        }
        _ => None,
    }
}

// ── Free variable detection ─────────────────────────────────────────────────

fn free_variables(block: &syn::Block) -> HashSet<String> {
    let mut used = HashSet::new();
    let mut defined = HashSet::new();
    collect_used_in_block(block, &mut used, &mut defined);
    &used - &defined
}

/// Collect names used as function call targets (not method calls).
/// These are module-level functions that should NOT be captured as parameters.
/// Find inner functions defined inside `func` that are named in `targets`,
/// remove them from the function body, and return them for lifting to the
/// module level.
fn lift_inner_functions(
    func: &mut syn::ItemFn,
    targets: &HashSet<String>,
) -> Vec<syn::ItemFn> {
    let mut lifted = Vec::new();
    let mut new_stmts = Vec::new();

    for stmt in func.block.stmts.drain(..) {
        let mut keep = true;
        if let syn::Stmt::Item(syn::Item::Fn(inner)) = &stmt {
            let name = inner.sig.ident.to_string();
            if targets.contains(&name) {
                lifted.push(inner.clone());
                keep = false;
            }
        }
        if keep {
            new_stmts.push(stmt);
        }
    }
    func.block.stmts = new_stmts;
    lifted
}

/// Collect variable names that are used as indexing targets in the block
/// (e.g., `matrix[i]` means `matrix` is an index target).
fn collect_index_targets_in_block(block: &syn::Block) -> HashSet<String> {
    let mut targets = HashSet::new();
    collect_index_targets_in_block_rec(block, &mut targets);
    targets
}

fn collect_index_targets_in_block_rec(block: &syn::Block, targets: &mut HashSet<String>) {
    for stmt in &block.stmts {
        match stmt {
            syn::Stmt::Expr(expr, _) => collect_index_targets_in_expr(expr, targets),
            syn::Stmt::Local(local) => {
                if let Some(init) = &local.init {
                    collect_index_targets_in_expr(&init.expr, targets);
                }
            }
            _ => {}
        }
    }
}

fn collect_index_targets_in_expr(expr: &syn::Expr, targets: &mut HashSet<String>) {
    match expr {
        syn::Expr::Index(idx) => {
            if let syn::Expr::Path(path) = &*idx.expr {
                if let Some(ident) = path.path.get_ident() {
                    targets.insert(ident.to_string());
                }
            }
            collect_index_targets_in_expr(&idx.expr, targets);
            collect_index_targets_in_expr(&idx.index, targets);
        }
        syn::Expr::ForLoop(fl) => {
            collect_index_targets_in_expr(&fl.expr, targets);
            collect_index_targets_in_block_rec(&fl.body, targets);
        }
        syn::Expr::Block(b) => collect_index_targets_in_block_rec(&b.block, targets),
        syn::Expr::If(if_expr) => {
            collect_index_targets_in_expr(&if_expr.cond, targets);
            collect_index_targets_in_block_rec(&if_expr.then_branch, targets);
            if let Some((_, else_expr)) = &if_expr.else_branch {
                collect_index_targets_in_expr(else_expr, targets);
            }
        }
        syn::Expr::Binary(bin) => {
            collect_index_targets_in_expr(&bin.left, targets);
            collect_index_targets_in_expr(&bin.right, targets);
        }
        syn::Expr::Unary(un) => collect_index_targets_in_expr(&un.expr, targets),
        syn::Expr::Assign(assign) => {
            collect_index_targets_in_expr(&assign.left, targets);
            collect_index_targets_in_expr(&assign.right, targets);
        }
        syn::Expr::Call(call) => {
            for arg in &call.args {
                collect_index_targets_in_expr(arg, targets);
            }
        }
        syn::Expr::MethodCall(mc) => {
            collect_index_targets_in_expr(&mc.receiver, targets);
            for arg in &mc.args {
                collect_index_targets_in_expr(arg, targets);
            }
        }
        syn::Expr::Reference(r) => collect_index_targets_in_expr(&r.expr, targets),
        syn::Expr::Paren(p) => collect_index_targets_in_expr(&p.expr, targets),
        syn::Expr::Group(g) => collect_index_targets_in_expr(&g.expr, targets),
        syn::Expr::Macro(mac) => {
            collect_used_in_macro_tokens(&mac.mac.tokens, targets);
        }
        _ => {}
    }
}

/// Check if a variable is the target of an index assignment in the block
/// (e.g., `matrix[i] = val` or `matrix[i] += val`).
fn has_index_assign_in_block(name: &str, block: &syn::Block) -> bool {
    for stmt in &block.stmts {
        if has_index_assign_in_stmt(name, stmt) {
            return true;
        }
    }
    false
}

fn has_index_assign_in_stmt(name: &str, stmt: &syn::Stmt) -> bool {
    match stmt {
        syn::Stmt::Expr(expr, _) => has_index_assign_in_expr(name, expr),
        _ => false,
    }
}

fn has_index_assign_in_expr(name: &str, expr: &syn::Expr) -> bool {
    match expr {
        syn::Expr::Assign(assign) => {
            if has_index_with_name(&assign.left, name) {
                return true;
            }
            has_index_assign_in_expr(name, &assign.right)
        }
        syn::Expr::Binary(bin) => {
            if is_binop_assign(&bin.op) && has_index_with_name(&bin.left, name) {
                return true;
            }
            has_index_assign_in_expr(name, &bin.left)
                || has_index_assign_in_expr(name, &bin.right)
        }
        syn::Expr::ForLoop(fl) => {
            has_index_assign_in_expr(name, &fl.expr)
                || has_index_assign_in_block(name, &fl.body)
        }
        syn::Expr::Block(b) => has_index_assign_in_block(name, &b.block),
        syn::Expr::If(if_expr) => {
            has_index_assign_in_expr(name, &if_expr.cond)
                || has_index_assign_in_block(name, &if_expr.then_branch)
                || if_expr
                    .else_branch
                    .as_ref()
                    .map_or(false, |(_, e)| has_index_assign_in_expr(name, e))
        }
        syn::Expr::Call(call) => call
            .args
            .iter()
            .any(|a| has_index_assign_in_expr(name, a)),
        syn::Expr::MethodCall(mc) => {
            has_index_assign_in_expr(name, &mc.receiver)
                || mc.args.iter().any(|a| has_index_assign_in_expr(name, a))
        }
        _ => false,
    }
}

fn has_index_with_name(expr: &syn::Expr, name: &str) -> bool {
    if let syn::Expr::Index(idx) = expr {
        if let syn::Expr::Path(path) = &*idx.expr {
            if let Some(ident) = path.path.get_ident() {
                return ident.to_string() == name;
            }
        }
    }
    false
}

fn collect_call_targets_in_block(block: &syn::Block) -> HashSet<String> {
    let mut targets = HashSet::new();
    collect_call_targets_in_block_rec(block, &mut targets);
    targets
}

fn collect_call_targets_in_block_rec(block: &syn::Block, targets: &mut HashSet<String>) {
    for stmt in &block.stmts {
        match stmt {
            syn::Stmt::Expr(expr, _) => collect_call_targets_in_expr(expr, targets),
            syn::Stmt::Local(local) => {
                if let Some(init) = &local.init {
                    collect_call_targets_in_expr(&init.expr, targets);
                }
            }
            _ => {}
        }
    }
}

fn collect_call_targets_in_expr(expr: &syn::Expr, targets: &mut HashSet<String>) {
    match expr {
        syn::Expr::Call(call) => {
            if let syn::Expr::Path(path) = &*call.func {
                if let Some(ident) = path.path.get_ident() {
                    targets.insert(ident.to_string());
                }
            }
            for arg in &call.args {
                collect_call_targets_in_expr(arg, targets);
            }
        }
        syn::Expr::ForLoop(fl) => {
            collect_call_targets_in_expr(&fl.expr, targets);
            collect_call_targets_in_block_rec(&fl.body, targets);
        }
        syn::Expr::Block(b) => collect_call_targets_in_block_rec(&b.block, targets),
        syn::Expr::If(if_expr) => {
            collect_call_targets_in_expr(&if_expr.cond, targets);
            collect_call_targets_in_block_rec(&if_expr.then_branch, targets);
            if let Some((_, else_expr)) = &if_expr.else_branch {
                collect_call_targets_in_expr(else_expr, targets);
            }
        }
        syn::Expr::Binary(bin) => {
            collect_call_targets_in_expr(&bin.left, targets);
            collect_call_targets_in_expr(&bin.right, targets);
        }
        syn::Expr::Unary(un) => collect_call_targets_in_expr(&un.expr, targets),
        syn::Expr::Assign(assign) => {
            collect_call_targets_in_expr(&assign.left, targets);
            collect_call_targets_in_expr(&assign.right, targets);
        }
        syn::Expr::MethodCall(mc) => {
            collect_call_targets_in_expr(&mc.receiver, targets);
            for arg in &mc.args {
                collect_call_targets_in_expr(arg, targets);
            }
        }
        syn::Expr::Index(idx) => {
            collect_call_targets_in_expr(&idx.expr, targets);
            collect_call_targets_in_expr(&idx.index, targets);
        }
        syn::Expr::Reference(r) => collect_call_targets_in_expr(&r.expr, targets),
        syn::Expr::Paren(p) => collect_call_targets_in_expr(&p.expr, targets),
        syn::Expr::Group(g) => collect_call_targets_in_expr(&g.expr, targets),
        syn::Expr::Array(arr) => {
            for elem in &arr.elems {
                collect_call_targets_in_expr(elem, targets);
            }
        }
        syn::Expr::Tuple(tup) => {
            for elem in &tup.elems {
                collect_call_targets_in_expr(elem, targets);
            }
        }
        syn::Expr::Return(ret) => {
            if let Some(e) = &ret.expr {
                collect_call_targets_in_expr(e, targets);
            }
        }
        syn::Expr::Macro(mac) => {
            collect_call_targets_in_macro_tokens(&mac.mac.tokens, targets);
        }
        _ => {}
    }
}

fn collect_call_targets_in_macro_tokens(
    tokens: &proc_macro2::TokenStream,
    targets: &mut HashSet<String>,
) {
    use proc_macro2::TokenTree;
    for tt in tokens.clone() {
        match tt {
            TokenTree::Group(g) => {
                collect_call_targets_in_macro_tokens(&g.stream(), targets);
            }
            _ => {}
        }
    }
}

fn collect_used_in_block(
    block: &syn::Block,
    used: &mut HashSet<String>,
    defined: &mut HashSet<String>,
) {
    for stmt in &block.stmts {
        match stmt {
            syn::Stmt::Local(local) => {
                if let Some(init) = &local.init {
                    collect_used_in_expr(&init.expr, used);
                }
                add_pat_bindings(&local.pat, defined);
            }
            syn::Stmt::Expr(expr, _) => {
                collect_used_in_expr(expr, used);
            }
            syn::Stmt::Macro(mac) => {
                // Macros may contain variable uses in their arguments
                collect_used_in_macro_tokens(&mac.mac.tokens, used);
            }
            _ => {}
        }
    }
}

fn pat_binding_names(pat: &syn::Pat) -> HashSet<String> {
    let mut names = HashSet::new();
    add_pat_bindings(pat, &mut names);
    names
}

fn add_pat_bindings(pat: &syn::Pat, defined: &mut HashSet<String>) {
    match pat {
        syn::Pat::Ident(pi) => {
            defined.insert(pi.ident.to_string());
        }
        syn::Pat::Type(pt) => add_pat_bindings(&pt.pat, defined),
        syn::Pat::Reference(pr) => add_pat_bindings(&pr.pat, defined),
        syn::Pat::Tuple(pt) => {
            for elem in &pt.elems {
                add_pat_bindings(elem, defined);
            }
        }
        _ => {}
    }
}

fn collect_used_in_macro_tokens(tokens: &proc_macro2::TokenStream, used: &mut HashSet<String>) {
    use proc_macro2::TokenTree;
    for tt in tokens.clone() {
        match tt {
            TokenTree::Group(g) => {
                // Try to parse the group contents as an expression
                if let Ok(expr) = syn::parse2::<syn::Expr>(g.stream()) {
                    collect_used_in_expr(&expr, used);
                } else {
                    // Recurse into the group
                    collect_used_in_macro_tokens(&g.stream(), used);
                }
            }
            TokenTree::Ident(ident) => {
                let name = ident.to_string();
                // Skip rust keywords
                if !matches!(
                    name.as_str(),
                    "if" | "else" | "for" | "while" | "let" | "mut"
                        | "fn" | "return" | "true" | "false" | "self"
                        | "use" | "mod" | "pub" | "struct" | "enum" | "match"
                ) {
                    used.insert(name);
                }
            }
            _ => {}
        }
    }
}

fn collect_used_in_expr(expr: &syn::Expr, used: &mut HashSet<String>) {
    match expr {
        syn::Expr::Path(path) => {
            if let Some(ident) = path.path.get_ident() {
                // Only simple identifiers (not function calls with multiple segments)
                if path.qself.is_none() && path.path.segments.len() == 1 {
                    used.insert(ident.to_string());
                }
            }
        }
        syn::Expr::ForLoop(fl) => {
            collect_used_in_expr(&fl.expr, used);
            let mut local_defs = HashSet::new();
            add_pat_bindings(&fl.pat, &mut local_defs);
            let mut body_used = HashSet::new();
            let mut body_defs = local_defs.clone();
            collect_used_in_block(&fl.body, &mut body_used, &mut body_defs);
            used.extend(&body_used - &local_defs);
        }
        syn::Expr::Block(block) => {
            let mut block_defs = HashSet::new();
            collect_used_in_block(&block.block, used, &mut block_defs);
        }
        syn::Expr::Binary(bin) => {
            collect_used_in_expr(&bin.left, used);
            collect_used_in_expr(&bin.right, used);
        }
        syn::Expr::Unary(un) => collect_used_in_expr(&un.expr, used),
        syn::Expr::Call(call) => {
            collect_used_in_expr(&call.func, used);
            for arg in &call.args {
                collect_used_in_expr(arg, used);
            }
        }
        syn::Expr::MethodCall(mc) => {
            collect_used_in_expr(&mc.receiver, used);
            for arg in &mc.args {
                collect_used_in_expr(arg, used);
            }
        }
        syn::Expr::Assign(assign) => {
            collect_used_in_expr(&assign.left, used);
            collect_used_in_expr(&assign.right, used);
        }
        // Compound assignments (e.g., a += b) are desugared by syn
        syn::Expr::Index(idx) => {
            collect_used_in_expr(&idx.expr, used);
            collect_used_in_expr(&idx.index, used);
        }
        syn::Expr::If(if_expr) => {
            collect_used_in_expr(&if_expr.cond, used);
            let mut then_defs = HashSet::new();
            collect_used_in_block(&if_expr.then_branch, used, &mut then_defs);
            if let Some((_, else_expr)) = &if_expr.else_branch {
                collect_used_in_expr(else_expr, used);
            }
        }
        syn::Expr::Reference(r) => collect_used_in_expr(&r.expr, used),
        syn::Expr::Field(f) => collect_used_in_expr(&f.base, used),
        syn::Expr::Range(r) => {
            if let Some(from) = &r.start {
                collect_used_in_expr(from, used);
            }
            if let Some(to) = &r.end {
                collect_used_in_expr(to, used);
            }
        }
        syn::Expr::Array(arr) => {
            for elem in &arr.elems {
                collect_used_in_expr(elem, used);
            }
        }
        syn::Expr::Tuple(tup) => {
            for elem in &tup.elems {
                collect_used_in_expr(elem, used);
            }
        }
        syn::Expr::Return(ret) => {
            if let Some(e) = &ret.expr {
                collect_used_in_expr(e, used);
            }
        }
        syn::Expr::Break(brk) => {
            if let Some(e) = &brk.expr {
                collect_used_in_expr(e, used);
            }
        }
        syn::Expr::Paren(p) => collect_used_in_expr(&p.expr, used),
        syn::Expr::Group(g) => collect_used_in_expr(&g.expr, used),
        syn::Expr::Macro(mac) => {
            // Try to parse each token tree in the macro body as an expression
            // (skipping string literals, punctuation, etc.)
            collect_used_in_macro_tokens(&mac.mac.tokens, used);
        }
        syn::Expr::Lit(_) | syn::Expr::Continue(_) => {}
        _ => {}
    }
}

// ── Nested loop detection ────────────────────────────────────────────────────

type AstPath = Vec<usize>;

/// Find the innermost nested for-loop in the block.
///
/// A loop is "nested" if it appears at depth >= 2 inside the function
/// (i.e., it's a for-loop inside another for-loop).  We collect all for-loops
/// with their nesting depths and return the deepest one.
fn find_innermost_nested_loop(block: &syn::Block) -> Option<(AstPath, syn::ExprForLoop)> {
    let mut loops: Vec<(AstPath, usize, syn::ExprForLoop)> = Vec::new();
    collect_loops_recursive(block, 0, &mut Vec::new(), &mut loops);

    // Return the deepest loop at depth >= 1 (inside at least one other for-loop)
    let deepest = loops
        .iter()
        .filter(|(_, depth, _)| *depth >= 1)
        .max_by_key(|(_, depth, _)| *depth);

    if let Some((_, depth, _)) = deepest {
        let count_at_depth = loops
            .iter()
            .filter(|(_, d, _)| *d == *depth)
            .count();
        eprintln!(
            "flatten-loops: found {} loops at max depth {depth}",
            count_at_depth,
        );
    }

    loops
        .into_iter()
        .filter(|(_, depth, _)| *depth >= 1)
        .max_by_key(|(_, depth, _)| *depth)
        .map(|(path, _, fl)| (path, fl))
}

fn collect_loops_recursive(
    block: &syn::Block,
    depth: usize,
    prefix: &mut Vec<usize>,
    out: &mut Vec<(AstPath, usize, syn::ExprForLoop)>,
) {
    for (i, stmt) in block.stmts.iter().enumerate() {
        prefix.push(i);
        match stmt {
            syn::Stmt::Expr(expr, _) => {
                collect_loops_in_expr(expr, depth, prefix, out);
            }
            syn::Stmt::Local(local) => {
                if let Some(init) = &local.init {
                    collect_loops_in_expr(&init.expr, depth, prefix, out);
                }
            }
            _ => {}
        }
        prefix.pop();
    }
}

fn collect_loops_in_expr(
    expr: &syn::Expr,
    depth: usize,
    prefix: &mut Vec<usize>,
    out: &mut Vec<(AstPath, usize, syn::ExprForLoop)>,
) {
    match expr {
        syn::Expr::ForLoop(fl) => {
            // Record this loop at its depth
            out.push((prefix.clone(), depth, fl.clone()));

            // Recurse into the body at depth+1
            prefix.push(0);
            collect_loops_recursive(&fl.body, depth + 1, prefix, out);
            prefix.pop();
        }
        syn::Expr::Block(block) => {
            collect_loops_recursive(&block.block, depth, prefix, out);
        }
        syn::Expr::If(if_expr) => {
            collect_loops_recursive(&if_expr.then_branch, depth, prefix, out);
            if let Some((_, else_expr)) = &if_expr.else_branch {
                collect_loops_in_expr(else_expr, depth, prefix, out);
            }
        }
        syn::Expr::Unsafe(unsafe_expr) => {
            collect_loops_recursive(&unsafe_expr.block, depth, prefix, out);
        }
        _ => {}
    }
}

// ── AST replacement ─────────────────────────────────────────────────────────

/// Navigate to `path` within `block` and replace the for-loop with `call_stmt`.
fn replace_loop_with_call(block: &mut syn::Block, path: &[usize], call_stmt: syn::Stmt) {
    if path.is_empty() {
        return;
    }

    let idx = path[0];
    if idx >= block.stmts.len() {
        return;
    }

    if path.len() == 1 {
        // Direct child — replace
        block.stmts[idx] = call_stmt;
        return;
    }

    // Navigate into the statement
    match &mut block.stmts[idx] {
        syn::Stmt::Expr(expr, _) => {
            navigate_expr(expr, &path[1..], call_stmt);
        }
        syn::Stmt::Local(local) => {
            if let Some(init) = &mut local.init {
                navigate_expr(&mut init.expr, &path[1..], call_stmt);
            }
        }
        _ => {}
    }
}

fn navigate_expr(expr: &mut syn::Expr, path: &[usize], call_stmt: syn::Stmt) {
    if path.is_empty() {
        return;
    }

    match expr {
        syn::Expr::ForLoop(fl) => {
            if path[0] == 0 {
                // First path element is the dummy "0" we pushed for "inside for-loop body"
                if path.len() == 2 {
                    // Direct replacement of a statement inside the for-loop body
                    if path[1] < fl.body.stmts.len() {
                        fl.body.stmts[path[1]] = call_stmt;
                    }
                } else if path.len() > 2 {
                    // Navigate deeper
                    let idx = path[1];
                    if idx < fl.body.stmts.len() {
                        match &mut fl.body.stmts[idx] {
                            syn::Stmt::Expr(e, _) => {
                                navigate_expr(e, &path[2..], call_stmt);
                            }
                            syn::Stmt::Local(local) => {
                                if let Some(init) = &mut local.init {
                                    navigate_expr(&mut init.expr, &path[2..], call_stmt);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        syn::Expr::Block(block) => {
            replace_loop_with_call(&mut block.block, path, call_stmt);
        }
        syn::Expr::If(if_expr) => {
            replace_loop_with_call(&mut if_expr.then_branch, path, call_stmt);
        }
        syn::Expr::Unsafe(unsafe_expr) => {
            replace_loop_with_call(&mut unsafe_expr.block, path, call_stmt);
        }
        _ => {}
    }
}

// ── Counter for unique function names ────────────────────────────────────────

/// Determine whether a type needs to be passed by reference (non-Copy types
/// like Vec, String, Box, etc.).  Primitives and references are fine to pass
/// by value.
fn needs_borrow(ty: &syn::Type) -> bool {
    let s = quote::quote! { #ty }.to_string();
    // Already a reference — pass as-is
    if s.contains('&') {
        return false;
    }
    // Copy types
    if s == "usize" || s == "isize" || s == "f64" || s == "f32" || s == "bool"
        || s == "u8" || s == "u16" || s == "u32" || s == "u64"
        || s == "i8" || s == "i16" || s == "i32" || s == "i64"
        || s == "()"
    {
        return false;
    }
    // Everything else (Vec, String, Box, custom structs, etc.) — borrow
    true
}

/// Determine whether a type is a Copy type (passed by value).
fn is_copy_type(ty: &syn::Type) -> bool {
    !needs_borrow(ty)
}

/// Check if a variable named `name` is mutated anywhere inside `block`.
/// This looks for assignments and compound assignments that target the variable.
fn is_mutated_in_block(name: &str, block: &syn::Block) -> bool {
    for stmt in &block.stmts {
        if is_mutated_in_stmt(name, stmt) {
            return true;
        }
    }
    false
}

fn is_mutated_in_stmt(name: &str, stmt: &syn::Stmt) -> bool {
    match stmt {
        syn::Stmt::Expr(expr, _) => is_mutated_in_expr(name, expr),
        syn::Stmt::Local(local) => {
            // Check the initializer for compound assignments (unlikely but possible)
            if let Some(init) = &local.init {
                if is_mutated_in_expr(name, &init.expr) {
                    return true;
                }
            }
            false
        }
        _ => false,
    }
}

fn is_mutated_in_expr(name: &str, expr: &syn::Expr) -> bool {
    match expr {
        // `a += b` is syn::Expr::Binary with BinOp::AddAssign etc.
        syn::Expr::Binary(bin) => {
            if is_binop_assign(&bin.op) && path_matches_name(&bin.left, name) {
                return true;
            }
            is_mutated_in_expr(name, &bin.left)
                || is_mutated_in_expr(name, &bin.right)
        }
        syn::Expr::Assign(assign) => {
            if path_matches_name(&assign.left, name) {
                return true;
            }
            is_mutated_in_expr(name, &assign.right)
        }
        syn::Expr::ForLoop(fl) => {
            if is_mutated_in_expr(name, &fl.expr) {
                return true;
            }
            is_mutated_in_block(name, &fl.body)
        }
        syn::Expr::Block(block) => is_mutated_in_block(name, &block.block),
        syn::Expr::If(if_expr) => {
            if is_mutated_in_expr(name, &if_expr.cond) {
                return true;
            }
            is_mutated_in_block(name, &if_expr.then_branch)
                || if_expr
                    .else_branch
                    .as_ref()
                    .map_or(false, |(_, e)| is_mutated_in_expr(name, e))
        }
        syn::Expr::Unary(un) => is_mutated_in_expr(name, &un.expr),
        syn::Expr::Call(call) => {
            for arg in &call.args {
                if is_mutated_in_expr(name, arg) {
                    return true;
                }
            }
            false
        }
        syn::Expr::MethodCall(mc) => {
            if is_mutated_in_expr(name, &mc.receiver) {
                return true;
            }
            for arg in &mc.args {
                if is_mutated_in_expr(name, arg) {
                    return true;
                }
            }
            false
        }
        syn::Expr::Index(idx) => {
            is_mutated_in_expr(name, &idx.expr)
                || is_mutated_in_expr(name, &idx.index)
        }
        syn::Expr::Reference(r) => is_mutated_in_expr(name, &r.expr),
        syn::Expr::Paren(p) => is_mutated_in_expr(name, &p.expr),
        syn::Expr::Group(g) => is_mutated_in_expr(name, &g.expr),
        syn::Expr::Array(arr) => arr.elems.iter().any(|e| is_mutated_in_expr(name, e)),
        syn::Expr::Tuple(tup) => tup.elems.iter().any(|e| is_mutated_in_expr(name, e)),
        syn::Expr::Return(ret) => {
            ret.expr
                .as_ref()
                .map_or(false, |e| is_mutated_in_expr(name, e))
        }
        _ => false,
    }
}

/// Check if an expression is a simple path matching `name`.
fn path_matches_name(expr: &syn::Expr, name: &str) -> bool {
    if let syn::Expr::Path(path) = expr {
        if let Some(ident) = path.path.get_ident() {
            return ident.to_string() == name;
        }
    }
    false
}

fn is_binop_assign(op: &syn::BinOp) -> bool {
    matches!(
        op,
        syn::BinOp::AddAssign(_)
            | syn::BinOp::SubAssign(_)
            | syn::BinOp::MulAssign(_)
            | syn::BinOp::DivAssign(_)
            | syn::BinOp::RemAssign(_)
            | syn::BinOp::BitAndAssign(_)
            | syn::BinOp::BitOrAssign(_)
            | syn::BinOp::BitXorAssign(_)
            | syn::BinOp::ShlAssign(_)
            | syn::BinOp::ShrAssign(_)
    )
}

fn next_counter() -> usize {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static COUNTER: AtomicUsize = AtomicUsize::new(1);
    COUNTER.fetch_add(1, Ordering::SeqCst)
}

// ── Pretty printing ─────────────────────────────────────────────────────────

fn pretty_print(file: &syn::File) -> String {
    use std::io::Write;
    use std::process::Command;

    let code = file
        .items
        .iter()
        .map(|item| quote::quote! { #item }.to_string())
        .collect::<Vec<_>>()
        .join("\n\n");

    // Try rustfmt for nice formatting
    if let Ok(mut child) = Command::new("rustfmt")
        .arg("--edition")
        .arg("2021")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        if let Some(stdin) = child.stdin.as_mut() {
            let _ = stdin.write_all(code.as_bytes());
        }
        if let Ok(output) = child.wait_with_output() {
            if output.status.success() {
                return String::from_utf8_lossy(&output.stdout).to_string();
            }
        }
    }
    code
}

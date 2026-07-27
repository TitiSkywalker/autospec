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

    let mut new_fns: Vec<syn::Item> = Vec::new();

    for item in &mut file.items {
        if let syn::Item::Fn(func) = item {
            // Extract nested loops from this function iteratively
            while let Some(extracted) = extract_one_nested_loop(func) {
                new_fns.push(syn::Item::Fn(extracted));
            }
        }
    }

    // Append extracted functions after the last item
    file.items.extend(new_fns);

    Ok(pretty_print(&file))
}

// ── Core extraction logic ────────────────────────────────────────────────────

/// Find the innermost nested for-loop in `func` (a for-loop whose body contains
/// another for-loop) and extract it into a new function.  The original function
/// is mutated in-place to replace the loop with a call to the new function.
fn extract_one_nested_loop(func: &mut syn::ItemFn) -> Option<syn::ItemFn> {
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

    // Build parameter list for the extracted function
    let params: Vec<CapturedVar> = captured_names
        .iter()
        .filter_map(|name| {
            known_types.get(name).map(|ty| CapturedVar {
                name: name.clone(),
                ty: ty.clone(),
            })
        })
        .collect();

    // Generate unique function name
    let fn_name = func.sig.ident.to_string();
    let inner_name = format_ident!("__{}_inner{}", fn_name, next_counter());

    // Build the extracted function — borrow non-Copy types to avoid move errors
    let param_decls: Vec<syn::FnArg> = params
        .iter()
        .map(|p| {
            let name: syn::Ident = format_ident!("{}", p.name);
            let ty = &p.ty;
            if needs_borrow(ty) {
                syn::parse_quote! { #name: &mut #ty }
            } else {
                syn::parse_quote! { #name: #ty }
            }
        })
        .collect();

    let extracted_fn: syn::ItemFn = syn::parse_quote! {
        fn #inner_name(#(#param_decls),*) {
            #for_loop
        }
    };

    // Build the call statement — mutably borrow owned values
    let call_args: Vec<syn::Expr> = params
        .iter()
        .map(|p| {
            let id: syn::Ident = format_ident!("{}", p.name);
            if needs_borrow(&p.ty) {
                syn::parse_quote! { &mut #id }
            } else {
                syn::parse_quote! { #id }
            }
        })
        .collect();

    let call_stmt: syn::Stmt = syn::parse_quote! {
        #inner_name(#(#call_args),*);
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
            _ => None,
        },
        syn::Expr::Binary(bin) => {
            infer_type(&bin.left, known).or_else(|| infer_type(&bin.right, known))
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
            if name.contains("nrows") || name.contains("ncols") || name.contains("offset") {
                Some(syn::parse_quote! { usize })
            } else if name.contains("diag") || name.contains("val") || name.contains("max") {
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
        syn::Expr::Reference(r) => {
            // If it's a &[f64], the result type is &[f64] (for passing to functions)
            infer_type(&r.expr, known)
        }
        syn::Expr::MethodCall(mc) => {
            // e.g., matrix.swap(i, j) returns ()
            if mc.method.to_string() == "swap" {
                Some(syn::parse_quote! { () })
            } else {
                None
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

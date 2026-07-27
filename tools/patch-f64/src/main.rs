//! Replace f64 operations and float literals in Rust source with calls to
//! `f64_helpers::*` functions, using `syn` for proper AST-level
//! transformation with type inference.
//!
//! Output: transformed source to stdout, float literal list to stderr.

use anyhow::Result;
use quote::format_ident;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

// ── Entry point ──────────────────────────────────────────────────────────────

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: patch-f64 <file.rs> [--in-place]");
        std::process::exit(1);
    }
    let path = PathBuf::from(&args[1]);
    let in_place = args.get(2).map_or(false, |a| a == "--in-place");
    let src = fs::read_to_string(&path)?;

    let (transformed, floats) = transform(&src)?;

    // Float literals to stderr
    for (name, value) in &floats {
        eprintln!("CONST:{value}:{name}");
    }

    if in_place {
        fs::write(&path, transformed)?;
        eprintln!("Wrote {}", path.display());
    } else {
        println!("{}", transformed);
    }
    Ok(())
}

// ── Core transformation ──────────────────────────────────────────────────────

fn transform(src: &str) -> Result<(String, Vec<(String, String)>)> {
    let mut file: syn::File = syn::parse_str(src)?;

    let mut patcher = F64Patcher::default();

    // Pass 1: collect type information
    patcher.collect_types(&file);

    // Pass 2: transform f64 operations
    patcher.transform_file(&mut file);

    // Sort float literals for deterministic output
    let mut floats: Vec<(String, String)> = patcher
        .float_literals
        .into_iter()
        .map(|v| {
            let name = "const_".to_string() + &v.replace('.', "_").replace('-', "neg_");
            (name, v)
        })
        .collect();
    floats.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    Ok((pretty_print(&file), floats))
}

// ── F64Patcher ───────────────────────────────────────────────────────────────

#[derive(Default)]
struct F64Patcher {
    f64_vars: HashSet<String>,
    float_literals: HashSet<String>,
}

impl F64Patcher {
    // ── Pass 1: collect type info ────────────────────────────────────────

    fn collect_types(&mut self, file: &syn::File) {
        for item in &file.items {
            if let syn::Item::Fn(func) = item {
                self.collect_fn_types(func);
            }
        }
    }

    fn collect_fn_types(&mut self, func: &syn::ItemFn) {
        // Function parameters
        for input in &func.sig.inputs {
            if let syn::FnArg::Typed(pt) = input {
                let is_f64 = type_is_f64(&pt.ty);
                collect_pat_names(&pt.pat, is_f64, &mut self.f64_vars);
            }
        }
        // Scan block for let bindings
        self.collect_block_types(&func.block);
    }

    fn collect_block_types(&mut self, block: &syn::Block) {
        for stmt in &block.stmts {
            match stmt {
                syn::Stmt::Local(local) => {
                    let is_f64 = if let syn::Pat::Type(pt) = &local.pat {
                        type_is_f64(&pt.ty)
                    } else if let Some(init) = &local.init {
                        expr_is_f64(&init.expr, &self.f64_vars)
                    } else {
                        false
                    };
                    collect_pat_names(&local.pat, is_f64, &mut self.f64_vars);
                }
                syn::Stmt::Expr(expr, _) => {
                    self.collect_expr_types(expr);
                }
                _ => {}
            }
        }
    }

    fn collect_expr_types(&mut self, expr: &syn::Expr) {
        match expr {
            syn::Expr::ForLoop(fl) => {
                // Loop variable is always usize (never f64 for `for x in range`)
                collect_pat_names(&fl.pat, false, &mut self.f64_vars);
                self.collect_block_types(&fl.body);
            }
            syn::Expr::If(if_expr) => {
                self.collect_block_types(&if_expr.then_branch);
                if let Some((_, else_expr)) = &if_expr.else_branch {
                    if let syn::Expr::Block(block) = &**else_expr {
                        self.collect_block_types(&block.block);
                    }
                }
            }
            syn::Expr::Block(block) => self.collect_block_types(&block.block),
            _ => {}
        }
    }

    // ── Pass 2: transform ────────────────────────────────────────────────

    fn transform_file(&mut self, file: &mut syn::File) {
        for item in &mut file.items {
            if let syn::Item::Fn(func) = item {
                // Reset f64_vars for this function
                self.f64_vars.clear();
                for input in &func.sig.inputs {
                    if let syn::FnArg::Typed(pt) = input {
                        let is_f64 = type_is_f64(&pt.ty);
                        collect_pat_names(&pt.pat, is_f64, &mut self.f64_vars);
                    }
                }
                self.transform_block(&mut func.block);
            }
        }
    }

    fn transform_block(&mut self, block: &mut syn::Block) {
        for stmt in &mut block.stmts {
            match stmt {
                syn::Stmt::Local(local) => {
                    // Transform the init expression first
                    if let Some(init) = &mut local.init {
                        self.transform_expr(&mut init.expr);
                    }
                    // Update type info for the binding
                    let is_f64 = if let syn::Pat::Type(pt) = &local.pat {
                        type_is_f64(&pt.ty)
                    } else if let Some(init) = &local.init {
                        expr_is_f64(&init.expr, &self.f64_vars)
                    } else {
                        false
                    };
                    collect_pat_names(&local.pat, is_f64, &mut self.f64_vars);
                }
                syn::Stmt::Expr(expr, _) => {
                    self.transform_expr(expr);
                    self.update_types_after_transform(expr);
                }
                _ => {}
            }
        }
    }

    fn update_types_after_transform(&mut self, expr: &mut syn::Expr) {
        // After transforming, update f64_vars for new bindings
        if let syn::Expr::Assign(assign) = expr {
            if expr_is_f64(&assign.right, &self.f64_vars) {
                if let syn::Expr::Path(path) = &*assign.left {
                    if let Some(ident) = path.path.get_ident() {
                        self.f64_vars.insert(ident.to_string());
                    }
                }
            }
        }
    }

    fn transform_expr(&mut self, expr: &mut syn::Expr) {
        // Recurse first (bottom-up)
        self.recurse_into_expr(expr);

        // Then check if THIS expression needs transformation
        if let syn::Expr::Lit(lit) = expr {
            if let syn::Lit::Float(val) = &lit.lit {
                let s = val.to_string();
                if s.ends_with("f64") {
                    return; // skip explicit f64 suffix literals
                }
                let clean = s.trim_end_matches("f32").to_string();
                self.float_literals.insert(clean.clone());
                let name = format_ident!(
                    "const_{}",
                    clean.replace('.', "_").replace('-', "neg_")
                );
                *expr = syn::parse_quote! { f64_helpers::#name() };
                return;
            }
        }

        // Binary operations on f64 operands
        if let syn::Expr::Binary(bin) = expr {
            if expr_is_f64(&bin.left, &self.f64_vars)
                || expr_is_f64(&bin.right, &self.f64_vars)
            {
                let name = match bin.op {
                    syn::BinOp::Add(_) => "add",
                    syn::BinOp::Sub(_) => "sub",
                    syn::BinOp::Mul(_) => "mul",
                    syn::BinOp::Div(_) => "div",
                    syn::BinOp::Eq(_) => "eq",
                    syn::BinOp::Ne(_) => "eq", // != → eq + negate
                    syn::BinOp::Gt(_) => "gt",
                    syn::BinOp::Lt(_) => "lt",
                    syn::BinOp::Ge(_) => "ge",
                    syn::BinOp::Le(_) => "le",
                    _ => return,
                };
                let func = format_ident!("{}", name);
                let left = &bin.left;
                let right = &bin.right;
                if name == "eq" && matches!(bin.op, syn::BinOp::Ne(_)) {
                    *expr = syn::parse_quote! { !f64_helpers::eq(#left, #right) };
                } else {
                    *expr = syn::parse_quote! { f64_helpers::#func(#left, #right) };
                }
                return;
            }
        }

        // Unary negation on f64
        if let syn::Expr::Unary(un) = expr {
            if let syn::UnOp::Neg(_) = un.op {
                if expr_is_f64(&un.expr, &self.f64_vars) {
                    let inner = &un.expr;
                    *expr = syn::parse_quote! { f64_helpers::neg(#inner) };
                    return;
                }
            }
        }

        // .abs() on f64
        if let syn::Expr::MethodCall(mc) = expr {
            if mc.method == "abs" && expr_is_f64(&mc.receiver, &self.f64_vars) {
                let recv = &mc.receiver;
                *expr = syn::parse_quote! { f64_helpers::abs(#recv) };
                return;
            }
        }

        // Assign: update f64_vars when assigning f64 value
        if let syn::Expr::Assign(assign) = expr {
            if expr_is_f64(&assign.right, &self.f64_vars) {
                if let syn::Expr::Path(path) = &*assign.left {
                    if let Some(ident) = path.path.get_ident() {
                        self.f64_vars.insert(ident.to_string());
                    }
                }
                // Also for array element assignment like a[i] = f64_val
                if let syn::Expr::Index(idx) = &*assign.left {
                    if expr_is_f64(&idx.expr, &self.f64_vars) {
                        // a[i] is f64, already tracked via array type
                    }
                }
            }
        }
    }

    fn recurse_into_expr(&mut self, expr: &mut syn::Expr) {
        // Must match each variant and destructure to get &mut access to children.
        // Using a helper macro or manual expansion.
        match expr {
            syn::Expr::Binary(bin) => {
                self.transform_expr(&mut bin.left);
                self.transform_expr(&mut bin.right);
            }
            syn::Expr::Unary(un) => self.transform_expr(&mut un.expr),
            syn::Expr::Call(call) => {
                for arg in &mut call.args {
                    self.transform_expr(arg);
                }
            }
            syn::Expr::MethodCall(mc) => {
                self.transform_expr(&mut mc.receiver);
            }
            syn::Expr::Assign(assign) => {
                self.transform_expr(&mut assign.right);
            }
            syn::Expr::Index(idx) => {
                self.transform_expr(&mut idx.expr);
                self.transform_expr(&mut idx.index);
            }
            syn::Expr::If(if_expr) => {
                self.transform_expr(&mut if_expr.cond);
                self.transform_block(&mut if_expr.then_branch);
                if let Some((_, else_expr)) = &mut if_expr.else_branch {
                    self.transform_expr(else_expr);
                }
            }
            syn::Expr::ForLoop(fl) => {
                self.transform_expr(&mut fl.expr);
                collect_pat_names(&fl.pat, false, &mut self.f64_vars);
                self.transform_block(&mut fl.body);
            }
            syn::Expr::Block(block) => self.transform_block(&mut block.block),
            syn::Expr::Paren(p) => self.transform_expr(&mut p.expr),
            syn::Expr::Group(g) => self.transform_expr(&mut g.expr),
            syn::Expr::Reference(r) => self.transform_expr(&mut r.expr),
            syn::Expr::Return(ret) => {
                if let Some(e) = &mut ret.expr {
                    self.transform_expr(e);
                }
            }
            syn::Expr::Array(arr) => {
                for elem in &mut arr.elems {
                    self.transform_expr(elem);
                }
            }
            syn::Expr::Tuple(tup) => {
                for elem in &mut tup.elems {
                    self.transform_expr(elem);
                }
            }
            syn::Expr::Range(r) => {
                if let Some(from) = &mut r.start {
                    self.transform_expr(from);
                }
                if let Some(to) = &mut r.end {
                    self.transform_expr(to);
                }
            }
            syn::Expr::Break(brk) => {
                if let Some(e) = &mut brk.expr {
                    self.transform_expr(e);
                }
            }
            syn::Expr::Field(f) => self.transform_expr(&mut f.base),
            syn::Expr::Match(m) => {
                self.transform_expr(&mut m.expr);
                for arm in &mut m.arms {
                    self.transform_expr(&mut arm.body);
                }
            }
            syn::Expr::Macro(mac) => {
                // vec![f64_expr; N] or vec![f64_expr, ...]
                // Tokens inside the macro can't be easily transformed with syn
                // → skip macro bodies for now
                let _ = mac;
            }
            _ => {}
        }
    }
}

// ── Type helpers ─────────────────────────────────────────────────────────────

fn type_is_f64(ty: &syn::Type) -> bool {
    let s = quote::quote! { #ty }.to_string();
    s == "f64" || s == "f32" || s.contains("f64") && !s.contains("usize")
}

fn expr_is_f64(expr: &syn::Expr, f64_vars: &HashSet<String>) -> bool {
    match expr {
        syn::Expr::Lit(lit) => matches!(&lit.lit, syn::Lit::Float(_)),
        syn::Expr::Path(path) => path
            .path
            .get_ident()
            .map_or(false, |id| f64_vars.contains(&id.to_string())),
        syn::Expr::Call(call) => {
            // f64_helpers::add/sub/mul/div/neg/abs/const_* → f64
            // f64_helpers::eq/gt/lt/ge/le → bool
            if let syn::Expr::Path(path) = &*call.func {
                if let Some(seg) = path.path.segments.last() {
                    let name = seg.ident.to_string();
                    if name == "eq" || name == "gt" || name == "lt"
                        || name == "ge" || name == "le"
                    {
                        return false; // comparison → bool
                    }
                    return name == "add"
                        || name == "sub"
                        || name == "mul"
                        || name == "div"
                        || name == "neg"
                        || name == "abs"
                        || name == "const_0"
                        || name == "const_1"
                        || name == "from_usize"
                        || name.starts_with("const_");
                }
            }
            false
        }
        syn::Expr::Binary(bin) => {
            // If either operand is f64, the result is f64
            expr_is_f64(&bin.left, f64_vars) || expr_is_f64(&bin.right, f64_vars)
        }
        syn::Expr::Unary(un) => {
            matches!(un.op, syn::UnOp::Neg(_)) && expr_is_f64(&un.expr, f64_vars)
        }
        syn::Expr::MethodCall(mc) => {
            if mc.method == "abs" {
                expr_is_f64(&mc.receiver, f64_vars)
            } else {
                false
            }
        }
        syn::Expr::Index(idx) => {
            // a[idx] where a: &[f64] or Vec<f64> → f64
            let base_ty = expr_type_string(&idx.expr, f64_vars);
            base_ty.contains("f64")
        }
        syn::Expr::Paren(p) => expr_is_f64(&p.expr, f64_vars),
        syn::Expr::Group(g) => expr_is_f64(&g.expr, f64_vars),
        syn::Expr::Reference(r) => expr_is_f64(&r.expr, f64_vars),
        _ => false,
    }
}

/// Guess the Rust type string of an expression (best-effort, for Index lookup).
fn expr_type_string(expr: &syn::Expr, f64_vars: &HashSet<String>) -> String {
    match expr {
        syn::Expr::Path(path) => {
            let name = path.path.get_ident().map(|i| i.to_string()).unwrap_or_default();
            if f64_vars.contains(&name) {
                "f64".into()
            } else if name == "matrix" || name == "a" || name == "b" || name == "c"
                || name == "expected" || name.contains("vec")
            {
                // Heuristic: common array/vec names in numerical code
                "Vec<f64>".into()
            } else {
                "usize".into()
            }
        }
        syn::Expr::Reference(r) => expr_type_string(&r.expr, f64_vars),
        _ => "unknown".into(),
    }
}

fn collect_pat_names(pat: &syn::Pat, is_f64: bool, out: &mut HashSet<String>) {
    match pat {
        syn::Pat::Ident(pi) => {
            if is_f64 {
                out.insert(pi.ident.to_string());
            }
        }
        syn::Pat::Type(pt) => {
            let is_f64 = type_is_f64(&pt.ty);
            collect_pat_names(&pt.pat, is_f64, out);
        }
        syn::Pat::Reference(pr) => collect_pat_names(&pr.pat, is_f64, out),
        syn::Pat::Tuple(pt) => {
            for elem in &pt.elems {
                collect_pat_names(elem, is_f64, out);
            }
        }
        _ => {}
    }
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

    if let Ok(mut child) = Command::new("rustfmt")
        .arg("--edition").arg("2021")
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

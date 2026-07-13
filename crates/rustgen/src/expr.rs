use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use vr_ir::{BinaryOp, BuiltinOp, Expr, MatchArm};

use crate::lit::gen_literal;
use crate::pat::gen_pattern;
use crate::stmt::gen_block;
use crate::ty::gen_path;

fn gen_binop(op: BinaryOp) -> TokenStream {
    match op {
        BinaryOp::Add => quote!(+),
        BinaryOp::Sub => quote!(-),
        BinaryOp::Mul => quote!(*),
        BinaryOp::Div => quote!(/),
        BinaryOp::Rem => quote!(%),
        BinaryOp::Eq => quote!(==),
        BinaryOp::Ne => quote!(!=),
        BinaryOp::Lt => quote!(<),
        BinaryOp::Le => quote!(<=),
        BinaryOp::Gt => quote!(>),
        BinaryOp::Ge => quote!(>=),
        BinaryOp::And => quote!(&&),
        BinaryOp::Or => quote!(||),
    }
}

pub(crate) fn gen_expr(expr: &Expr) -> TokenStream {
    match expr {
        Expr::Lit(lit) => gen_literal(lit),
        Expr::Var(name) => {
            let id = format_ident!("{}", name);
            quote!(#id)
        }
        Expr::Path(segs) => gen_path(segs),
        Expr::Field { base, name } => {
            let base = gen_expr(base);
            let name = format_ident!("{}", name);
            quote!(#base.#name)
        }
        Expr::Call { func, args } => {
            let func = gen_expr(func);
            let args = args.iter().map(gen_expr);
            quote!(#func(#(#args),*))
        }
        Expr::MethodCall {
            receiver,
            method,
            args,
        } => {
            let receiver = gen_expr(receiver);
            let method = format_ident!("{}", method);
            let args = args.iter().map(gen_expr);
            quote!(#receiver.#method(#(#args),*))
        }
        Expr::Binary { op, lhs, rhs } => {
            let opt = gen_binop(*op);
            let lhs = gen_expr(lhs);
            let rhs = gen_expr(rhs);
            quote!((#lhs #opt #rhs))
        }
        Expr::Ref { mutable, expr } => {
            let inner = gen_expr(expr);
            if *mutable {
                quote!(&mut #inner)
            } else {
                quote!(&#inner)
            }
        }
        Expr::StructLit { name, fields } => {
            let name = format_ident!("{}", name);
            let fields = fields.iter().map(|(fname, fexpr)| {
                let fname = format_ident!("{}", fname);
                let fexpr = gen_expr(fexpr);
                quote!(#fname: #fexpr)
            });
            quote!(#name { #(#fields),* })
        }
        Expr::Match { scrutinee, arms } => {
            let scrutinee = gen_expr(scrutinee);
            let arms = arms.iter().map(gen_arm);
            quote!(match #scrutinee { #(#arms)* })
        }
        Expr::If { cond, then, else_ } => {
            let cond = gen_expr(cond);
            let then = gen_block(then);
            match else_ {
                Some(e) => {
                    let e = gen_expr(e);
                    quote!(if #cond #then else #e)
                }
                None => quote!(if #cond #then),
            }
        }
        Expr::Block(block) => gen_block(block),
        Expr::Try(inner) => {
            let inner = gen_expr(inner);
            quote!(#inner?)
        }
        Expr::Builtin { op, args } => gen_builtin(op, args),
    }
}

fn gen_arm(arm: &MatchArm) -> TokenStream {
    let pat = gen_pattern(&arm.pattern);
    let body = gen_expr(&arm.body);
    match &arm.guard {
        Some(g) => {
            let g = gen_expr(g);
            quote!(#pat if #g => #body,)
        }
        None => quote!(#pat => #body,),
    }
}

fn gen_builtin(op: &BuiltinOp, args: &[Expr]) -> TokenStream {
    let args_ts: Vec<TokenStream> = args.iter().map(gen_expr).collect();
    match op {
        BuiltinOp::PrintLine(tmpl) => {
            if args_ts.is_empty() {
                quote!(println!(#tmpl))
            } else {
                quote!(println!(#tmpl, #(#args_ts),*))
            }
        }
        BuiltinOp::EPrintLine(tmpl) => {
            if args_ts.is_empty() {
                quote!(eprintln!(#tmpl))
            } else {
                quote!(eprintln!(#tmpl, #(#args_ts),*))
            }
        }
        BuiltinOp::ReadFileToString => {
            let path = &args_ts[0];
            quote!(std::fs::read_to_string(#path).map_err(|e| e.to_string()))
        }
        BuiltinOp::NthArg(n) => {
            let n = proc_macro2::Literal::usize_unsuffixed(*n);
            quote!(std::env::args().nth(#n))
        }
        BuiltinOp::Exit => {
            let code = &args_ts[0];
            quote!(std::process::exit(#code))
        }
    }
}

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use vr_ir::{AssignOp, Block, Stmt};

use crate::expr::gen_expr;
use crate::ty::gen_type;

pub(crate) fn gen_block(block: &Block) -> TokenStream {
    let stmts = block.stmts.iter().map(gen_stmt);
    match &block.tail {
        Some(tail) => {
            let tail = gen_expr(tail);
            quote!({ #(#stmts)* #tail })
        }
        None => quote!({ #(#stmts)* }),
    }
}

fn gen_assign_op(op: AssignOp) -> TokenStream {
    match op {
        AssignOp::Assign => quote!(=),
        AssignOp::Add => quote!(+=),
        AssignOp::Sub => quote!(-=),
        AssignOp::Mul => quote!(*=),
        AssignOp::Div => quote!(/=),
        AssignOp::Rem => quote!(%=),
    }
}

fn gen_stmt(stmt: &Stmt) -> TokenStream {
    match stmt {
        Stmt::Let {
            name,
            mutable,
            ty,
            value,
        } => {
            let id = format_ident!("{}", name);
            let value = gen_expr(value);
            let mut_kw = if *mutable { quote!(mut) } else { quote!() };
            match ty {
                Some(t) => {
                    let t = gen_type(t);
                    quote!(let #mut_kw #id: #t = #value;)
                }
                None => quote!(let #mut_kw #id = #value;),
            }
        }
        Stmt::Assign { target, op, value } => {
            let target = gen_expr(target);
            let opt = gen_assign_op(*op);
            let value = gen_expr(value);
            quote!(#target #opt #value;)
        }
        Stmt::ForEach {
            binding,
            iter,
            body,
        } => {
            let binding = format_ident!("{}", binding);
            let iter = gen_expr(iter);
            let body = gen_block(body);
            quote!(for #binding in #iter #body)
        }
        Stmt::Expr(e) => {
            let e = gen_expr(e);
            quote!(#e;)
        }
        Stmt::Return(opt) => match opt {
            Some(e) => {
                let e = gen_expr(e);
                quote!(return #e;)
            }
            None => quote!(return;),
        },
    }
}

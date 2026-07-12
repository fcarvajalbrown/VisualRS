use crate::expr::Expr;
use crate::ops::AssignOp;
use crate::ty::Type;

/// A braced block: zero or more statements, then an optional trailing
/// expression that is the block's value.
#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub tail: Option<Box<Expr>>,
}

impl Block {
    pub fn new(stmts: Vec<Stmt>, tail: Option<Expr>) -> Self {
        Block {
            stmts,
            tail: tail.map(Box::new),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    Let {
        name: String,
        mutable: bool,
        ty: Option<Type>,
        value: Expr,
    },
    Assign {
        target: Expr,
        op: AssignOp,
        value: Expr,
    },
    ForEach {
        binding: String,
        iter: Expr,
        body: Block,
    },
    Expr(Expr),
    Return(Option<Expr>),
}

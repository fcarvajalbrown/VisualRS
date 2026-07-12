//! Visual Rust Typed IR: a target-agnostic, graph-agnostic representation of a
//! program. No Rust-AST types appear here (ADR-0005).

pub mod expr;
pub mod fixtures;
pub mod item;
pub mod lit;
pub mod ops;
pub mod pat;
pub mod stmt;
pub mod ty;
pub mod validate;

pub use expr::{BuiltinOp, Expr, MatchArm};
pub use item::{
    EnumDef, Field, FunctionDef, Item, Param, Program, StructDef, Variant, VariantPayload,
};
pub use lit::Literal;
pub use ops::{AssignOp, BinaryOp};
pub use pat::Pattern;
pub use stmt::{Block, Stmt};
pub use ty::Type;

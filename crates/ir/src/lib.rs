//! Visual Rust Typed IR: a target-agnostic, graph-agnostic representation of a
//! program. No Rust-AST types appear here (ADR-0005).

pub mod lit;
pub mod ops;
pub mod ty;

pub use lit::Literal;
pub use ops::{AssignOp, BinaryOp};
pub use ty::Type;

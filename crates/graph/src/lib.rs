//! Visual Rust graph front-end: a Blueprints-style node/pin/wire model of a
//! program, plus lowering to the target-agnostic `vr_ir`. Depends on `vr-ir`
//! only; it knows nothing about Rust syntax (that is `vr-rustgen`) or Godot
//! (that is the deferred editor plugin). See
//! `docs/superpowers/specs/2026-07-12-phase2-headless-graph-core-design.md`.

pub mod build;
pub mod fixtures;
pub mod lower;
pub mod model;
pub mod validate;

pub use lower::{lower, LowerError};

pub use model::{
    Arm, Block, DataEdge, EnumDecl, FunctionGraph, Graph, GraphItem, Leaf, Node, NodeId, NodeKind,
    StructDecl, VariantDecl,
};

//! The tiny flat seed graph the walking skeleton renders:
//! `let n = 1 + 2; println!("n: {}", n);`. Mirrors vr-graph's own
//! `arithmetic_graph` fixture so it exercises the real builder API.

use vr_graph::build::{binary, builtin, int, var, BlockBuilder};
use vr_graph::{FunctionGraph, Graph, GraphItem, NodeKind};
use vr_ir::{BinaryOp, BuiltinOp, Type};

/// Build the seed `main()` graph:
///
/// ```ignore
/// fn main() {
///     let n = (1 + 2);
///     println!("n: {}", n);
/// }
/// ```
pub fn seed_graph() -> Graph {
    let mut b = BlockBuilder::new();

    // let n = 1 + 2;
    let sum = binary(&mut b, BinaryOp::Add, int(1), int(2));
    let let_n = b.stmt(NodeKind::Let {
        name: "n".into(),
        mutable: false,
    });
    b.feed(let_n, 0, sum);

    // println!("n: {}", n);
    let print = builtin(&mut b, BuiltinOp::PrintLine("n: {}".into()), vec![var("n")]);
    let es = b.stmt(NodeKind::ExprStmt);
    b.feed(es, 0, print);

    Graph {
        items: vec![GraphItem::Function(FunctionGraph {
            name: "main".into(),
            params: vec![],
            ret: Type::Unit,
            body: b.build(),
        })],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_graph_is_valid() {
        assert!(seed_graph().validate().is_ok());
    }
}

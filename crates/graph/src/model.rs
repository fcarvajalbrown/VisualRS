use std::collections::BTreeMap;
use vr_ir::{AssignOp, BinaryOp, Literal, Pattern, Type, VariantPayload};

/// A stable identity for a node within a single `Block`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(pub u32);

/// An inline value on an input pin: the "default value widget" of a Blueprints
/// input port. Used for trivial leaves instead of a wired node.
#[derive(Clone, Debug, PartialEq)]
pub enum Leaf {
    Lit(Literal),
    /// A binding reference by name (a "Get" node), e.g. `report`, `line`.
    Var(String),
    /// A path constant, e.g. `LineKind::Blank`, `Ok`, `None`.
    Path(Vec<String>),
}

/// A data wire: the single output of `from` feeds input port `to_port` of `to`.
#[derive(Clone, Debug, PartialEq)]
pub struct DataEdge {
    pub from: NodeId,
    pub to: NodeId,
    pub to_port: u16,
}

/// A node plus any inline leaf values for input ports that are not wired.
#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    pub kind: NodeKind,
    pub inline: Vec<(u16, Leaf)>,
}

/// Statement/control nodes (threaded by exec edges) and value nodes (connected
/// by data edges). Port conventions: `Let` in0=value; `Assign` in0=target,
/// in1=value; `ForEach` in0=iter; `ExprStmt` in0=expr; `Return` in0=value;
/// `Field` in0=base; `Call` in0=func, in1..=args; `Method` in0=receiver,
/// in1..=args; `Binary` in0=lhs, in1=rhs; `Ref` in0=value; `StructLit`
/// in_i=field i; `Builtin` in0..=args; `Try` in0=value; `Match` in0=scrutinee;
/// `If` in0=cond. Every value node has exactly one output.
#[derive(Clone, Debug, PartialEq)]
pub enum NodeKind {
    // --- statement / control (threaded by `Block::exec`) ---
    Let {
        name: String,
        mutable: bool,
    },
    Assign {
        op: AssignOp,
    },
    ForEach {
        binding: String,
        body: Block,
    },
    ExprStmt,
    Return {
        has_value: bool,
    },
    // --- value (connected by `Block::data` / inline leaves) ---
    Field {
        name: String,
    },
    Call,
    Method {
        method: String,
    },
    Binary {
        op: BinaryOp,
    },
    Ref {
        mutable: bool,
    },
    StructLit {
        name: String,
        fields: Vec<String>,
    },
    Builtin {
        op: vr_ir::BuiltinOp,
    },
    Try,
    Match {
        arms: Vec<Arm>,
    },
    If {
        then: Block,
        els: Option<Block>,
    },
    /// A path used directly as a value, e.g. `LineKind::Blank` as a block tail.
    PathValue(Vec<String>),
    /// A binding used directly as a value, e.g. `report` as a block tail.
    VarValue(String),
}

/// A braced scope: statement nodes threaded from `entry` via `exec`, value nodes
/// wired via `data`, and an optional trailing value node `tail`.
#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub nodes: BTreeMap<NodeId, Node>,
    /// Sequential exec thread: `(from_stmt, to_stmt)` pairs.
    pub exec: Vec<(NodeId, NodeId)>,
    pub data: Vec<DataEdge>,
    pub entry: Option<NodeId>,
    pub tail: Option<NodeId>,
}

/// One arm of a `Match` value node. `body`/`guard` are sub-blocks.
#[derive(Clone, Debug, PartialEq)]
pub struct Arm {
    pub pattern: Pattern,
    pub guard: Option<Block>,
    pub body: Block,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StructDecl {
    pub name: String,
    pub fields: Vec<(String, Type)>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VariantDecl {
    pub name: String,
    pub payload: VariantPayload,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EnumDecl {
    pub name: String,
    pub variants: Vec<VariantDecl>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionGraph {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub ret: Type,
    pub body: Block,
}

#[derive(Clone, Debug, PartialEq)]
pub enum GraphItem {
    Struct(StructDecl),
    Enum(EnumDecl),
    Function(FunctionGraph),
}

/// A whole program as a graph: declarative struct/enum decls plus function
/// bodies as node graphs.
#[derive(Clone, Debug, PartialEq)]
pub struct Graph {
    pub items: Vec<GraphItem>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use vr_ir::Literal;

    #[test]
    fn block_holds_nodes_and_edges() {
        let mut nodes = std::collections::BTreeMap::new();
        nodes.insert(
            NodeId(0),
            Node {
                kind: NodeKind::Let {
                    name: "n".into(),
                    mutable: false,
                },
                inline: vec![(0, Leaf::Lit(Literal::Int(1)))],
            },
        );
        let block = Block {
            nodes,
            exec: vec![],
            data: vec![],
            entry: Some(NodeId(0)),
            tail: None,
        };
        assert_eq!(block.entry, Some(NodeId(0)));
        assert!(matches!(block.nodes[&NodeId(0)].kind, NodeKind::Let { .. }));
    }
}

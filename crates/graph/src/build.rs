use std::collections::BTreeMap;
use vr_ir::{BinaryOp, BuiltinOp, Literal};

use crate::model::{Block, DataEdge, Leaf, Node, NodeId, NodeKind};

/// A resolved input source: either a wired node output or an inline leaf.
#[derive(Clone, Debug)]
pub enum Src {
    Node(NodeId),
    Leaf(Leaf),
}

/// Incrementally assembles a `Block`, assigning `NodeId`s and threading exec.
pub struct BlockBuilder {
    nodes: BTreeMap<NodeId, Node>,
    exec: Vec<(NodeId, NodeId)>,
    data: Vec<DataEdge>,
    entry: Option<NodeId>,
    last_stmt: Option<NodeId>,
    tail: Option<NodeId>,
    next: u32,
}

impl BlockBuilder {
    pub fn new() -> Self {
        BlockBuilder {
            nodes: BTreeMap::new(),
            exec: vec![],
            data: vec![],
            entry: None,
            last_stmt: None,
            tail: None,
            next: 0,
        }
    }

    fn fresh(&mut self) -> NodeId {
        let id = NodeId(self.next);
        self.next += 1;
        id
    }

    /// Insert a value node (no exec threading) and return its id.
    pub fn value(&mut self, kind: NodeKind) -> NodeId {
        let id = self.fresh();
        self.nodes.insert(id, Node { kind, inline: vec![] });
        id
    }

    /// Insert a statement node and thread it onto the exec chain.
    pub fn stmt(&mut self, kind: NodeKind) -> NodeId {
        let id = self.fresh();
        self.nodes.insert(id, Node { kind, inline: vec![] });
        match self.last_stmt {
            None => self.entry = Some(id),
            Some(prev) => self.exec.push((prev, id)),
        }
        self.last_stmt = Some(id);
        id
    }

    /// Feed input port `port` of `node` from `src` (a wire or an inline leaf).
    pub fn feed(&mut self, node: NodeId, port: u16, src: Src) {
        match src {
            Src::Node(from) => self.data.push(DataEdge {
                from,
                to: node,
                to_port: port,
            }),
            Src::Leaf(leaf) => {
                if let Some(n) = self.nodes.get_mut(&node) {
                    n.inline.push((port, leaf));
                }
            }
        }
    }

    pub fn set_tail(&mut self, id: NodeId) {
        self.tail = Some(id);
    }

    /// Convenience for tests: set the tail from a `Src`. Tails are value nodes;
    /// bare leaves must be wrapped in a value node (`PathValue`/`VarValue`).
    pub fn set_tail_src(&mut self, src: Src) {
        match src {
            Src::Node(id) => self.tail = Some(id),
            Src::Leaf(_) => panic!("tail must be a value node; wrap leaves in a value node"),
        }
    }

    pub fn build(self) -> Block {
        Block {
            nodes: self.nodes,
            exec: self.exec,
            data: self.data,
            entry: self.entry,
            tail: self.tail,
        }
    }
}

impl Default for BlockBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// --- leaf value helpers (return Src::Leaf) ----------------------------------
pub fn lit(l: Literal) -> Src {
    Src::Leaf(Leaf::Lit(l))
}
pub fn int(v: i128) -> Src {
    Src::Leaf(Leaf::Lit(Literal::Int(v)))
}
pub fn var(name: &str) -> Src {
    Src::Leaf(Leaf::Var(name.into()))
}
pub fn path(segs: &[&str]) -> Src {
    Src::Leaf(Leaf::Path(segs.iter().map(|s| s.to_string()).collect()))
}

// --- node value helpers (insert a node, wire inputs, return Src::Node) -------
pub fn field(b: &mut BlockBuilder, base: Src, name: &str) -> Src {
    let id = b.value(NodeKind::Field { name: name.into() });
    b.feed(id, 0, base);
    Src::Node(id)
}
pub fn method(b: &mut BlockBuilder, receiver: Src, m: &str, args: Vec<Src>) -> Src {
    let id = b.value(NodeKind::Method { method: m.into() });
    b.feed(id, 0, receiver);
    for (i, a) in args.into_iter().enumerate() {
        b.feed(id, (i + 1) as u16, a);
    }
    Src::Node(id)
}
pub fn call(b: &mut BlockBuilder, func: Src, args: Vec<Src>) -> Src {
    let id = b.value(NodeKind::Call);
    b.feed(id, 0, func);
    for (i, a) in args.into_iter().enumerate() {
        b.feed(id, (i + 1) as u16, a);
    }
    Src::Node(id)
}
pub fn binary(b: &mut BlockBuilder, op: BinaryOp, lhs: Src, rhs: Src) -> Src {
    let id = b.value(NodeKind::Binary { op });
    b.feed(id, 0, lhs);
    b.feed(id, 1, rhs);
    Src::Node(id)
}
pub fn reference(b: &mut BlockBuilder, mutable: bool, value: Src) -> Src {
    let id = b.value(NodeKind::Ref { mutable });
    b.feed(id, 0, value);
    Src::Node(id)
}
pub fn struct_lit(b: &mut BlockBuilder, name: &str, fields: Vec<(&str, Src)>) -> Src {
    let names: Vec<String> = fields.iter().map(|(n, _)| n.to_string()).collect();
    let id = b.value(NodeKind::StructLit {
        name: name.into(),
        fields: names,
    });
    for (i, (_, s)) in fields.into_iter().enumerate() {
        b.feed(id, i as u16, s);
    }
    Src::Node(id)
}
pub fn builtin(b: &mut BlockBuilder, op: BuiltinOp, args: Vec<Src>) -> Src {
    let id = b.value(NodeKind::Builtin { op });
    for (i, a) in args.into_iter().enumerate() {
        b.feed(id, i as u16, a);
    }
    Src::Node(id)
}
pub fn try_(b: &mut BlockBuilder, value: Src) -> Src {
    let id = b.value(NodeKind::Try);
    b.feed(id, 0, value);
    Src::Node(id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use vr_ir::BinaryOp;

    #[test]
    fn builds_a_block_with_a_wired_binary_tail() {
        // { 1 + 2 }  (as a tail value)
        let mut b = BlockBuilder::new();
        let sum = binary(&mut b, BinaryOp::Add, int(1), int(2));
        b.set_tail_src(sum);
        let block = b.build();
        assert_eq!(block.nodes.len(), 1); // the Binary node; operands are inline leaves
        assert!(block.tail.is_some());
    }
}

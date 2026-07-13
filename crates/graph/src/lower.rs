use std::fmt;

use vr_ir::{
    Block as IrBlock, EnumDef, Expr, Field as IrField, FunctionDef, Item, MatchArm, Param, Program,
    Stmt, StructDef, Variant,
};

use crate::model::{Arm, Block, Graph, GraphItem, Leaf, NodeId, NodeKind};

/// A lowering failure. A validated graph should never produce these.
#[derive(Clone, Debug, PartialEq)]
pub enum LowerError {
    MissingInput { port: u16 },
    MissingEntry,
    DanglingNode(NodeId),
    NoTailValue,
}

impl fmt::Display for LowerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LowerError::MissingInput { port } => write!(f, "input port {port} has no source"),
            LowerError::MissingEntry => write!(f, "block has statements-shaped exec but no entry"),
            LowerError::DanglingNode(n) => write!(f, "node {n:?} referenced but not present"),
            LowerError::NoTailValue => write!(f, "expected a tail value but found none"),
        }
    }
}

impl std::error::Error for LowerError {}

/// Lower a whole graph to a `vr_ir::Program`.
pub fn lower(graph: &Graph) -> Result<Program, LowerError> {
    let mut items = Vec::new();
    for item in &graph.items {
        items.push(match item {
            GraphItem::Struct(s) => Item::Struct(StructDef {
                name: s.name.clone(),
                fields: s
                    .fields
                    .iter()
                    .map(|(n, t)| IrField {
                        name: n.clone(),
                        ty: t.clone(),
                    })
                    .collect(),
            }),
            GraphItem::Enum(e) => Item::Enum(EnumDef {
                name: e.name.clone(),
                variants: e
                    .variants
                    .iter()
                    .map(|v| Variant {
                        name: v.name.clone(),
                        payload: v.payload.clone(),
                    })
                    .collect(),
            }),
            GraphItem::Function(func) => Item::Function(FunctionDef {
                name: func.name.clone(),
                params: func
                    .params
                    .iter()
                    .map(|(n, t)| Param {
                        name: n.clone(),
                        ty: t.clone(),
                    })
                    .collect(),
                ret: func.ret.clone(),
                body: lower_block(&func.body)?,
            }),
        });
    }
    Ok(Program { items })
}

/// Follow the exec thread from `entry`, lowering statements in order.
fn lower_block(block: &Block) -> Result<IrBlock, LowerError> {
    let mut stmts = Vec::new();
    let mut cur = block.entry;
    while let Some(id) = cur {
        stmts.push(lower_stmt(block, id)?);
        cur = next_exec(block, id);
    }
    let tail = match block.tail {
        Some(t) => Some(lower_value(block, t)?),
        None => None,
    };
    Ok(IrBlock {
        stmts,
        tail: tail.map(Box::new),
    })
}

fn next_exec(block: &Block, from: NodeId) -> Option<NodeId> {
    block.exec.iter().find(|(a, _)| *a == from).map(|(_, b)| *b)
}

fn lower_stmt(block: &Block, id: NodeId) -> Result<Stmt, LowerError> {
    let node = block.nodes.get(&id).ok_or(LowerError::DanglingNode(id))?;
    Ok(match &node.kind {
        NodeKind::Let { name, mutable } => Stmt::Let {
            name: name.clone(),
            mutable: *mutable,
            ty: None,
            value: resolve(block, id, 0)?,
        },
        NodeKind::Assign { op } => Stmt::Assign {
            target: resolve(block, id, 0)?,
            op: *op,
            value: resolve(block, id, 1)?,
        },
        NodeKind::ForEach { binding, body } => Stmt::ForEach {
            binding: binding.clone(),
            iter: resolve(block, id, 0)?,
            body: lower_block(body)?,
        },
        NodeKind::ExprStmt => Stmt::Expr(resolve(block, id, 0)?),
        NodeKind::Return { has_value } => {
            if *has_value {
                Stmt::Return(Some(resolve(block, id, 0)?))
            } else {
                Stmt::Return(None)
            }
        }
        _ => return Err(LowerError::DanglingNode(id)), // a value node used as a statement
    })
}

/// Resolve input `port` of `node`: a data edge lowers its source value; else the
/// inline leaf lowers; else the input is missing.
fn resolve(block: &Block, node: NodeId, port: u16) -> Result<Expr, LowerError> {
    if let Some(edge) = block
        .data
        .iter()
        .find(|e| e.to == node && e.to_port == port)
    {
        return lower_value(block, edge.from);
    }
    let n = block
        .nodes
        .get(&node)
        .ok_or(LowerError::DanglingNode(node))?;
    if let Some((_, leaf)) = n.inline.iter().find(|(p, _)| *p == port) {
        return Ok(lower_leaf(leaf));
    }
    Err(LowerError::MissingInput { port })
}

fn lower_leaf(leaf: &Leaf) -> Expr {
    match leaf {
        Leaf::Lit(l) => Expr::Lit(l.clone()),
        Leaf::Var(n) => Expr::Var(n.clone()),
        Leaf::Path(segs) => Expr::Path(segs.clone()),
    }
}

/// Highest input port index (+1) that has any source on `node`.
fn input_count(block: &Block, node: NodeId) -> u16 {
    let mut max: i32 = -1;
    for e in block.data.iter().filter(|e| e.to == node) {
        max = max.max(e.to_port as i32);
    }
    if let Some(n) = block.nodes.get(&node) {
        for (p, _) in &n.inline {
            max = max.max(*p as i32);
        }
    }
    (max + 1) as u16
}

fn resolve_variadic(block: &Block, node: NodeId, start: u16) -> Result<Vec<Expr>, LowerError> {
    let count = input_count(block, node);
    let mut out = Vec::new();
    for p in start..count {
        out.push(resolve(block, node, p)?);
    }
    Ok(out)
}

fn lower_value(block: &Block, id: NodeId) -> Result<Expr, LowerError> {
    let node = block.nodes.get(&id).ok_or(LowerError::DanglingNode(id))?;
    Ok(match &node.kind {
        NodeKind::Field { name } => Expr::Field {
            base: Box::new(resolve(block, id, 0)?),
            name: name.clone(),
        },
        NodeKind::Call => {
            let func = resolve(block, id, 0)?;
            let args = resolve_variadic(block, id, 1)?;
            Expr::Call {
                func: Box::new(func),
                args,
            }
        }
        NodeKind::Method { method } => {
            let receiver = resolve(block, id, 0)?;
            let args = resolve_variadic(block, id, 1)?;
            Expr::MethodCall {
                receiver: Box::new(receiver),
                method: method.clone(),
                args,
            }
        }
        NodeKind::Binary { op } => Expr::Binary {
            op: *op,
            lhs: Box::new(resolve(block, id, 0)?),
            rhs: Box::new(resolve(block, id, 1)?),
        },
        NodeKind::Ref { mutable } => Expr::Ref {
            mutable: *mutable,
            expr: Box::new(resolve(block, id, 0)?),
        },
        NodeKind::StructLit { name, fields } => {
            let mut out = Vec::new();
            for (i, fname) in fields.iter().enumerate() {
                out.push((fname.clone(), resolve(block, id, i as u16)?));
            }
            Expr::StructLit {
                name: name.clone(),
                fields: out,
            }
        }
        NodeKind::Builtin { op } => {
            let args = resolve_variadic(block, id, 0)?;
            Expr::Builtin {
                op: op.clone(),
                args,
            }
        }
        NodeKind::Try => Expr::Try(Box::new(resolve(block, id, 0)?)),
        NodeKind::Match { arms } => {
            let scrutinee = resolve(block, id, 0)?;
            let mut ir_arms = Vec::new();
            for a in arms {
                ir_arms.push(lower_arm(a)?);
            }
            Expr::Match {
                scrutinee: Box::new(scrutinee),
                arms: ir_arms,
            }
        }
        NodeKind::If { then, els } => {
            let cond = resolve(block, id, 0)?;
            let then_block = lower_block(then)?;
            let else_expr = match els {
                Some(eb) => Some(Box::new(lower_else(eb)?)),
                None => None,
            };
            Expr::If {
                cond: Box::new(cond),
                then: then_block,
                else_: else_expr,
            }
        }
        NodeKind::PathValue(segs) => Expr::Path(segs.clone()),
        NodeKind::VarValue(n) => Expr::Var(n.clone()),
        // statement kinds are not values
        NodeKind::Let { .. }
        | NodeKind::Assign { .. }
        | NodeKind::ForEach { .. }
        | NodeKind::ExprStmt
        | NodeKind::Return { .. } => return Err(LowerError::DanglingNode(id)),
    })
}

fn lower_arm(arm: &Arm) -> Result<MatchArm, LowerError> {
    let guard = match &arm.guard {
        Some(g) => Some(lower_arm_body(g)?),
        None => None,
    };
    Ok(MatchArm {
        pattern: arm.pattern.clone(),
        guard,
        body: lower_arm_body(&arm.body)?,
    })
}

/// A match-arm body: bare tail if there are no statements, else a block expr.
fn lower_arm_body(block: &Block) -> Result<Expr, LowerError> {
    if block.entry.is_none() {
        if let Some(t) = block.tail {
            return lower_value(block, t);
        }
    }
    Ok(Expr::Block(lower_block(block)?))
}

/// An `else` branch: bare only if it is an `else if` (no stmts, tail is an `If`).
fn lower_else(block: &Block) -> Result<Expr, LowerError> {
    if block.entry.is_none() {
        if let Some(t) = block.tail {
            if let Some(n) = block.nodes.get(&t) {
                if matches!(n.kind, NodeKind::If { .. }) {
                    return lower_value(block, t);
                }
            }
        }
    }
    Ok(Expr::Block(lower_block(block)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::*;
    use crate::model::*;
    use vr_ir::{BinaryOp, BuiltinOp, Type};

    // fn main() { let n = 1 + 2; println!("n: {}", n); }
    fn arithmetic_graph() -> Graph {
        let mut b = BlockBuilder::new();
        let sum = binary(&mut b, BinaryOp::Add, int(1), int(2));
        let let_n = b.stmt(NodeKind::Let {
            name: "n".into(),
            mutable: false,
        });
        b.feed(let_n, 0, sum);
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

    #[test]
    fn arithmetic_lowers_and_generates() {
        let prog = lower(&arithmetic_graph()).expect("lowers");
        let src = vr_rustgen::generate(&prog).expect("generates");
        assert!(src.contains("let n = (1 + 2)"), "got:\n{src}");
        assert!(src.contains(r#"println!("n: {}", n)"#), "got:\n{src}");
    }
}

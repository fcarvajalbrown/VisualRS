//! Pure `model -> egui_snarl` renderer. Testable without a display: it only
//! builds a `Snarl<NodeView>` (node data + positions + wires) and never touches
//! an egui `Ui`. This renderer is permanent — it is also how a saved graph would
//! later be loaded onto the canvas.

use std::collections::BTreeMap;

use egui::pos2;
use egui_snarl::{InPinId, OutPinId, Snarl};

use vr_graph::model::{Leaf, Node, NodeId, NodeKind};
use vr_graph::{Graph, GraphItem};

/// Per-node display data. The `SnarlViewer` in `app.rs` reads this to draw a
/// node's title and pins; this module never touches egui `Ui`.
#[derive(Clone, Debug, PartialEq)]
pub struct NodeView {
    pub title: String,
    pub inputs: Vec<InputRow>,
    pub has_output: bool,
}

/// One input pin's display: either a data-wire target (label only) or an inline
/// literal/variable leaf rendered as read-only text.
#[derive(Clone, Debug, PartialEq)]
pub enum InputRow {
    Wired { label: String },
    Inline { text: String },
}

/// Render the first function body of `graph` onto a fresh `Snarl<NodeView>`:
/// one snarl node per model node (in `NodeId` order, laid out top-to-bottom),
/// one wire per data edge. A graph with no function renders as an empty `Snarl`.
pub fn to_snarl(graph: &Graph) -> Snarl<NodeView> {
    let mut snarl = Snarl::new();

    let Some(body) = graph.items.iter().find_map(|item| match item {
        GraphItem::Function(f) => Some(&f.body),
        _ => None,
    }) else {
        return snarl;
    };

    // Ports that are fed by a data edge, grouped per destination node.
    let mut wired: BTreeMap<NodeId, Vec<u16>> = BTreeMap::new();
    for edge in &body.data {
        wired.entry(edge.to).or_default().push(edge.to_port);
    }

    // Insert one snarl node per model node; remember the id mapping for wiring.
    let mut ids: BTreeMap<NodeId, egui_snarl::NodeId> = BTreeMap::new();
    for (i, (node_id, node)) in body.nodes.iter().enumerate() {
        let ports = wired.get(node_id).map(Vec::as_slice).unwrap_or(&[]);
        let view = build_node_view(node, ports);
        let snarl_id = snarl.insert_node(pos2(0.0, i as f32 * 120.0), view);
        ids.insert(*node_id, snarl_id);
    }

    // One wire per data edge: source output 0 -> destination input `to_port`.
    for edge in &body.data {
        if let (Some(&from), Some(&to)) = (ids.get(&edge.from), ids.get(&edge.to)) {
            snarl.connect(
                OutPinId {
                    node: from,
                    output: 0,
                },
                InPinId {
                    node: to,
                    input: edge.to_port as usize,
                },
            );
        }
    }

    snarl
}

/// Build the display data for one node. Input rows cover the union of inline
/// ports and wired ports, in ascending port order.
fn build_node_view(node: &Node, wired_ports: &[u16]) -> NodeView {
    let mut ports: Vec<u16> = node
        .inline
        .iter()
        .map(|(p, _)| *p)
        .chain(wired_ports.iter().copied())
        .collect();
    ports.sort_unstable();
    ports.dedup();

    let inputs = ports
        .iter()
        .map(|&p| match node.inline.iter().find(|(ip, _)| *ip == p) {
            Some((_, leaf)) => InputRow::Inline {
                text: leaf_text(leaf),
            },
            None => InputRow::Wired {
                label: port_label(&node.kind, p),
            },
        })
        .collect();

    NodeView {
        title: node_title(&node.kind),
        inputs,
        has_output: is_value_node(&node.kind),
    }
}

/// Human-readable node title. Only the seed's kinds are special-cased; anything
/// else falls back to the kind's `Debug`.
fn node_title(kind: &NodeKind) -> String {
    match kind {
        NodeKind::Let { name, mutable } => {
            if *mutable {
                format!("let mut {name}")
            } else {
                format!("let {name}")
            }
        }
        NodeKind::ExprStmt => "expr".to_string(),
        NodeKind::Binary { op } => format!("{op:?}"),
        NodeKind::Builtin { op } => builtin_title(op),
        other => format!("{other:?}"),
    }
}

fn builtin_title(op: &vr_ir::BuiltinOp) -> String {
    match op {
        vr_ir::BuiltinOp::PrintLine(tmpl) => format!("println!({tmpl:?})"),
        vr_ir::BuiltinOp::EPrintLine(tmpl) => format!("eprintln!({tmpl:?})"),
        other => format!("{other:?}"),
    }
}

/// Whether a node produces a value (has exactly one output pin). Statement /
/// control nodes have none.
fn is_value_node(kind: &NodeKind) -> bool {
    use NodeKind::*;
    matches!(
        kind,
        Field { .. }
            | Call
            | Method { .. }
            | Binary { .. }
            | Ref { .. }
            | StructLit { .. }
            | Builtin { .. }
            | Try
            | Match { .. }
            | If { .. }
            | PathValue(_)
            | VarValue(_)
    )
}

fn port_label(kind: &NodeKind, port: u16) -> String {
    match (kind, port) {
        (NodeKind::Let { .. }, 0) => "value".to_string(),
        (NodeKind::ExprStmt, 0) => "expr".to_string(),
        (NodeKind::Binary { .. }, 0) => "lhs".to_string(),
        (NodeKind::Binary { .. }, 1) => "rhs".to_string(),
        (NodeKind::Builtin { .. }, p) => format!("arg{p}"),
        _ => format!("in{port}"),
    }
}

fn leaf_text(leaf: &Leaf) -> String {
    match leaf {
        Leaf::Lit(lit) => literal_text(lit),
        Leaf::Var(name) => name.clone(),
        Leaf::Path(segs) => segs.join("::"),
    }
}

fn literal_text(lit: &vr_ir::Literal) -> String {
    match lit {
        vr_ir::Literal::Int(v) => v.to_string(),
        vr_ir::Literal::Float(v) => v.to_string(),
        vr_ir::Literal::Bool(v) => v.to_string(),
        vr_ir::Literal::Char(c) => format!("'{c}'"),
        vr_ir::Literal::Str(s) => format!("{s:?}"),
        vr_ir::Literal::Unit => "()".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seed::seed_graph;

    #[test]
    fn seed_produces_four_nodes() {
        let snarl = to_snarl(&seed_graph());
        assert_eq!(snarl.nodes().count(), 4);
    }

    #[test]
    fn seed_has_two_data_wires() {
        let snarl = to_snarl(&seed_graph());
        assert_eq!(snarl.wires().count(), 2);
    }

    #[test]
    fn titles_cover_the_seed_nodes() {
        let snarl = to_snarl(&seed_graph());
        let titles: Vec<String> = snarl.nodes().map(|n| n.title.clone()).collect();
        assert!(titles.iter().any(|t| t == "let n"), "titles: {titles:?}");
        assert!(titles.iter().any(|t| t == "expr"), "titles: {titles:?}");
        assert!(
            titles.iter().any(|t| t.contains("Add")),
            "titles: {titles:?}"
        );
        assert!(
            titles.iter().any(|t| t.contains("println!")),
            "titles: {titles:?}"
        );
    }

    #[test]
    fn binary_node_has_two_inline_literal_inputs() {
        let snarl = to_snarl(&seed_graph());
        let add = snarl
            .nodes()
            .find(|n| n.title.contains("Add"))
            .expect("Add node present");
        assert_eq!(add.inputs.len(), 2);
        assert!(matches!(&add.inputs[0], InputRow::Inline { text } if text == "1"));
        assert!(matches!(&add.inputs[1], InputRow::Inline { text } if text == "2"));
        assert!(add.has_output);
    }
}

//! Pure `model -> egui_snarl` renderer. Testable without a display: it only
//! builds a `Snarl<NodeView>` (node data + positions + wires) and never touches
//! an egui `Ui`. This renderer is permanent — it is also how a saved graph would
//! later be loaded onto the canvas.
//!
//! It follows Unreal Blueprints' two-wire model (see the design spec §5.1):
//! **statement** (impure) nodes carry white triangular execution pins threaded by
//! the block's `exec` edges, while **value** (pure) nodes have only coloured data
//! pins carrying the block's `data` edges. Execution pins live at index 0 on each
//! side; data pins follow.
//!
//! Layout reads begin-to-end like a Blueprint graph: a synthetic entry node (the
//! function name) sits at the far left, statements run left-to-right along one
//! execution spine threaded by the white exec wire, and pure value nodes sit above
//! the statement that consumes them, feeding data down.

use std::collections::BTreeMap;

use egui::{pos2, Pos2};
use egui_snarl::{InPinId, OutPinId, Snarl};

use vr_graph::model::{Leaf, Node, NodeId, NodeKind};
use vr_graph::{Graph, GraphItem};

/// Horizontal spacing between spine columns.
const COL_W: f32 = 220.0;
/// Vertical position of the execution spine row.
const ROW_Y: f32 = 260.0;
/// How far above the spine a pure value node sits from its consumer.
const PURE_DY: f32 = 150.0;
/// Extra vertical offset when several pure nodes stack above one statement.
const STACK_DY: f32 = 120.0;

/// Per-node display data. The `SnarlViewer` in `app.rs` reads this to draw a
/// node's title and pins; this module never touches egui `Ui`.
#[derive(Clone, Debug, PartialEq)]
pub struct NodeView {
    pub title: String,
    pub inputs: Vec<InputRow>,
    pub outputs: Vec<OutputRow>,
}

/// One input pin's display, top-to-bottom on the node's left side.
#[derive(Clone, Debug, PartialEq)]
pub enum InputRow {
    /// White triangular execution-in pin (statement nodes only), at index 0.
    Exec,
    /// A data pin fed by a wire; the label names the port.
    Wired { label: String },
    /// A data pin with an inline literal/variable leaf rendered as read-only text.
    Inline { text: String },
}

/// One output pin's display, top-to-bottom on the node's right side.
#[derive(Clone, Debug, PartialEq)]
pub enum OutputRow {
    /// White triangular execution-out pin (entry and statement nodes), at index 0.
    Exec,
    /// The node's single data output (value nodes only).
    Data,
}

/// Pin layout for a single model node: the display `NodeView` plus the index maps
/// `to_snarl` needs to attach exec and data wires to the right pins.
struct Layout {
    view: NodeView,
    /// Input pin index of the exec-in pin, if this is a statement node.
    exec_in: Option<usize>,
    /// Model data port -> input pin index (offset past the exec pin).
    data_in: BTreeMap<u16, usize>,
    /// Output pin index of the exec-out pin, if this is a statement node.
    exec_out: Option<usize>,
    /// Output pin index of the data output, if this is a value node.
    data_out: Option<usize>,
}

/// Render the first function body of `graph` onto a fresh `Snarl<NodeView>`:
/// a synthetic entry node plus one snarl node per model node, wired by data edges
/// and threaded by exec edges into a left-to-right flow. A graph with no function
/// renders as an empty `Snarl`.
pub fn to_snarl(graph: &Graph) -> Snarl<NodeView> {
    let mut snarl = Snarl::new();

    let Some((fname, body)) = graph.items.iter().find_map(|item| match item {
        GraphItem::Function(f) => Some((f.name.as_str(), &f.body)),
        _ => None,
    }) else {
        return snarl;
    };

    // Ports fed by a data edge, grouped per destination node.
    let mut wired: BTreeMap<NodeId, Vec<u16>> = BTreeMap::new();
    for edge in &body.data {
        wired.entry(edge.to).or_default().push(edge.to_port);
    }

    // Pin layout for every model node.
    let mut layouts: BTreeMap<NodeId, Layout> = BTreeMap::new();
    for (node_id, node) in &body.nodes {
        let ports = wired.get(node_id).map(Vec::as_slice).unwrap_or(&[]);
        layouts.insert(*node_id, build_layout(node, ports));
    }

    // Statement execution order, walked from `entry` along exec successors.
    let mut succ: BTreeMap<NodeId, NodeId> = BTreeMap::new();
    for (a, b) in &body.exec {
        succ.insert(*a, *b);
    }
    let mut order: Vec<NodeId> = Vec::new();
    let mut cur = body.entry;
    while let Some(id) = cur {
        if order.contains(&id) {
            break; // defensive: never loop on a malformed cycle
        }
        order.push(id);
        cur = succ.get(&id).copied();
    }
    // Column index per statement; the entry node reserves column 0.
    let mut col: BTreeMap<NodeId, usize> = BTreeMap::new();
    for (i, id) in order.iter().enumerate() {
        col.insert(*id, i + 1);
    }

    // Positions: statements on the spine, pure nodes stacked above their consumer.
    let mut positions: BTreeMap<NodeId, Pos2> = BTreeMap::new();
    for (id, &c) in &col {
        positions.insert(*id, pos2(c as f32 * COL_W, ROW_Y));
    }
    let mut stacked: BTreeMap<usize, usize> = BTreeMap::new();
    for (id, layout) in &layouts {
        if layout.exec_in.is_some() {
            continue; // statement: already placed on the spine
        }
        // Column of the statement this value node feeds (via a data edge).
        let consumer_col = body
            .data
            .iter()
            .find(|e| e.from == *id)
            .and_then(|e| col.get(&e.to).copied())
            .unwrap_or(0);
        let k = stacked.entry(consumer_col).or_insert(0);
        let y = ROW_Y - PURE_DY - (*k as f32) * STACK_DY;
        *k += 1;
        positions.insert(*id, pos2(consumer_col as f32 * COL_W, y));
    }

    // Insert the synthetic entry node (function name) at the head of the spine.
    let begin_id = snarl.insert_node(
        pos2(0.0, ROW_Y),
        NodeView {
            title: fname.to_string(),
            inputs: vec![],
            outputs: vec![OutputRow::Exec],
        },
    );

    // Insert every model node at its computed position.
    let mut ids: BTreeMap<NodeId, egui_snarl::NodeId> = BTreeMap::new();
    for (id, layout) in &layouts {
        let pos = positions.get(id).copied().unwrap_or(pos2(0.0, 0.0));
        let sid = snarl.insert_node(pos, layout.view.clone());
        ids.insert(*id, sid);
    }

    // Exec wire from the entry node into the first statement.
    if let Some(entry) = body.entry {
        if let (Some(&to), Some(input)) =
            (ids.get(&entry), layouts.get(&entry).and_then(|l| l.exec_in))
        {
            snarl.connect(
                OutPinId {
                    node: begin_id,
                    output: 0,
                },
                InPinId { node: to, input },
            );
        }
    }

    // Data wires: value-node output -> destination's data input pin.
    for edge in &body.data {
        if let (Some(&from), Some(&to), Some(from_l), Some(to_l)) = (
            ids.get(&edge.from),
            ids.get(&edge.to),
            layouts.get(&edge.from),
            layouts.get(&edge.to),
        ) {
            if let (Some(out), Some(&input)) = (from_l.data_out, to_l.data_in.get(&edge.to_port)) {
                snarl.connect(
                    OutPinId {
                        node: from,
                        output: out,
                    },
                    InPinId { node: to, input },
                );
            }
        }
    }

    // Exec wires: predecessor statement's exec-out -> successor's exec-in.
    for (a, b) in &body.exec {
        if let (Some(&from), Some(&to), Some(from_l), Some(to_l)) =
            (ids.get(a), ids.get(b), layouts.get(a), layouts.get(b))
        {
            if let (Some(out), Some(input)) = (from_l.exec_out, to_l.exec_in) {
                snarl.connect(
                    OutPinId {
                        node: from,
                        output: out,
                    },
                    InPinId { node: to, input },
                );
            }
        }
    }

    snarl
}

/// Build the pin layout for one node. Statement (impure) nodes get an exec pin at
/// index 0 on each side; data inputs follow. Value (pure) nodes get only data
/// pins and a single data output.
fn build_layout(node: &Node, wired_ports: &[u16]) -> Layout {
    let statement = !is_value_node(&node.kind);

    let mut data_ports: Vec<u16> = node
        .inline
        .iter()
        .map(|(p, _)| *p)
        .chain(wired_ports.iter().copied())
        .collect();
    data_ports.sort_unstable();
    data_ports.dedup();

    let mut inputs = Vec::new();
    let mut data_in = BTreeMap::new();
    let exec_in = if statement {
        inputs.push(InputRow::Exec);
        Some(0)
    } else {
        None
    };
    for &p in &data_ports {
        let idx = inputs.len();
        let row = match node.inline.iter().find(|(ip, _)| *ip == p) {
            Some((_, leaf)) => InputRow::Inline {
                text: leaf_text(leaf),
            },
            None => InputRow::Wired {
                label: port_label(&node.kind, p),
            },
        };
        inputs.push(row);
        data_in.insert(p, idx);
    }

    let (outputs, exec_out, data_out) = if statement {
        (vec![OutputRow::Exec], Some(0), None)
    } else {
        (vec![OutputRow::Data], None, Some(0))
    };

    Layout {
        view: NodeView {
            title: node_title(&node.kind),
            inputs,
            outputs,
        },
        exec_in,
        data_in,
        exec_out,
        data_out,
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

/// Whether a node produces a value (pure node: data pins only, no exec pins).
/// Statement / control nodes return `false` — they are impure and carry exec pins.
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
    fn seed_produces_five_nodes_including_entry() {
        // 4 model nodes (Add, let n, println, expr) + 1 synthetic entry node.
        let snarl = to_snarl(&seed_graph());
        assert_eq!(snarl.nodes().count(), 5);
    }

    #[test]
    fn seed_has_four_wires_two_data_two_exec() {
        // Data: Add -> let n, println -> expr.
        // Exec: main -> let n, let n -> expr.
        let snarl = to_snarl(&seed_graph());
        assert_eq!(snarl.wires().count(), 4);
    }

    #[test]
    fn entry_node_is_named_for_the_function_with_one_exec_output() {
        let snarl = to_snarl(&seed_graph());
        let entry = snarl
            .nodes()
            .find(|n| n.title == "main")
            .expect("entry node present");
        assert!(entry.inputs.is_empty());
        assert_eq!(entry.outputs, vec![OutputRow::Exec]);
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
    fn value_node_add_is_pure_with_two_inline_inputs() {
        let snarl = to_snarl(&seed_graph());
        let add = snarl
            .nodes()
            .find(|n| n.title.contains("Add"))
            .expect("Add node present");
        // Pure: two inline data inputs, one data output, and no exec pins.
        assert_eq!(add.inputs.len(), 2);
        assert!(matches!(&add.inputs[0], InputRow::Inline { text } if text == "1"));
        assert!(matches!(&add.inputs[1], InputRow::Inline { text } if text == "2"));
        assert!(!add.inputs.iter().any(|r| matches!(r, InputRow::Exec)));
        assert_eq!(add.outputs, vec![OutputRow::Data]);
    }

    #[test]
    fn statement_node_let_has_exec_pins() {
        let snarl = to_snarl(&seed_graph());
        let let_n = snarl
            .nodes()
            .find(|n| n.title == "let n")
            .expect("let n node present");
        // Impure: exec-in at index 0, data input after it, exec-out on the right.
        assert!(matches!(let_n.inputs[0], InputRow::Exec));
        assert!(matches!(&let_n.inputs[1], InputRow::Wired { label } if label == "value"));
        assert_eq!(let_n.outputs, vec![OutputRow::Exec]);
    }
}

use std::collections::HashSet;

use vr_ir::{Literal, Type, VariantPayload};

use crate::model::{Block, Graph, GraphItem, Leaf, Node, NodeId, NodeKind};

impl Graph {
    /// Phase-2 well-formedness. Returns all problems found, or `Ok`.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Declared type names + item-name/main checks.
        let mut defined: HashSet<&str> = HashSet::new();
        let mut names: HashSet<String> = HashSet::new();
        let mut main_count = 0usize;
        let mut struct_fields: Vec<(&str, &Vec<(String, Type)>)> = Vec::new();
        for item in &self.items {
            let name = match item {
                GraphItem::Struct(s) => {
                    defined.insert(s.name.as_str());
                    struct_fields.push((s.name.as_str(), &s.fields));
                    s.name.clone()
                }
                GraphItem::Enum(e) => {
                    defined.insert(e.name.as_str());
                    e.name.clone()
                }
                GraphItem::Function(f) => {
                    if f.name == "main" {
                        main_count += 1;
                    }
                    f.name.clone()
                }
            };
            if !names.insert(name.clone()) {
                errors.push(format!("duplicate top-level item name: `{name}`"));
            }
        }
        if main_count == 0 {
            errors.push("no entry point: expected a function named `main`".into());
        } else if main_count > 1 {
            errors.push(format!("expected exactly one `main`, found {main_count}"));
        }

        // Named-type references across item signatures.
        for item in &self.items {
            match item {
                GraphItem::Struct(s) => {
                    for (_, t) in &s.fields {
                        check_named(t, &defined, &mut errors);
                    }
                }
                GraphItem::Enum(e) => {
                    for v in &e.variants {
                        match &v.payload {
                            VariantPayload::Unit => {}
                            VariantPayload::Tuple(tys) => {
                                for t in tys {
                                    check_named(t, &defined, &mut errors);
                                }
                            }
                            VariantPayload::Struct(fields) => {
                                for f in fields {
                                    check_named(&f.ty, &defined, &mut errors);
                                }
                            }
                        }
                    }
                }
                GraphItem::Function(f) => {
                    for (_, t) in &f.params {
                        check_named(t, &defined, &mut errors);
                    }
                    check_named(&f.ret, &defined, &mut errors);
                    check_block(&f.body, &struct_fields, &mut errors);
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

fn check_named(ty: &Type, defined: &HashSet<&str>, errors: &mut Vec<String>) {
    match ty {
        Type::Named(n) => {
            if !defined.contains(n.as_str()) {
                errors.push(format!("undefined type: `{n}`"));
            }
        }
        Type::Ref { inner, .. } | Type::Vec(inner) | Type::Option(inner) => {
            check_named(inner, defined, errors)
        }
        Type::Result(a, b) => {
            check_named(a, defined, errors);
            check_named(b, defined, errors);
        }
        Type::Tuple(items) => {
            for t in items {
                check_named(t, defined, errors);
            }
        }
        _ => {}
    }
}

/// Recursively check a block's nodes: required inputs satisfied, plus basic
/// StructLit field typing where a literal is statically known.
fn check_block(
    block: &Block,
    struct_fields: &[(&str, &Vec<(String, Type)>)],
    errors: &mut Vec<String>,
) {
    for (id, node) in &block.nodes {
        check_inputs(block, *id, node, errors);
        if let NodeKind::StructLit { name, fields } = &node.kind {
            if let Some((_, decl)) = struct_fields.iter().find(|(n, _)| n == name) {
                for (i, fname) in fields.iter().enumerate() {
                    if let Some((_, ty)) = decl.iter().find(|(dn, _)| dn == fname) {
                        if let Some(leaf) = inline_of(node, i as u16) {
                            if let Some(msg) = leaf_type_conflict(fname, leaf, ty) {
                                errors.push(msg);
                            }
                        }
                    }
                }
            }
        }
        // Recurse into nested scopes.
        match &node.kind {
            NodeKind::ForEach { body, .. } => check_block(body, struct_fields, errors),
            NodeKind::If { then, els } => {
                check_block(then, struct_fields, errors);
                if let Some(e) = els {
                    check_block(e, struct_fields, errors);
                }
            }
            NodeKind::Match { arms } => {
                for a in arms {
                    check_block(&a.body, struct_fields, errors);
                    if let Some(g) = &a.guard {
                        check_block(g, struct_fields, errors);
                    }
                }
            }
            _ => {}
        }
    }
}

fn inline_of(node: &Node, port: u16) -> Option<&Leaf> {
    node.inline.iter().find(|(p, _)| *p == port).map(|(_, l)| l)
}

fn required_ports(node: &Node) -> Vec<u16> {
    match &node.kind {
        NodeKind::Let { .. } => vec![0],
        NodeKind::Assign { .. } => vec![0, 1],
        NodeKind::ForEach { .. } => vec![0],
        NodeKind::ExprStmt => vec![0],
        NodeKind::Return { has_value } => {
            if *has_value {
                vec![0]
            } else {
                vec![]
            }
        }
        NodeKind::Field { .. } => vec![0],
        NodeKind::Binary { .. } => vec![0, 1],
        NodeKind::Ref { .. } => vec![0],
        NodeKind::Try => vec![0],
        NodeKind::Match { .. } => vec![0],
        NodeKind::If { .. } => vec![0],
        // variadic: at least their fixed prefix
        NodeKind::Call | NodeKind::Method { .. } => vec![0],
        NodeKind::Builtin { .. } => vec![],
        NodeKind::StructLit { fields, .. } => (0..fields.len() as u16).collect(),
        // leaf value nodes have no inputs
        NodeKind::PathValue(_) | NodeKind::VarValue(_) => vec![],
    }
}

fn check_inputs(block: &Block, id: NodeId, node: &Node, errors: &mut Vec<String>) {
    for port in required_ports(node) {
        let wired = block.data.iter().any(|e| e.to == id && e.to_port == port);
        let inlined = node.inline.iter().any(|(p, _)| *p == port);
        if !wired && !inlined {
            errors.push(format!("node {id:?} input port {port} is not connected"));
        }
    }
}

/// A statically-detectable literal/field-type conflict, or `None` if compatible
/// or unknowable.
fn leaf_type_conflict(field: &str, leaf: &Leaf, ty: &Type) -> Option<String> {
    let Leaf::Lit(l) = leaf else { return None };
    let ok = matches!(
        (l, ty),
        (Literal::Int(_), Type::I32 | Type::I64 | Type::Usize)
            | (Literal::Float(_), Type::F64)
            | (Literal::Bool(_), Type::Bool)
            | (Literal::Char(_), Type::Char)
            | (Literal::Str(_), Type::Str | Type::String)
            | (Literal::Unit, Type::Unit)
    );
    if ok {
        None
    } else {
        Some(format!(
            "field `{field}`: literal {l:?} is not compatible with {ty:?}"
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::build::*;
    use crate::model::*;
    use vr_ir::{Literal, Type};

    fn empty_main() -> FunctionGraph {
        FunctionGraph {
            name: "main".into(),
            params: vec![],
            ret: Type::Unit,
            body: BlockBuilder::new().build(),
        }
    }

    #[test]
    fn well_formed_graph_validates() {
        let g = Graph {
            items: vec![GraphItem::Function(empty_main())],
        };
        assert!(g.validate().is_ok());
    }

    #[test]
    fn missing_main_is_reported() {
        let g = Graph { items: vec![] };
        let errs = g.validate().unwrap_err();
        assert!(errs.iter().any(|e| e.contains("main")), "got: {errs:?}");
    }

    #[test]
    fn undefined_named_type_is_reported() {
        let g = Graph {
            items: vec![GraphItem::Function(FunctionGraph {
                name: "main".into(),
                params: vec![("r".into(), Type::Named("Ghost".into()))],
                ret: Type::Unit,
                body: BlockBuilder::new().build(),
            })],
        };
        let errs = g.validate().unwrap_err();
        assert!(errs.iter().any(|e| e.contains("Ghost")), "got: {errs:?}");
    }

    #[test]
    fn struct_field_type_mismatch_is_reported() {
        // Report { words: usize } but the graph feeds a bool literal.
        let mut b = BlockBuilder::new();
        let lit_node = struct_lit(&mut b, "Report", vec![("words", lit(Literal::Bool(true)))]);
        let let_r = b.stmt(NodeKind::Let {
            name: "r".into(),
            mutable: false,
        });
        b.feed(let_r, 0, lit_node);
        let g = Graph {
            items: vec![
                GraphItem::Struct(StructDecl {
                    name: "Report".into(),
                    fields: vec![("words".into(), Type::Usize)],
                }),
                GraphItem::Function(FunctionGraph {
                    name: "main".into(),
                    params: vec![],
                    ret: Type::Unit,
                    body: b.build(),
                }),
            ],
        };
        let errs = g.validate().unwrap_err();
        assert!(errs.iter().any(|e| e.contains("words")), "got: {errs:?}");
    }
}

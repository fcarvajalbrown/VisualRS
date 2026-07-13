use vr_ir::{AssignOp, BuiltinOp, Literal, Pattern, Type, VariantPayload};

use crate::build::*;
use crate::model::*;

/// The hardcoded "line report" program, expressed as a graph. Lowers to the same
/// `vr_ir::Program` shape as `vr_ir::fixtures::line_report`, producing
/// byte-identical Rust source (the parity oracle for the graph front-end).
pub fn line_report_graph() -> Graph {
    Graph {
        items: vec![
            report_struct(),
            line_kind_enum(),
            classify_fn(),
            build_report_fn(),
            run_fn(),
            main_fn(),
        ],
    }
}

/// A `LineKind::<variant>`-style path used directly as a block tail value.
fn path_value(b: &mut BlockBuilder, segs: &[&str]) -> NodeId {
    b.value(NodeKind::PathValue(
        segs.iter().map(|s| s.to_string()).collect(),
    ))
}

fn report_struct() -> GraphItem {
    let f = |n: &str| (n.to_string(), Type::Usize);
    GraphItem::Struct(StructDecl {
        name: "Report".into(),
        fields: vec![
            f("total_lines"),
            f("blank_lines"),
            f("comment_lines"),
            f("content_lines"),
            f("words"),
        ],
    })
}

fn line_kind_enum() -> GraphItem {
    let v = |n: &str| VariantDecl {
        name: n.into(),
        payload: VariantPayload::Unit,
    };
    GraphItem::Enum(EnumDecl {
        name: "LineKind".into(),
        variants: vec![v("Blank"), v("Comment"), v("Content")],
    })
}

// fn classify(line: &str) -> LineKind {
//   let trimmed = line.trim();
//   if trimmed.is_empty() { LineKind::Blank }
//   else if trimmed.starts_with('#') { LineKind::Comment }
//   else { LineKind::Content }
// }
fn classify_fn() -> GraphItem {
    let mut b = BlockBuilder::new();

    // let trimmed = line.trim();
    let trim = method(&mut b, var("line"), "trim", vec![]);
    let let_trimmed = b.stmt(NodeKind::Let {
        name: "trimmed".into(),
        mutable: false,
    });
    b.feed(let_trimmed, 0, trim);

    // innermost else: { LineKind::Content } -> Expr::Block(tail: Content)
    let content_else = {
        let mut t = BlockBuilder::new();
        let p = path_value(&mut t, &["LineKind", "Content"]);
        t.set_tail(p);
        t.build()
    };

    // inner if: if trimmed.starts_with('#') { Comment } else { <content else> }
    let inner_block = {
        let mut inner = BlockBuilder::new();
        let starts = method(
            &mut inner,
            var("trimmed"),
            "starts_with",
            vec![lit(Literal::Char('#'))],
        );
        let comment_then = {
            let mut t = BlockBuilder::new();
            let p = path_value(&mut t, &["LineKind", "Comment"]);
            t.set_tail(p);
            t.build()
        };
        let inner_if = inner.value(NodeKind::If {
            then: comment_then,
            els: Some(content_else),
        });
        inner.feed(inner_if, 0, starts);
        inner.set_tail(inner_if);
        inner.build()
    };

    // outer if: if trimmed.is_empty() { Blank } else { <inner if> }
    let empty = method(&mut b, var("trimmed"), "is_empty", vec![]);
    let blank_then = {
        let mut t = BlockBuilder::new();
        let p = path_value(&mut t, &["LineKind", "Blank"]);
        t.set_tail(p);
        t.build()
    };
    let outer_if = b.value(NodeKind::If {
        then: blank_then,
        els: Some(inner_block),
    });
    b.feed(outer_if, 0, empty);
    b.set_tail(outer_if);

    GraphItem::Function(FunctionGraph {
        name: "classify".into(),
        params: vec![("line".into(), Type::Str)],
        ret: Type::Named("LineKind".into()),
        body: b.build(),
    })
}

// fn build_report(text: &str) -> Report { ... }
fn build_report_fn() -> GraphItem {
    let mut b = BlockBuilder::new();

    let init = struct_lit(
        &mut b,
        "Report",
        vec![
            ("total_lines", int(0)),
            ("blank_lines", int(0)),
            ("comment_lines", int(0)),
            ("content_lines", int(0)),
            ("words", int(0)),
        ],
    );
    let let_report = b.stmt(NodeKind::Let {
        name: "report".into(),
        mutable: true,
    });
    b.feed(let_report, 0, init);

    // loop body: report.total_lines += 1; match classify(line) { ... }
    let mut body = BlockBuilder::new();
    bump(&mut body, "total_lines");
    let scrut = call(&mut body, var("classify"), vec![var("line")]);
    let arms = vec![
        arm_bump("Blank", "blank_lines"),
        arm_bump("Comment", "comment_lines"),
        content_arm(),
    ];
    let match_node = body.value(NodeKind::Match { arms });
    body.feed(match_node, 0, scrut);
    let es = body.stmt(NodeKind::ExprStmt);
    body.feed(es, 0, Src::Node(match_node));

    let lines_iter = method(&mut b, var("text"), "lines", vec![]);
    let for_stmt = b.stmt(NodeKind::ForEach {
        binding: "line".into(),
        body: body.build(),
    });
    b.feed(for_stmt, 0, lines_iter);

    let report_tail = b.value(NodeKind::VarValue("report".into()));
    b.set_tail(report_tail);

    GraphItem::Function(FunctionGraph {
        name: "build_report".into(),
        params: vec![("text".into(), Type::Str)],
        ret: Type::Named("Report".into()),
        body: b.build(),
    })
}

/// `report.<field> += 1;` appended as a statement to `b`.
fn bump(b: &mut BlockBuilder, field_name: &str) {
    let target = field(b, var("report"), field_name);
    let asg = b.stmt(NodeKind::Assign { op: AssignOp::Add });
    b.feed(asg, 0, target);
    b.feed(asg, 1, int(1));
}

/// A `LineKind::<variant> => { report.<field> += 1; }` arm.
fn arm_bump(variant: &str, field_name: &str) -> Arm {
    let mut body = BlockBuilder::new();
    bump(&mut body, field_name);
    Arm {
        pattern: Pattern::Path(vec!["LineKind".into(), variant.into()]),
        guard: None,
        body: body.build(),
    }
}

/// `LineKind::Content => { report.content_lines += 1; report.words += line.split_whitespace().count(); }`
fn content_arm() -> Arm {
    let mut body = BlockBuilder::new();
    bump(&mut body, "content_lines");
    let split = method(&mut body, var("line"), "split_whitespace", vec![]);
    let count = method(&mut body, split, "count", vec![]);
    let target = field(&mut body, var("report"), "words");
    let asg = body.stmt(NodeKind::Assign { op: AssignOp::Add });
    body.feed(asg, 0, target);
    body.feed(asg, 1, count);
    Arm {
        pattern: Pattern::Path(vec!["LineKind".into(), "Content".into()]),
        guard: None,
        body: body.build(),
    }
}

// fn run() -> Result<(), String> { ... }
fn run_fn() -> GraphItem {
    let mut b = BlockBuilder::new();

    // let path = match std::env::args().nth(1) {
    //   Some(p) => p,
    //   None => { return Err("usage: report <file>".to_string()); }
    // };
    let nth = builtin(&mut b, BuiltinOp::NthArg(1), vec![]);
    let some_arm = {
        let mut t = BlockBuilder::new();
        let p = t.value(NodeKind::VarValue("p".into()));
        t.set_tail(p);
        Arm {
            pattern: Pattern::TupleStruct {
                path: vec!["Some".into()],
                elems: vec![Pattern::Binding("p".into())],
            },
            guard: None,
            body: t.build(),
        }
    };
    let none_arm = {
        let mut nb = BlockBuilder::new();
        let msg = method(
            &mut nb,
            lit(Literal::Str("usage: report <file>".into())),
            "to_string",
            vec![],
        );
        let err = call(&mut nb, path(&["Err"]), vec![msg]);
        let ret = nb.stmt(NodeKind::Return { has_value: true });
        nb.feed(ret, 0, err);
        Arm {
            pattern: Pattern::Path(vec!["None".into()]),
            guard: None,
            body: nb.build(),
        }
    };
    let match_path = b.value(NodeKind::Match {
        arms: vec![some_arm, none_arm],
    });
    b.feed(match_path, 0, nth);
    let let_path = b.stmt(NodeKind::Let {
        name: "path".into(),
        mutable: false,
    });
    b.feed(let_path, 0, Src::Node(match_path));

    // let text = read_file(&path)?;
    let path_ref = reference(&mut b, false, var("path"));
    let read = builtin(&mut b, BuiltinOp::ReadFileToString, vec![path_ref]);
    let tried = try_(&mut b, read);
    let let_text = b.stmt(NodeKind::Let {
        name: "text".into(),
        mutable: false,
    });
    b.feed(let_text, 0, tried);

    // let report = build_report(&text);
    let text_ref = reference(&mut b, false, var("text"));
    let br = call(&mut b, var("build_report"), vec![text_ref]);
    let let_report = b.stmt(NodeKind::Let {
        name: "report".into(),
        mutable: false,
    });
    b.feed(let_report, 0, br);

    // println!("<label>: {}", report.<field>);
    print_line(&mut b, "lines", "total_lines");
    print_line(&mut b, "blank", "blank_lines");
    print_line(&mut b, "comment", "comment_lines");
    print_line(&mut b, "content", "content_lines");
    print_line(&mut b, "words", "words");

    // tail: Ok(())
    let ok = call(&mut b, path(&["Ok"]), vec![lit(Literal::Unit)]);
    b.set_tail_src(ok);

    GraphItem::Function(FunctionGraph {
        name: "run".into(),
        params: vec![],
        ret: Type::Result(Box::new(Type::Unit), Box::new(Type::String)),
        body: b.build(),
    })
}

/// `println!("<label>: {}", report.<field>);` appended as a statement.
fn print_line(b: &mut BlockBuilder, label: &str, field_name: &str) {
    let arg = field(b, var("report"), field_name);
    let call_node = builtin(b, BuiltinOp::PrintLine(format!("{label}: {{}}")), vec![arg]);
    let es = b.stmt(NodeKind::ExprStmt);
    b.feed(es, 0, call_node);
}

// fn main() { match run() { Ok(()) => {}, Err(e) => { eprintln!("error: {}", e); exit(1); } } }
fn main_fn() -> GraphItem {
    let mut b = BlockBuilder::new();
    let run_call = call(&mut b, var("run"), vec![]);
    let ok_arm = Arm {
        pattern: Pattern::TupleStruct {
            path: vec!["Ok".into()],
            elems: vec![Pattern::Tuple(vec![])],
        },
        guard: None,
        body: BlockBuilder::new().build(),
    };
    let err_arm = {
        let mut eb = BlockBuilder::new();
        let ep = builtin(
            &mut eb,
            BuiltinOp::EPrintLine("error: {}".into()),
            vec![var("e")],
        );
        let es = eb.stmt(NodeKind::ExprStmt);
        eb.feed(es, 0, ep);
        let exit = builtin(&mut eb, BuiltinOp::Exit, vec![int(1)]);
        let es2 = eb.stmt(NodeKind::ExprStmt);
        eb.feed(es2, 0, exit);
        Arm {
            pattern: Pattern::TupleStruct {
                path: vec!["Err".into()],
                elems: vec![Pattern::Binding("e".into())],
            },
            guard: None,
            body: eb.build(),
        }
    };
    let match_node = b.value(NodeKind::Match {
        arms: vec![ok_arm, err_arm],
    });
    b.feed(match_node, 0, run_call);
    let es = b.stmt(NodeKind::ExprStmt);
    b.feed(es, 0, Src::Node(match_node));

    GraphItem::Function(FunctionGraph {
        name: "main".into(),
        params: vec![],
        ret: Type::Unit,
        body: b.build(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn graph_capstone_matches_ir_capstone_source() {
        let from_graph = crate::lower(&line_report_graph()).expect("graph lowers");
        let graph_src = vr_rustgen::generate(&from_graph).expect("graph generates");
        let ir_src = vr_rustgen::generate(&vr_ir::fixtures::line_report()).expect("ir generates");
        assert_eq!(
            graph_src, ir_src,
            "graph-derived source must match the IR fixture"
        );
    }

    #[test]
    fn graph_capstone_validates() {
        line_report_graph()
            .validate()
            .expect("capstone graph must validate");
    }
}

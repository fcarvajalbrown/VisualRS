//! Hardcoded IR fixtures. In Phase 1 these stand in for what the editor's
//! graph will lower to in Phase 2. They are constructed directly at the IR
//! level (the "graph -> IR" front-end does not exist yet).

use crate::*;

// --- tiny expression helpers to keep the fixture readable -------------------
fn var(name: &str) -> Expr {
    Expr::Var(name.into())
}
fn field(base: Expr, name: &str) -> Expr {
    Expr::Field {
        base: Box::new(base),
        name: name.into(),
    }
}
fn path(segs: &[&str]) -> Expr {
    Expr::Path(segs.iter().map(|s| s.to_string()).collect())
}
fn int(v: i128) -> Expr {
    Expr::Lit(Literal::Int(v))
}
fn method(recv: Expr, m: &str, args: Vec<Expr>) -> Expr {
    Expr::MethodCall {
        receiver: Box::new(recv),
        method: m.into(),
        args,
    }
}

/// The hardcoded "line report" program: reads a file path from argv, counts
/// blank/comment/content lines and words, and prints a report. Std-only, so it
/// compiles on any channel with no external crates.
pub fn line_report() -> Program {
    Program {
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

fn report_struct() -> Item {
    let f = |n: &str| Field {
        name: n.into(),
        ty: Type::Usize,
    };
    Item::Struct(StructDef {
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

fn line_kind_enum() -> Item {
    let v = |n: &str| Variant {
        name: n.into(),
        payload: VariantPayload::Unit,
    };
    Item::Enum(EnumDef {
        name: "LineKind".into(),
        variants: vec![v("Blank"), v("Comment"), v("Content")],
    })
}

fn classify_fn() -> Item {
    // let trimmed = line.trim();
    let let_trimmed = Stmt::Let {
        name: "trimmed".into(),
        mutable: false,
        ty: None,
        value: method(var("line"), "trim", vec![]),
    };
    // else { LineKind::Content }
    let else_content = Expr::Block(Block::new(vec![], Some(path(&["LineKind", "Content"]))));
    // else if trimmed.starts_with('#') { Comment } else { Content }
    let inner_if = Expr::If {
        cond: Box::new(method(
            var("trimmed"),
            "starts_with",
            vec![Expr::Lit(Literal::Char('#'))],
        )),
        then: Block::new(vec![], Some(path(&["LineKind", "Comment"]))),
        else_: Some(Box::new(else_content)),
    };
    // if trimmed.is_empty() { Blank } else { inner_if }
    let outer_if = Expr::If {
        cond: Box::new(method(var("trimmed"), "is_empty", vec![])),
        then: Block::new(vec![], Some(path(&["LineKind", "Blank"]))),
        else_: Some(Box::new(inner_if)),
    };
    Item::Function(FunctionDef {
        name: "classify".into(),
        params: vec![Param {
            name: "line".into(),
            ty: Type::Str,
        }],
        ret: Type::Named("LineKind".into()),
        body: Block::new(vec![let_trimmed], Some(outer_if)),
    })
}

fn build_report_fn() -> Item {
    // let mut report = Report { total_lines: 0, ... };
    let init = Stmt::Let {
        name: "report".into(),
        mutable: true,
        ty: None,
        value: Expr::StructLit {
            name: "Report".into(),
            fields: vec![
                ("total_lines".into(), int(0)),
                ("blank_lines".into(), int(0)),
                ("comment_lines".into(), int(0)),
                ("content_lines".into(), int(0)),
                ("words".into(), int(0)),
            ],
        },
    };

    let bump = |f: &str| Stmt::Assign {
        target: field(var("report"), f),
        op: AssignOp::Add,
        value: int(1),
    };

    // Content arm: { report.content_lines += 1; report.words += line.split_whitespace().count(); }
    let content_body = Expr::Block(Block::new(
        vec![
            bump("content_lines"),
            Stmt::Assign {
                target: field(var("report"), "words"),
                op: AssignOp::Add,
                value: method(
                    method(var("line"), "split_whitespace", vec![]),
                    "count",
                    vec![],
                ),
            },
        ],
        None,
    ));

    let line_kind_arm = |variant: &str, body: Expr| MatchArm {
        pattern: Pattern::Path(vec!["LineKind".into(), variant.into()]),
        guard: None,
        body,
    };

    // match classify(line) { Blank => {...}, Comment => {...}, Content => {...} }
    let classify_match = Stmt::Expr(Expr::Match {
        scrutinee: Box::new(Expr::Call {
            func: Box::new(var("classify")),
            args: vec![var("line")],
        }),
        arms: vec![
            line_kind_arm(
                "Blank",
                Expr::Block(Block::new(vec![bump("blank_lines")], None)),
            ),
            line_kind_arm(
                "Comment",
                Expr::Block(Block::new(vec![bump("comment_lines")], None)),
            ),
            line_kind_arm("Content", content_body),
        ],
    });

    // for line in text.lines() { report.total_lines += 1; match ... }
    let for_loop = Stmt::ForEach {
        binding: "line".into(),
        iter: method(var("text"), "lines", vec![]),
        body: Block::new(vec![bump("total_lines"), classify_match], None),
    };

    Item::Function(FunctionDef {
        name: "build_report".into(),
        params: vec![Param {
            name: "text".into(),
            ty: Type::Str,
        }],
        ret: Type::Named("Report".into()),
        body: Block::new(vec![init, for_loop], Some(var("report"))),
    })
}

fn run_fn() -> Item {
    // let path = match std::env::args().nth(1) { Some(p) => p, None => return Err(...) };
    let path_let = Stmt::Let {
        name: "path".into(),
        mutable: false,
        ty: None,
        value: Expr::Match {
            scrutinee: Box::new(Expr::Builtin {
                op: BuiltinOp::NthArg(1),
                args: vec![],
            }),
            arms: vec![
                MatchArm {
                    pattern: Pattern::TupleStruct {
                        path: vec!["Some".into()],
                        elems: vec![Pattern::Binding("p".into())],
                    },
                    guard: None,
                    body: var("p"),
                },
                MatchArm {
                    pattern: Pattern::Path(vec!["None".into()]),
                    guard: None,
                    body: Expr::Block(Block::new(
                        vec![Stmt::Return(Some(Expr::Call {
                            func: Box::new(path(&["Err"])),
                            args: vec![method(
                                Expr::Lit(Literal::Str("usage: report <file>".into())),
                                "to_string",
                                vec![],
                            )],
                        }))],
                        None,
                    )),
                },
            ],
        },
    };

    // let text = <read file>?;
    let text_let = Stmt::Let {
        name: "text".into(),
        mutable: false,
        ty: None,
        value: Expr::Try(Box::new(Expr::Builtin {
            op: BuiltinOp::ReadFileToString,
            args: vec![Expr::Ref {
                mutable: false,
                expr: Box::new(var("path")),
            }],
        })),
    };

    // let report = build_report(&text);
    let report_let = Stmt::Let {
        name: "report".into(),
        mutable: false,
        ty: None,
        value: Expr::Call {
            func: Box::new(var("build_report")),
            args: vec![Expr::Ref {
                mutable: false,
                expr: Box::new(var("text")),
            }],
        },
    };

    let print = |label: &str, f: &str| {
        Stmt::Expr(Expr::Builtin {
            op: BuiltinOp::PrintLine(format!("{label}: {{}}")),
            args: vec![field(var("report"), f)],
        })
    };

    Item::Function(FunctionDef {
        name: "run".into(),
        params: vec![],
        ret: Type::Result(Box::new(Type::Unit), Box::new(Type::String)),
        body: Block::new(
            vec![
                path_let,
                text_let,
                report_let,
                print("lines", "total_lines"),
                print("blank", "blank_lines"),
                print("comment", "comment_lines"),
                print("content", "content_lines"),
                print("words", "words"),
            ],
            Some(Expr::Call {
                func: Box::new(path(&["Ok"])),
                args: vec![Expr::Lit(Literal::Unit)],
            }),
        ),
    })
}

fn main_fn() -> Item {
    // match run() { Ok(()) => {}, Err(e) => { eprintln!("error: {}", e); exit(1); } }
    let match_expr = Expr::Match {
        scrutinee: Box::new(Expr::Call {
            func: Box::new(var("run")),
            args: vec![],
        }),
        arms: vec![
            MatchArm {
                pattern: Pattern::TupleStruct {
                    path: vec!["Ok".into()],
                    elems: vec![Pattern::Tuple(vec![])],
                },
                guard: None,
                body: Expr::Block(Block::new(vec![], None)),
            },
            MatchArm {
                pattern: Pattern::TupleStruct {
                    path: vec!["Err".into()],
                    elems: vec![Pattern::Binding("e".into())],
                },
                guard: None,
                body: Expr::Block(Block::new(
                    vec![
                        Stmt::Expr(Expr::Builtin {
                            op: BuiltinOp::EPrintLine("error: {}".into()),
                            args: vec![var("e")],
                        }),
                        Stmt::Expr(Expr::Builtin {
                            op: BuiltinOp::Exit,
                            args: vec![int(1)],
                        }),
                    ],
                    None,
                )),
            },
        ],
    };
    Item::Function(FunctionDef {
        name: "main".into(),
        params: vec![],
        ret: Type::Unit,
        body: Block::new(vec![Stmt::Expr(match_expr)], None),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_report_is_valid_and_has_expected_items() {
        let prog = line_report();
        prog.validate().expect("fixture must validate");

        let names: Vec<&str> = prog
            .items
            .iter()
            .map(|i| match i {
                crate::Item::Struct(s) => s.name.as_str(),
                crate::Item::Enum(e) => e.name.as_str(),
                crate::Item::Function(f) => f.name.as_str(),
            })
            .collect();

        assert!(names.contains(&"Report"));
        assert!(names.contains(&"LineKind"));
        assert!(names.contains(&"classify"));
        assert!(names.contains(&"build_report"));
        assert!(names.contains(&"run"));
        assert!(names.contains(&"main"));
    }
}

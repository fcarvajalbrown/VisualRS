//! Visual Rust Rust backend: lowers `vr_ir` into Rust source via syn/quote.

mod expr;
mod item;
mod lit;
mod pat;
mod stmt;
mod ty;

use std::fmt;

/// Errors from the Rust backend.
#[derive(Debug)]
pub enum GenError {
    /// The IR failed `Program::validate`.
    Validation(Vec<String>),
    /// The emitted tokens did not parse as a `syn::File` (a codegen bug).
    Parse(String),
}

impl fmt::Display for GenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GenError::Validation(errs) => {
                writeln!(f, "IR validation failed:")?;
                for e in errs {
                    writeln!(f, "  - {e}")?;
                }
                Ok(())
            }
            GenError::Parse(msg) => write!(f, "generated tokens did not parse: {msg}"),
        }
    }
}

impl std::error::Error for GenError {}

/// Lower a whole program to formatted, readable Rust source.
///
/// Pipeline: validate -> emit `TokenStream` -> parse as `syn::File` (validity
/// gate) -> format with `prettyplease`.
pub fn generate(program: &vr_ir::Program) -> Result<String, GenError> {
    program.validate().map_err(GenError::Validation)?;

    let tokens = emit_program(program);
    let file: syn::File = syn::parse2(tokens).map_err(|e| GenError::Parse(e.to_string()))?;
    Ok(prettyplease::unparse(&file))
}

/// Emit the token stream for the whole file: each top-level item in order.
fn emit_program(program: &vr_ir::Program) -> proc_macro2::TokenStream {
    use quote::quote;
    let items = program.items.iter().map(item::gen_item);
    quote! {
        #(#items)*
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vr_ir::*;

    #[test]
    fn generates_an_empty_main() {
        let prog = Program {
            items: vec![Item::Function(FunctionDef {
                name: "main".into(),
                params: vec![],
                ret: Type::Unit,
                body: Block::new(vec![], None),
            })],
        };
        let src = generate(&prog).expect("should generate");
        assert!(src.contains("fn main()"), "got:\n{src}");
    }

    #[test]
    fn invalid_program_returns_validation_error() {
        let prog = Program { items: vec![] }; // no main
        let err = generate(&prog).unwrap_err();
        assert!(matches!(err, GenError::Validation(_)));
    }

    #[test]
    fn generates_struct_and_enum() {
        use vr_ir::*;
        let prog = Program {
            items: vec![
                Item::Struct(StructDef {
                    name: "Report".into(),
                    fields: vec![Field {
                        name: "words".into(),
                        ty: Type::Usize,
                    }],
                }),
                Item::Enum(EnumDef {
                    name: "LineKind".into(),
                    variants: vec![
                        Variant {
                            name: "Blank".into(),
                            payload: VariantPayload::Unit,
                        },
                        Variant {
                            name: "Content".into(),
                            payload: VariantPayload::Unit,
                        },
                    ],
                }),
                Item::Function(FunctionDef {
                    name: "main".into(),
                    params: vec![],
                    ret: Type::Unit,
                    body: Block::new(vec![], None),
                }),
            ],
        };
        let src = generate(&prog).unwrap();
        assert!(src.contains("struct Report"), "got:\n{src}");
        assert!(src.contains("words: usize"), "got:\n{src}");
        assert!(src.contains("enum LineKind"), "got:\n{src}");
        assert!(src.contains("Blank"), "got:\n{src}");
    }

    #[test]
    fn generates_match_try_and_builtins() {
        use vr_ir::*;
        // fn main() {
        //   let n = 1 + 2;
        //   println!("n: {}", n);
        // }
        let prog = Program {
            items: vec![Item::Function(FunctionDef {
                name: "main".into(),
                params: vec![],
                ret: Type::Unit,
                body: Block::new(
                    vec![
                        Stmt::Let {
                            name: "n".into(),
                            mutable: false,
                            ty: None,
                            value: Expr::Binary {
                                op: BinaryOp::Add,
                                lhs: Box::new(Expr::Lit(Literal::Int(1))),
                                rhs: Box::new(Expr::Lit(Literal::Int(2))),
                            },
                        },
                        Stmt::Expr(Expr::Builtin {
                            op: BuiltinOp::PrintLine("n: {}".into()),
                            args: vec![Expr::Var("n".into())],
                        }),
                    ],
                    None,
                ),
            })],
        };
        let src = generate(&prog).unwrap();
        // The backend always parenthesizes binary operands (precedence-safe),
        // so the expected form is `(1 + 2)`, not `1 + 2`.
        assert!(src.contains("let n = (1 + 2)"), "got:\n{src}");
        assert!(src.contains(r#"println!("n: {}", n)"#), "got:\n{src}");
    }

    #[test]
    fn capstone_generates_readable_rust() {
        let prog = vr_ir::fixtures::line_report();
        let src = generate(&prog).expect("capstone must generate");

        // Structural spot-checks across the whole MVP subset:
        assert!(src.contains("struct Report"), "got:\n{src}");
        assert!(src.contains("enum LineKind"), "got:\n{src}");
        assert!(
            src.contains("fn classify(line: &str) -> LineKind"),
            "got:\n{src}"
        );
        assert!(
            src.contains("fn build_report(text: &str) -> Report"),
            "got:\n{src}"
        );
        assert!(
            src.contains("fn run() -> Result<(), String>"),
            "got:\n{src}"
        );
        assert!(src.contains("for line in text.lines()"), "got:\n{src}");
        assert!(src.contains("match"), "got:\n{src}");
        assert!(src.contains("std::fs::read_to_string"), "got:\n{src}");
        assert!(src.contains("std::process::exit(1)"), "got:\n{src}");
        assert!(src.contains("fn main()"), "got:\n{src}");
    }
}

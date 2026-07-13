//! Visual Rust Rust backend: lowers `vr_ir` into Rust source via syn/quote.

mod lit;
mod ty;
// mod pat;   // Task 10
// mod expr;  // Task 10
// mod stmt;  // Task 10
// mod item;  // Task 9

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

/// Emit the token stream for the whole file. Fully implemented in Task 9 once
/// item generation exists; the Task 8 skeleton handles only an empty `main`.
fn emit_program(program: &vr_ir::Program) -> proc_macro2::TokenStream {
    use quote::quote;
    // Placeholder until Task 9 wires real item generation:
    let _ = program;
    quote! { fn main() {} }
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
}

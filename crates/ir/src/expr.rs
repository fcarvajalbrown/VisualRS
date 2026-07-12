use crate::lit::Literal;
use crate::ops::BinaryOp;
use crate::pat::Pattern;
use crate::stmt::Block;

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Lit(Literal),
    Var(String),
    /// A multi-segment path, e.g. `LineKind::Blank` or `Ok`.
    Path(Vec<String>),
    Field {
        base: Box<Expr>,
        name: String,
    },
    Call {
        func: Box<Expr>,
        args: Vec<Expr>,
    },
    MethodCall {
        receiver: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },
    Binary {
        op: BinaryOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Ref {
        mutable: bool,
        expr: Box<Expr>,
    },
    StructLit {
        name: String,
        fields: Vec<(String, Expr)>,
    },
    Match {
        scrutinee: Box<Expr>,
        arms: Vec<MatchArm>,
    },
    If {
        cond: Box<Expr>,
        then: Block,
        else_: Option<Box<Expr>>,
    },
    Block(Block),
    /// The `?` operator applied to a fallible expression.
    Try(Box<Expr>),
    /// An abstract, well-known operation. The backend chooses concrete code;
    /// the IR stays free of std-library specifics (ADR-0005).
    Builtin {
        op: BuiltinOp,
        args: Vec<Expr>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: Expr,
}

/// Well-known operations the Rust backend maps to concrete code:
/// - `PrintLine(tmpl)`  -> `println!(tmpl, args...)`
/// - `EPrintLine(tmpl)` -> `eprintln!(tmpl, args...)`
/// - `ReadFileToString` -> `std::fs::read_to_string(arg0).map_err(|e| e.to_string())` : Result<String, String>
/// - `NthArg(n)`        -> `std::env::args().nth(n)` : Option<String>
/// - `Exit`             -> `std::process::exit(arg0)`
#[derive(Clone, Debug, PartialEq)]
pub enum BuiltinOp {
    PrintLine(String),
    EPrintLine(String),
    ReadFileToString,
    NthArg(usize),
    Exit,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lit::Literal;
    use crate::ops::BinaryOp;

    #[test]
    fn method_call_chains_are_expressions() {
        let e = Expr::MethodCall {
            receiver: Box::new(Expr::Var("line".into())),
            method: "trim".into(),
            args: vec![],
        };
        match e {
            Expr::MethodCall { method, .. } => assert_eq!(method, "trim"),
            _ => panic!("expected method call"),
        }
    }

    #[test]
    fn try_wraps_a_fallible_expr() {
        let e = Expr::Try(Box::new(Expr::Builtin {
            op: BuiltinOp::ReadFileToString,
            args: vec![Expr::Var("path".into())],
        }));
        assert!(matches!(e, Expr::Try(_)));
    }

    #[test]
    fn binary_greater_than() {
        let e = Expr::Binary {
            op: BinaryOp::Gt,
            lhs: Box::new(Expr::Lit(Literal::Int(1))),
            rhs: Box::new(Expr::Lit(Literal::Int(0))),
        };
        assert!(matches!(
            e,
            Expr::Binary {
                op: BinaryOp::Gt,
                ..
            }
        ));
    }
}

/// A target-agnostic type reference. The Rust backend maps these to concrete
/// Rust types; other backends could map them differently (ADR-0005).
#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    Unit,
    Bool,
    Char,
    I32,
    I64,
    Usize,
    F64,
    /// Borrowed text (`&str` in the Rust backend).
    Str,
    /// Owned text (`String` in the Rust backend).
    String,
    /// A user-defined struct or enum, by name.
    Named(String),
    Ref {
        mutable: bool,
        inner: Box<Type>,
    },
    Vec(Box<Type>),
    Option(Box<Type>),
    Result(Box<Type>, Box<Type>),
    Tuple(Vec<Type>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn result_of_string_and_named_is_constructible() {
        let ty = Type::Result(Box::new(Type::Unit), Box::new(Type::String));
        match ty {
            Type::Result(ok, err) => {
                assert_eq!(*ok, Type::Unit);
                assert_eq!(*err, Type::String);
            }
            _ => panic!("expected Result"),
        }
    }

    #[test]
    fn ref_carries_mutability() {
        let ty = Type::Ref {
            mutable: true,
            inner: Box::new(Type::Str),
        };
        assert_eq!(
            ty,
            Type::Ref {
                mutable: true,
                inner: Box::new(Type::Str),
            }
        );
    }
}

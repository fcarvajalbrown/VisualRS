use crate::lit::Literal;

/// A match/binding pattern. Covers the MVP subset: wildcards, bindings,
/// unit-variant paths, data-carrying variants (`Some(x)`, `Ok(v)`), tuples
/// (including `()`), and literals.
#[derive(Clone, Debug, PartialEq)]
pub enum Pattern {
    Wildcard,
    Binding(String),
    /// A path with no payload, e.g. `LineKind::Blank`.
    Path(Vec<String>),
    /// A path with tuple payload, e.g. `Some(x)`, `Ok(v)`, `Err(e)`.
    TupleStruct {
        path: Vec<String>,
        elems: Vec<Pattern>,
    },
    Tuple(Vec<Pattern>),
    Literal(Literal),
}

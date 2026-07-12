/// A literal value. Target-agnostic: no Rust suffixes or types leak in here.
#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    Int(i128),
    Float(f64),
    Bool(bool),
    Char(char),
    Str(String),
    Unit,
}

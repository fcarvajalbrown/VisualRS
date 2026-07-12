use crate::stmt::Block;
use crate::ty::Type;

/// A whole program: a flat list of top-level items. A function named `main`
/// (returning `Unit`) is the entry point in the Rust backend.
#[derive(Clone, Debug, PartialEq)]
pub struct Program {
    pub items: Vec<Item>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Item {
    Struct(StructDef),
    Enum(EnumDef),
    Function(FunctionDef),
}

#[derive(Clone, Debug, PartialEq)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Field {
    pub name: String,
    pub ty: Type,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EnumDef {
    pub name: String,
    pub variants: Vec<Variant>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Variant {
    pub name: String,
    pub payload: VariantPayload,
}

#[derive(Clone, Debug, PartialEq)]
pub enum VariantPayload {
    Unit,
    Tuple(Vec<Type>),
    Struct(Vec<Field>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionDef {
    pub name: String,
    pub params: Vec<Param>,
    pub ret: Type,
    pub body: Block,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Param {
    pub name: String,
    pub ty: Type,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ty::Type;

    #[test]
    fn struct_with_named_fields() {
        let s = StructDef {
            name: "Report".into(),
            fields: vec![
                Field {
                    name: "total_lines".into(),
                    ty: Type::Usize,
                },
                Field {
                    name: "words".into(),
                    ty: Type::Usize,
                },
            ],
        };
        assert_eq!(s.fields.len(), 2);
        assert_eq!(s.fields[0].name, "total_lines");
    }

    #[test]
    fn enum_with_unit_variants() {
        let e = EnumDef {
            name: "LineKind".into(),
            variants: vec![
                Variant {
                    name: "Blank".into(),
                    payload: VariantPayload::Unit,
                },
                Variant {
                    name: "Comment".into(),
                    payload: VariantPayload::Unit,
                },
                Variant {
                    name: "Content".into(),
                    payload: VariantPayload::Unit,
                },
            ],
        };
        assert_eq!(e.variants.len(), 3);
        assert!(matches!(e.variants[0].payload, VariantPayload::Unit));
    }
}

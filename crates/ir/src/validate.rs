use crate::item::{Item, Program, VariantPayload};
use crate::ty::Type;
use std::collections::HashSet;

impl Program {
    /// Basic well-formedness for Phase 1. Returns all problems found, or `Ok`.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        let mut defined_types: HashSet<&str> = HashSet::new();
        let mut item_names: HashSet<String> = HashSet::new();
        let mut main_count = 0usize;

        for item in &self.items {
            let name = match item {
                Item::Struct(s) => {
                    defined_types.insert(s.name.as_str());
                    s.name.clone()
                }
                Item::Enum(e) => {
                    defined_types.insert(e.name.as_str());
                    e.name.clone()
                }
                Item::Function(f) => {
                    if f.name == "main" {
                        main_count += 1;
                    }
                    f.name.clone()
                }
            };
            if !item_names.insert(name.clone()) {
                errors.push(format!("duplicate top-level item name: `{name}`"));
            }
        }

        if main_count == 0 {
            errors.push("no entry point: expected a function named `main`".into());
        } else if main_count > 1 {
            errors.push(format!("expected exactly one `main`, found {main_count}"));
        }

        // Collect every Type mentioned anywhere in an item's signature surface.
        for item in &self.items {
            match item {
                Item::Struct(s) => {
                    for f in &s.fields {
                        check_named(&f.ty, &defined_types, &mut errors);
                    }
                }
                Item::Enum(e) => {
                    for v in &e.variants {
                        match &v.payload {
                            VariantPayload::Unit => {}
                            VariantPayload::Tuple(tys) => {
                                for t in tys {
                                    check_named(t, &defined_types, &mut errors);
                                }
                            }
                            VariantPayload::Struct(fields) => {
                                for f in fields {
                                    check_named(&f.ty, &defined_types, &mut errors);
                                }
                            }
                        }
                    }
                }
                Item::Function(f) => {
                    for p in &f.params {
                        check_named(&p.ty, &defined_types, &mut errors);
                    }
                    check_named(&f.ret, &defined_types, &mut errors);
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

/// Walk a type and report any `Named` that is not defined in the program.
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

#[cfg(test)]
mod tests {
    use crate::*;

    fn empty_main() -> FunctionDef {
        FunctionDef {
            name: "main".into(),
            params: vec![],
            ret: Type::Unit,
            body: Block::new(vec![], None),
        }
    }

    #[test]
    fn undefined_named_type_is_reported() {
        let prog = Program {
            items: vec![Item::Function(FunctionDef {
                name: "main".into(),
                params: vec![Param {
                    name: "r".into(),
                    ty: Type::Named("Ghost".into()),
                }],
                ret: Type::Unit,
                body: Block::new(vec![], None),
            })],
        };
        let errs = prog.validate().unwrap_err();
        assert!(errs.iter().any(|e| e.contains("Ghost")), "got: {errs:?}");
    }

    #[test]
    fn missing_main_is_reported() {
        let prog = Program { items: vec![] };
        let errs = prog.validate().unwrap_err();
        assert!(errs.iter().any(|e| e.contains("main")), "got: {errs:?}");
    }

    #[test]
    fn well_formed_program_validates() {
        let prog = Program {
            items: vec![Item::Function(empty_main())],
        };
        assert!(prog.validate().is_ok());
    }
}

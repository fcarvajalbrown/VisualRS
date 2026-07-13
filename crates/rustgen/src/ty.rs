use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use vr_ir::Type;

pub(crate) fn gen_type(ty: &Type) -> TokenStream {
    match ty {
        Type::Unit => quote!(()),
        Type::Bool => quote!(bool),
        Type::Char => quote!(char),
        Type::I32 => quote!(i32),
        Type::I64 => quote!(i64),
        Type::Usize => quote!(usize),
        Type::F64 => quote!(f64),
        Type::Str => quote!(&str),
        Type::String => quote!(String),
        Type::Named(n) => {
            let id = format_ident!("{}", n);
            quote!(#id)
        }
        Type::Ref { mutable, inner } => {
            let inner = gen_type(inner);
            if *mutable {
                quote!(&mut #inner)
            } else {
                quote!(&#inner)
            }
        }
        Type::Vec(inner) => {
            let inner = gen_type(inner);
            quote!(Vec<#inner>)
        }
        Type::Option(inner) => {
            let inner = gen_type(inner);
            quote!(Option<#inner>)
        }
        Type::Result(ok, err) => {
            let ok = gen_type(ok);
            let err = gen_type(err);
            quote!(Result<#ok, #err>)
        }
        Type::Tuple(items) => {
            let items = items.iter().map(gen_type);
            quote!((#(#items),*))
        }
    }
}

/// A `::`-joined path such as `LineKind::Blank` or `std::process::exit`.
pub(crate) fn gen_path(segs: &[String]) -> TokenStream {
    let idents = segs.iter().map(|s| format_ident!("{}", s));
    quote!(#(#idents)::*)
}

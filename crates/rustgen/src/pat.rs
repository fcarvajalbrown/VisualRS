use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use vr_ir::Pattern;

use crate::lit::gen_literal;
use crate::ty::gen_path;

pub(crate) fn gen_pattern(pat: &Pattern) -> TokenStream {
    match pat {
        Pattern::Wildcard => quote!(_),
        Pattern::Binding(name) => {
            let id = format_ident!("{}", name);
            quote!(#id)
        }
        Pattern::Path(segs) => gen_path(segs),
        Pattern::TupleStruct { path, elems } => {
            let path = gen_path(path);
            let elems = elems.iter().map(gen_pattern);
            quote!(#path(#(#elems),*))
        }
        Pattern::Tuple(elems) => {
            let elems = elems.iter().map(gen_pattern);
            quote!((#(#elems),*))
        }
        Pattern::Literal(lit) => gen_literal(lit),
    }
}

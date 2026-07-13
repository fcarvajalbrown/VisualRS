use proc_macro2::TokenStream;
use quote::quote;
use vr_ir::Literal;

pub(crate) fn gen_literal(lit: &Literal) -> TokenStream {
    match lit {
        Literal::Int(v) => {
            let l = proc_macro2::Literal::i128_unsuffixed(*v);
            quote!(#l)
        }
        Literal::Float(v) => {
            let l = proc_macro2::Literal::f64_unsuffixed(*v);
            quote!(#l)
        }
        Literal::Bool(b) => quote!(#b),
        Literal::Char(c) => quote!(#c),
        Literal::Str(s) => quote!(#s),
        Literal::Unit => quote!(()),
    }
}

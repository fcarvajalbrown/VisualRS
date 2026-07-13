use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use vr_ir::{EnumDef, Field, FunctionDef, Item, StructDef, VariantPayload};

use crate::stmt::gen_block;
use crate::ty::gen_type;

pub(crate) fn gen_item(item: &Item) -> TokenStream {
    match item {
        Item::Struct(s) => gen_struct(s),
        Item::Enum(e) => gen_enum(e),
        Item::Function(f) => gen_fn(f),
    }
}

fn gen_field(field: &Field) -> TokenStream {
    let name = format_ident!("{}", field.name);
    let ty = gen_type(&field.ty);
    quote!(#name: #ty)
}

fn gen_struct(s: &StructDef) -> TokenStream {
    let name = format_ident!("{}", s.name);
    let fields = s.fields.iter().map(gen_field);
    quote! {
        struct #name {
            #(#fields),*
        }
    }
}

fn gen_enum(e: &EnumDef) -> TokenStream {
    let name = format_ident!("{}", e.name);
    let variants = e.variants.iter().map(|v| {
        let vname = format_ident!("{}", v.name);
        match &v.payload {
            VariantPayload::Unit => quote!(#vname),
            VariantPayload::Tuple(tys) => {
                let tys = tys.iter().map(gen_type);
                quote!(#vname(#(#tys),*))
            }
            VariantPayload::Struct(fields) => {
                let fields = fields.iter().map(gen_field);
                quote!(#vname { #(#fields),* })
            }
        }
    });
    quote! {
        enum #name {
            #(#variants),*
        }
    }
}

fn gen_fn(f: &FunctionDef) -> TokenStream {
    let name = format_ident!("{}", f.name);
    let params = f.params.iter().map(|p| {
        let pname = format_ident!("{}", p.name);
        let ty = gen_type(&p.ty);
        quote!(#pname: #ty)
    });
    let body = gen_block(&f.body);

    // `fn main()` has no return-type arrow when it returns unit; same for any
    // unit-returning fn -> omit `-> ()` for readable output.
    if matches!(f.ret, vr_ir::Type::Unit) {
        quote! {
            fn #name(#(#params),*) #body
        }
    } else {
        let ret = gen_type(&f.ret);
        quote! {
            fn #name(#(#params),*) -> #ret #body
        }
    }
}

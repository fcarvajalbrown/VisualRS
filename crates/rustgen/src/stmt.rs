use proc_macro2::TokenStream;
use quote::quote;
use vr_ir::Block;

// Temporary stub so item/function generation compiles in Task 9. Replaced with
// the real statement/block lowering in Task 10.
pub(crate) fn gen_block(block: &Block) -> TokenStream {
    let _ = block;
    quote!({})
}

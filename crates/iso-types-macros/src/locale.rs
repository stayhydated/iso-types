//! Configured locale expansion: validate the model, then emit Rust.

use proc_macro2::TokenStream;

mod emit;
mod model;

pub(crate) fn expand(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let model = model::Model::parse(attr, item)?;
    Ok(es_fluent_derive_core::macro_support::with_build_inputs(
        emit::emit(&model),
    ))
}

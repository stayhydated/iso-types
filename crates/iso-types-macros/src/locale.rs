//! Configured locale expansion: validate the model, then emit Rust.

use proc_macro2::TokenStream;

mod emit;
mod model;
mod scope;

pub(crate) fn expand(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let model = model::Model::parse(attr, item)?;
    Ok(emit::emit(&model))
}

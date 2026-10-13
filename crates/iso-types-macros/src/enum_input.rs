use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemEnum, Path, Token, parse_quote, punctuated::Punctuated};

pub(crate) fn facade() -> TokenStream {
    use proc_macro_crate::{FoundCrate, crate_name};
    match crate_name("iso-types") {
        Ok(FoundCrate::Itself) => quote!(::iso_types),
        Ok(FoundCrate::Name(name)) => {
            let ident = format_ident!("{name}");
            quote!(::#ident)
        }
        Err(_) => quote!(::iso_types),
    }
}

pub(crate) fn prepare(input: &mut ItemEnum) -> syn::Result<()> {
    if !input.variants.is_empty()
        || !input.generics.params.is_empty()
        || input.generics.where_clause.is_some()
    {
        return Err(syn::Error::new_spanned(
            input,
            "expected an empty enum without generics",
        ));
    }
    let mut seen = std::collections::HashSet::new();
    for attr in &input.attrs {
        if attr.path().is_ident("derive") {
            for path in attr.parse_args_with(Punctuated::<Path, Token![,]>::parse_terminated)? {
                if let Some(segment) = path.segments.last() {
                    seen.insert(segment.ident.to_string());
                }
            }
        }
    }
    let derives: Vec<Path> = [
        ("Clone", parse_quote!(::core::clone::Clone)),
        ("Copy", parse_quote!(::core::marker::Copy)),
        ("Debug", parse_quote!(::core::fmt::Debug)),
        ("Eq", parse_quote!(::core::cmp::Eq)),
        ("Hash", parse_quote!(::core::hash::Hash)),
        ("PartialEq", parse_quote!(::core::cmp::PartialEq)),
    ]
    .into_iter()
    .filter(|(name, _)| !seen.contains(*name))
    .map(|(_, path)| path)
    .collect();
    if !derives.is_empty() {
        input.attrs.push(parse_quote!(#[derive(#(#derives),*)]));
    }
    Ok(())
}

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemEnum, LitStr, Meta, Path, Token, parse_quote, punctuated::Punctuated};

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
    // Helper attributes follow the derives that register them with the compiler.
    let mut strum_attrs = Vec::new();
    input.attrs.retain(|attr| {
        if attr.path().is_ident("strum") {
            strum_attrs.push(attr.clone());
            false
        } else {
            true
        }
    });
    let root = facade();
    let derives: Vec<Path> = [
        ("Clone", parse_quote!(::core::clone::Clone)),
        ("Copy", parse_quote!(::core::marker::Copy)),
        ("Debug", parse_quote!(::core::fmt::Debug)),
        ("Eq", parse_quote!(::core::cmp::Eq)),
        ("Hash", parse_quote!(::core::hash::Hash)),
        ("PartialEq", parse_quote!(::core::cmp::PartialEq)),
        ("VariantArray", parse_quote!(#root::strum::VariantArray)),
    ]
    .into_iter()
    .filter(|(name, _)| !seen.contains(*name))
    .map(|(_, path)| path)
    .collect();
    if !derives.is_empty() {
        input.attrs.push(parse_quote!(#[derive(#(#derives),*)]));
    }
    let mut has_strum_path = false;
    for attr in &strum_attrs {
        let options = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
        has_strum_path |= options.iter().any(|option| option.path().is_ident("crate"));
    }
    input.attrs.extend(strum_attrs);
    if !has_strum_path {
        let path = LitStr::new(&quote!(#root::strum).to_string(), input.ident.span());
        input.attrs.push(parse_quote!(#[strum(crate = #path)]));
    }
    Ok(())
}

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, ItemEnum, LitStr, Meta, Path, Token, parse_quote, punctuated::Punctuated};

pub(crate) fn set_variants(input: &mut ItemEnum, variants: &[Ident], codes: &[LitStr]) {
    input.variants = variants
        .iter()
        .zip(codes)
        .map(|(variant, code)| -> syn::Variant {
            parse_quote!(#[strum(to_string = #code)] #variant)
        })
        .collect();
    input.attrs.push(parse_quote!(
        #[doc = "`from_repr()` accepts a zero-based declaration ordinal. Use parsing for ISO codes and locale tags."]
    ));
}

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
        ("EnumIter", parse_quote!(#root::strum::EnumIter)),
        ("EnumCount", parse_quote!(#root::strum::EnumCount)),
        ("FromRepr", parse_quote!(#root::strum::FromRepr)),
        ("IntoStaticStr", parse_quote!(#root::strum::IntoStaticStr)),
    ]
    .into_iter()
    .filter(|(name, _)| !seen.contains(*name))
    .map(|(_, path)| path)
    .collect();
    if !derives.is_empty() {
        input.attrs.push(parse_quote!(#[derive(#(#derives),*)]));
    }
    let mut has_strum_path = false;
    let mut has_const_into_str = false;
    for attr in &strum_attrs {
        let options = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
        for option in &options {
            if option.path().is_ident("prefix") || option.path().is_ident("suffix") {
                return Err(syn::Error::new_spanned(
                    option,
                    "ISO macros own canonical codes; remove Strum prefix or suffix",
                ));
            }
        }
        has_strum_path |= options.iter().any(|option| option.path().is_ident("crate"));
        has_const_into_str |= options
            .iter()
            .any(|option| option.path().is_ident("const_into_str"));
    }
    input.attrs.extend(strum_attrs);
    if !has_strum_path {
        let path = LitStr::new(&quote!(#root::strum).to_string(), input.ident.span());
        input.attrs.push(parse_quote!(#[strum(crate = #path)]));
    }
    if !has_const_into_str {
        input.attrs.push(parse_quote!(#[strum(const_into_str)]));
    }
    Ok(())
}

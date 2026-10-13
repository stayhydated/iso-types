//! Parse declarations and validate selections before emitting Rust.

use std::collections::HashSet;

use proc_macro2::{Span, TokenStream};
use syn::{
    Ident, ItemEnum, LitStr, Token, parse::Parser as _, parse_quote, punctuated::Punctuated,
};

use crate::enum_input::{prepare, set_variants};

use super::data;

pub(super) struct Model {
    pub input: ItemEnum,
    pub standard: LitStr,
    pub type_name: Ident,
    pub variants: Vec<Ident>,
    pub complete: bool,
}

impl Model {
    pub fn parse(attr: TokenStream, item: TokenStream) -> syn::Result<Self> {
        let mut standard = None;
        let mut codes = None;
        let mut country = None;
        let mut category = None;
        syn::meta::parser(|meta| {
            if meta.path.is_ident("standard") && standard.is_none() {
                standard = Some(meta.value()?.parse::<LitStr>()?);
            } else if meta.path.is_ident("codes") && codes.is_none() {
                let content;
                let value = meta.value()?;
                syn::bracketed!(content in value);
                codes = Some(Punctuated::<LitStr, Token![,]>::parse_terminated(&content)?);
            } else if meta.path.is_ident("country") && country.is_none() {
                country = Some(meta.value()?.parse::<LitStr>()?);
            } else if meta.path.is_ident("category") && category.is_none() {
                category = Some(meta.value()?.parse::<LitStr>()?);
            } else {
                return Err(meta.error("expected standard, codes, country, or category once each"));
            }
            Ok(())
        })
        .parse2(attr)?;
        let standard =
            standard.ok_or_else(|| syn::Error::new(Span::call_site(), "standard is required"))?;
        let catalog = data::catalog(&standard)?;
        for (filter, field) in [
            (country.as_ref(), "country"),
            (category.as_ref(), "category"),
        ] {
            if let Some(filter) = filter {
                let value = filter.value();
                let valid = catalog.entries.iter().any(|entry| {
                    (if field == "country" {
                        &entry.country
                    } else {
                        &entry.category
                    })
                    .as_deref()
                        == Some(value.as_str())
                });
                if !valid {
                    return Err(syn::Error::new_spanned(
                        filter,
                        format!(
                            "unavailable {field} filter `{value}` in the {} catalog",
                            standard.value()
                        ),
                    ));
                }
            }
        }
        let mut selected = if let Some(codes) = codes {
            if codes.is_empty() {
                return Err(syn::Error::new_spanned(
                    &standard,
                    "select at least one code",
                ));
            }
            let mut seen = HashSet::new();
            let mut selected = Vec::with_capacity(codes.len());
            for code in codes {
                let value = code.value();
                let index = catalog.entries.binary_search_by(|entry| entry.code.cmp(&value))
                    .map_err(|_| syn::Error::new_spanned(&code, format!(
                        "expected a canonical code in the current {} catalog; `{value}` is unavailable",
                        standard.value(),
                    )))?;
                if !seen.insert(value) {
                    return Err(syn::Error::new_spanned(code, "duplicate catalog code"));
                }
                selected.push(&catalog.entries[index]);
            }
            selected
        } else {
            catalog.entries.iter().collect()
        };
        let country_value = country.as_ref().map(LitStr::value);
        let category_value = category.as_ref().map(LitStr::value);
        selected.retain(|entry| {
            country_value
                .as_deref()
                .is_none_or(|filter| entry.country.as_deref() == Some(filter))
                && category_value
                    .as_deref()
                    .is_none_or(|filter| entry.category.as_deref() == Some(filter))
        });
        if selected.is_empty() {
            return Err(syn::Error::new_spanned(
                &standard,
                "catalog filters select no codes",
            ));
        }
        let mut input: ItemEnum = syn::parse2(item)?;
        prepare(&mut input)?;
        let description = format!(
            "{} catalog selection. Requires iso-types' `{}` catalog feature.",
            standard.value(),
            catalog.feature
        );
        input.attrs.push(parse_quote!(#[doc = #description]));
        let variants = selected
            .iter()
            .map(|entry| syn::parse_str::<Ident>(&entry.variant))
            .collect::<syn::Result<Vec<_>>>()?;
        let codes: Vec<_> = selected
            .iter()
            .map(|entry| LitStr::new(&entry.code, standard.span()))
            .collect();
        set_variants(&mut input, &variants, &codes);
        Ok(Self {
            input,
            standard,
            type_name: syn::parse_str(&catalog.type_name)?,
            variants,
            complete: selected.len() == catalog.entries.len(),
        })
    }
}

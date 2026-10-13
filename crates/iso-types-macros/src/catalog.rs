#[cfg(feature = "locales")]
use heck::ToUpperCamelCase as _;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemEnum, LitStr, Path, Token, parse_quote, punctuated::Punctuated};

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
    let derives: Vec<Path> = ["Clone", "Copy", "Debug", "Eq", "Hash", "PartialEq"]
        .into_iter()
        .filter(|name| !seen.contains(*name))
        .map(syn::parse_str)
        .collect::<syn::Result<_>>()?;
    if !derives.is_empty() {
        input.attrs.push(parse_quote!(#[derive(#(#derives),*)]));
    }
    Ok(())
}

pub(crate) fn expand(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let mut standard = None;
    let mut codes = None;
    let parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("standard") && standard.is_none() {
            standard = Some(meta.value()?.parse::<LitStr>()?);
        } else if meta.path.is_ident("codes") && codes.is_none() {
            let content;
            let value = meta.value()?;
            syn::bracketed!(content in value);
            codes = Some(Punctuated::<LitStr, Token![,]>::parse_terminated(&content)?);
        } else {
            return Err(meta.error("expected standard = \"...\" or codes = [\"...\"] once each"));
        }
        Ok(())
    });
    use syn::parse::Parser as _;
    parser.parse2(attr)?;
    let standard = standard
        .ok_or_else(|| syn::Error::new(proc_macro2::Span::call_site(), "standard is required"))?;
    let data: serde_json::Value = serde_json::from_str(include_str!("../data/catalogs.json"))
        .map_err(|error| syn::Error::new_spanned(&standard, error))?;
    let catalog = data
        .get(standard.value())
        .ok_or_else(|| syn::Error::new_spanned(&standard, "unsupported ISO catalog"))?;
    let records = catalog["entries"]
        .as_array()
        .ok_or_else(|| syn::Error::new_spanned(&standard, "invalid catalog"))?;
    let selected = if let Some(codes) = codes {
        if codes.is_empty() {
            return Err(syn::Error::new_spanned(
                &standard,
                "select at least one code",
            ));
        }
        let mut selected = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for code in codes {
            let value = code.value();
            let entry = records
                .iter()
                .find(|r| r["code"].as_str() == Some(value.as_str()))
                .ok_or_else(|| {
                    syn::Error::new_spanned(
                        &code,
                        "expected a canonical code in this maintained catalog",
                    )
                })?;
            if !seen.insert(value) {
                return Err(syn::Error::new_spanned(code, "duplicate ISO code"));
            }
            selected.push(entry);
        }
        selected
    } else {
        records.iter().collect()
    };
    let mut input: ItemEnum = syn::parse2(item)?;
    prepare(&mut input)?;
    let variants = selected
        .iter()
        .map(|e| format_ident!("{}", e["variant"].as_str().expect("generated variant")))
        .collect::<Vec<_>>();
    let typename = format_ident!("{}", catalog["type"].as_str().expect("generated type"));
    input.variants = variants
        .iter()
        .map(|variant| -> syn::Variant { parse_quote!(#variant) })
        .collect();
    let name = &input.ident;
    let root = facade();
    let codes = selected
        .iter()
        .map(|e| e["code"].as_str().expect("generated code"))
        .collect::<Vec<_>>();
    let conversion_arms = variants
        .iter()
        .map(|v| quote!(Self::#v => #root::#typename::#v));
    let parsing_arms = variants
        .iter()
        .map(|v| quote!(#root::#typename::#v => Ok(Self::#v)));
    let unsupported = if selected.len() == records.len() {
        quote!()
    } else {
        quote!(
            _ => Err(#root::ParseIsoCodeError { standard: #standard, input: value.code().to_owned() })
        )
    };
    let fluent = if cfg!(feature = "fluent") {
        quote! {
            impl #root::__fluent::FluentMessage for #name {
                fn to_fluent_string_with(&self, lookup: &mut #root::__fluent::FluentMessageLookup<'_>) -> String {
                    #root::__fluent::FluentMessage::to_fluent_string_with(&self.catalog_value(), lookup)
                }
            }
        }
    } else {
        quote!()
    };
    Ok(quote! {
        #input
        impl #name {
            pub const ALL: &'static [Self] = &[#(Self::#variants),*];
            pub const STANDARD: &'static str = #standard;
            pub const fn catalog_value(self) -> #root::#typename { match self { #(#conversion_arms),* } }
            pub const fn code(self) -> &'static str { match self { #(Self::#variants => #codes),* } }
            pub const LABEL_SOURCE: #root::LabelSource = #root::#typename::LABEL_SOURCE;
            pub const fn english_name(self) -> &'static str { self.catalog_value().english_name() }
        }
        impl From<#name> for #root::#typename { fn from(value: #name) -> Self { value.catalog_value() } }
        impl std::convert::TryFrom<#root::#typename> for #name {
            type Error = #root::ParseIsoCodeError;
            fn try_from(value: #root::#typename) -> Result<Self, Self::Error> {
                match value { #(#parsing_arms,)* #unsupported }
            }
        }
        impl std::str::FromStr for #name {
            type Err = #root::ParseIsoCodeError;
            fn from_str(input: &str) -> Result<Self, #root::ParseIsoCodeError> { Self::try_from(input.parse::<#root::#typename>()?) }
        }
        impl #root::IsoCode for #name {
            const STANDARD: &'static str = Self::STANDARD;
            const LABEL_SOURCE: #root::LabelSource = Self::LABEL_SOURCE;
            const ALL: &'static [Self] = Self::ALL;
            fn code(self) -> &'static str { self.code() }
            fn english_name(self) -> &'static str { self.english_name() }
        }
        impl std::fmt::Display for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(self.english_name()) }
        }
        #fluent
    })
}

#[cfg(feature = "locales")]
pub(crate) fn locale_variant(code: &str) -> syn::Ident {
    format_ident!("{}", code.replace('-', "_").to_upper_camel_case())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_declarations_report_the_input_error() {
        let cases = [
            (
                quote!(),
                quote!(
                    enum Codes {}
                ),
                "standard is required",
            ),
            (
                quote!(standard = "9999"),
                quote!(
                    enum Codes {}
                ),
                "unsupported ISO catalog",
            ),
            (
                quote!(standard = "3166-1", codes = ["ZZ"]),
                quote!(
                    enum Codes {}
                ),
                "canonical code",
            ),
            (
                quote!(standard = "639-3", codes = ["aaa"]),
                quote!(
                    enum Codes {}
                ),
                "canonical code",
            ),
            (
                quote!(standard = "3166-3"),
                quote!(
                    enum Codes {}
                ),
                "unsupported ISO catalog",
            ),
            (
                quote!(standard = "639-5"),
                quote!(
                    enum Codes {}
                ),
                "unsupported ISO catalog",
            ),
            (
                quote!(standard = "639-3", codes = ["ang"]),
                quote!(
                    enum Codes {}
                ),
                "canonical code",
            ),
            (
                quote!(standard = "15924", codes = ["Egyp"]),
                quote!(
                    enum Codes {}
                ),
                "canonical code",
            ),
            (
                quote!(standard = "3166-1", codes = ["US", "US"]),
                quote!(
                    enum Codes {}
                ),
                "duplicate ISO code",
            ),
            (
                quote!(standard = "3166-1", codes = []),
                quote!(
                    enum Codes {}
                ),
                "select at least one code",
            ),
            (
                quote!(standard = "3166-1"),
                quote!(
                    enum Codes {
                        A,
                    }
                ),
                "empty enum",
            ),
            (
                quote!(standard = "3166-1"),
                quote!(
                    enum Codes<T> {}
                ),
                "without generics",
            ),
            (
                quote!(standard = "3166-1", standard = "639-1"),
                quote!(
                    enum Codes {}
                ),
                "once each",
            ),
        ];
        for (attributes, item, message) in cases {
            let error = expand(attributes, item).unwrap_err();
            assert!(error.to_string().contains(message), "{error}");
        }
    }
}

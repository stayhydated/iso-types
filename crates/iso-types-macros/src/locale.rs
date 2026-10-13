use super::catalog::{facade, locale_variant, prepare};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemEnum, parse_quote};
use writeable::TryWriteable;

fn canonical_locale(code: &str) -> syn::Result<icu_locale::Locale> {
    let mut locale: icu_locale::Locale = code.parse().map_err(|error| {
        syn::Error::new(
            proc_macro2::Span::call_site(),
            format!("invalid ICU locale {code}: {error}"),
        )
    })?;
    icu_locale::LocaleCanonicalizer::new_extended().canonicalize(&mut locale);
    Ok(locale)
}

fn english_label(code: &str) -> syn::Result<String> {
    let target = canonical_locale(code)?.id;
    let name = icu_locale::names::LanguageIdentifierDisplayName::try_new_heavy(
        icu_locale::locale!("en").into(),
        target,
        icu_locale::names::LanguageIdentifierDisplayNameOptions::default(),
    )
    .map_err(|error| syn::Error::new(proc_macro2::Span::call_site(), error))?;
    name.as_borrowed()
        .try_write_to_string()
        .map(|text| text.into_owned())
        .map_err(|_| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("ICU English name unavailable for configured locale {code}"),
            )
        })
}

pub(crate) fn expand(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let custom = if attr.is_empty() {
        false
    } else {
        let mode: syn::Ident = syn::parse2(attr)?;
        if mode != "custom" {
            return Err(syn::Error::new_spanned(
                mode,
                "expected custom or no arguments",
            ));
        }
        true
    };
    if custom && !cfg!(feature = "fluent") {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "custom labels require iso-types' fluent feature",
        ));
    }
    let mut input: ItemEnum = syn::parse2(item)?;
    prepare(&mut input)?;
    let config = es_fluent_toml::I18nConfig::read_from_manifest_dir()
        .map_err(|e| syn::Error::new_spanned(&input.ident, e))?;
    let mut locales = config
        .available_languages()
        .map_err(|e| syn::Error::new_spanned(&input.ident, e))?;
    let fallback = config
        .fallback_language_identifier()
        .map_err(|e| syn::Error::new_spanned(&input.ident, e))?;
    locales.push(fallback.clone());
    locales.sort_by_key(|l| l.to_string());
    locales.dedup();
    let codes = locales.iter().map(ToString::to_string).collect::<Vec<_>>();
    let labels = codes.iter().map(|code| {
        let canonical = canonical_locale(code)?.to_string();
        if &canonical != code {
            return Err(syn::Error::new_spanned(&input.ident,
                format!("use ICU canonical locale {canonical} in i18n.toml and locale directories instead of {code}")));
        }
        english_label(code)
    }).collect::<syn::Result<Vec<_>>>()?;
    let variants = codes.iter().map(|l| locale_variant(l)).collect::<Vec<_>>();
    let mut seen = std::collections::HashSet::new();
    for variant in &variants {
        if !seen.insert(variant.to_string()) {
            return Err(syn::Error::new_spanned(
                &input.ident,
                "locale names produce duplicate Rust variants",
            ));
        }
    }
    let fallback = locale_variant(&fallback.to_string());
    input.variants = variants
        .iter()
        .map(|v| -> syn::Variant { parse_quote!(#v) })
        .collect();
    let name = &input.ident;
    let root = facade();
    let fluent = if cfg!(feature = "fluent") {
        let ids = codes
            .iter()
            .map(|c| {
                if custom {
                    c.clone()
                } else {
                    format!("iso-locale-{c}")
                }
            })
            .collect::<Vec<_>>();
        let owner = if custom {
            quote!(env!("CARGO_PKG_NAME"))
        } else {
            quote!("iso-types")
        };
        let keepalive = format_ident!("__ISO_LOCALE_LINK_{name}");
        let inventory = if custom {
            let mod_name = format_ident!("__iso_locale_inventory_{name}");
            let names = variants.iter().map(ToString::to_string).collect::<Vec<_>>();
            quote! {
                #[doc(hidden)]
                #[allow(non_snake_case)]
                mod #mod_name {
                    static VARIANTS: &[#root::__fluent::registry::FtlVariant] = &[
                        #(#root::__fluent::registry::__macro::ftl_variant(#names, #root::__fluent::registry::__macro::static_entry_id(#ids), &[], module_path!(), line!())),*
                    ];
                    static TYPE_INFO: #root::__fluent::registry::FtlTypeInfo = #root::__fluent::registry::__macro::ftl_type_info(
                        #root::__fluent::meta::TypeKind::Enum, stringify!(#name), VARIANTS,
                        #root::__fluent::registry::__macro::ftl_scope(env!("CARGO_PKG_NAME"), None), file!(), module_path!(), None,
                    );
                    #root::__fluent::__inventory::submit!(#root::__fluent::registry::RegisteredFtlType(&TYPE_INFO));
                }
            }
        } else {
            quote!()
        };
        quote! {
            #[cfg(target_arch = "wasm32")]
            #[doc(hidden)]
            #[used]
            static #keepalive: fn() -> usize = #root::i18n::force_link;
            impl #root::__fluent::FluentMessage for #name {
                fn to_fluent_string_with(&self, lookup: &mut #root::__fluent::FluentMessageLookup<'_>) -> String {
                    let id = match self { #(Self::#variants => #root::__fluent::registry::__macro::static_entry_id(#ids)),* };
                    lookup(#root::__fluent::registry::__macro::static_message_key(#owner, #root::__fluent::registry::__macro::static_domain(#owner), id), None)
                }
            }
            #inventory
        }
    } else {
        quote!()
    };
    Ok(quote! {
        #input
        impl #name {
            pub const ALL: &'static [Self] = &[#(Self::#variants),*];
            pub const fn code(self) -> &'static str { match self { #(Self::#variants => #codes),* } }
            pub const fn english_name(self) -> &'static str { match self { #(Self::#variants => #labels),* } }
        }
        impl Default for #name { fn default() -> Self { Self::#fallback } }
        impl From<#name> for #root::LanguageIdentifier {
            fn from(value: #name) -> Self { match value { #(#name::#variants => #root::langid!(#codes)),* } }
        }
        impl From<&#name> for #root::LanguageIdentifier { fn from(value: &#name) -> Self { (*value).into() } }
        impl From<#name> for #root::icu_locale::LanguageIdentifier {
            fn from(value: #name) -> Self { value.code().parse().expect("validated ICU locale") }
        }
        impl From<&#name> for #root::icu_locale::LanguageIdentifier { fn from(value: &#name) -> Self { (*value).into() } }
        impl std::convert::TryFrom<&#root::icu_locale::LanguageIdentifier> for #name {
            type Error = #root::LocaleConversionError;
            fn try_from(value: &#root::icu_locale::LanguageIdentifier) -> Result<Self, Self::Error> {
                Self::try_from(#root::parse_language_identifier(&value.to_string())?)
            }
        }
        impl std::convert::TryFrom<#root::icu_locale::LanguageIdentifier> for #name {
            type Error = #root::LocaleConversionError;
            fn try_from(value: #root::icu_locale::LanguageIdentifier) -> Result<Self, Self::Error> { Self::try_from(&value) }
        }
        impl std::convert::TryFrom<&#root::LanguageIdentifier> for #name {
            type Error = #root::LocaleConversionError;
            fn try_from(value: &#root::LanguageIdentifier) -> Result<Self, Self::Error> {
                let canonical = #root::canonicalize_language_identifier(value);
                match canonical.to_string().as_str() { #(#codes => Ok(Self::#variants),)* _ => Err(#root::LocaleConversionError::Unsupported(canonical)) }
            }
        }
        impl std::convert::TryFrom<#root::LanguageIdentifier> for #name {
            type Error = #root::LocaleConversionError;
            fn try_from(value: #root::LanguageIdentifier) -> Result<Self, Self::Error> { Self::try_from(&value) }
        }
        impl std::str::FromStr for #name {
            type Err = #root::LocaleConversionError;
            fn from_str(input: &str) -> Result<Self, Self::Err> {
                let locale = #root::parse_language_identifier(input)?;
                Self::try_from(&locale)
            }
        }
        impl std::fmt::Display for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(self.english_name()) }
        }
        #fluent
    })
}

#[cfg(test)]
mod tests {
    use super::english_label;

    #[test]
    fn configured_locale_names_require_complete_icu_data() {
        assert_eq!(
            english_label("zh-Hant-TW").unwrap(),
            "Traditional Chinese (Taiwan)"
        );
        assert!(english_label("aaa").is_err());
        assert!(english_label("en-fonipa").is_ok());
        assert!(english_label("en-abcde").is_err());
    }
}

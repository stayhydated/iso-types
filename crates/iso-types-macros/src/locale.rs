use super::catalog::{facade, locale_variant, prepare};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemEnum, parse_quote};

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
        }
        impl Default for #name { fn default() -> Self { Self::#fallback } }
        impl From<#name> for #root::LanguageIdentifier {
            fn from(value: #name) -> Self { match value { #(#name::#variants => #root::langid!(#codes)),* } }
        }
        impl From<&#name> for #root::LanguageIdentifier { fn from(value: &#name) -> Self { (*value).into() } }
        impl std::convert::TryFrom<&#root::LanguageIdentifier> for #name {
            type Error = #root::LocaleConversionError;
            fn try_from(value: &#root::LanguageIdentifier) -> Result<Self, Self::Error> {
                match value.to_string().as_str() { #(#codes => Ok(Self::#variants),)* _ => Err(#root::LocaleConversionError::Unsupported(value.clone())) }
            }
        }
        impl std::convert::TryFrom<#root::LanguageIdentifier> for #name {
            type Error = #root::LocaleConversionError;
            fn try_from(value: #root::LanguageIdentifier) -> Result<Self, Self::Error> { Self::try_from(&value) }
        }
        impl std::str::FromStr for #name {
            type Err = #root::LocaleConversionError;
            fn from_str(input: &str) -> Result<Self, Self::Err> {
                let locale = input.parse::<#root::LanguageIdentifier>().map_err(|source| #root::LocaleConversionError::Invalid { input: input.to_owned(), source })?;
                Self::try_from(&locale)
            }
        }
        impl std::fmt::Display for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(&#root::locale_english_name(&self.into())) }
        }
        #fluent
    })
}

//! Emit locale conversions and Fluent registration from validated locale data.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::enum_input::facade;

use super::model::Model;

fn fluent(model: &Model, root: &TokenStream) -> TokenStream {
    let Model {
        input,
        custom,
        variants,
        codes,
        ..
    } = model;
    let name = &input.ident;
    if cfg!(feature = "fluent") {
        let ids = codes
            .iter()
            .map(|c| {
                if *custom {
                    c.value()
                } else {
                    format!("iso-locale-{}", c.value())
                }
            })
            .collect::<Vec<_>>();
        let owner = if *custom {
            quote!(env!("CARGO_PKG_NAME"))
        } else {
            quote!("iso-types")
        };
        let keepalive = format_ident!("__ISO_LOCALE_LINK_{name}");
        let inventory = if *custom {
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
                fn to_fluent_string_with(&self, lookup: &mut #root::__fluent::FluentMessageLookup<'_>) -> ::std::string::String {
                    let id = match self { #(Self::#variants => #root::__fluent::registry::__macro::static_entry_id(#ids)),* };
                    lookup(#root::__fluent::registry::__macro::static_message_key(#owner, #root::__fluent::registry::__macro::static_domain(#owner), id), None)
                }
            }
            #inventory
        }
    } else {
        quote!()
    }
}

pub(super) fn emit(model: &Model) -> TokenStream {
    let Model {
        input,
        fallback,
        variants,
        codes,
        labels,
        ..
    } = model;
    let name = &input.ident;
    let root = facade();
    let fluent = fluent(model, &root);
    quote! {
        #input
        impl #name {
            pub const fn code(self) -> &'static str { self.into_str() }
            pub const fn english_name(self) -> &'static str { match self { #(Self::#variants => #labels),* } }
            fn __iso_locale_from_canonical(value: #root::LanguageIdentifier) -> ::core::result::Result<Self, #root::LocaleConversionError> {
                match value.to_string().as_str() {
                    #(#codes => ::core::result::Result::Ok(Self::#variants),)*
                    _ => ::core::result::Result::Err(#root::LocaleConversionError::Unsupported(value)),
                }
            }
        }
        impl ::core::default::Default for #name { fn default() -> Self { Self::#fallback } }
        impl ::core::convert::From<#name> for #root::LanguageIdentifier {
            fn from(value: #name) -> Self { match value { #(#name::#variants => #root::langid!(#codes)),* } }
        }
        impl ::core::convert::From<&#name> for #root::LanguageIdentifier { fn from(value: &#name) -> Self { ::core::convert::From::from(*value) } }
        impl ::core::convert::From<#name> for #root::icu_locale::LanguageIdentifier {
            fn from(value: #name) -> Self { value.code().parse().expect("validated ICU locale") }
        }
        impl ::core::convert::From<&#name> for #root::icu_locale::LanguageIdentifier { fn from(value: &#name) -> Self { ::core::convert::From::from(*value) } }
        impl ::std::convert::TryFrom<&#root::icu_locale::LanguageIdentifier> for #name {
            type Error = #root::LocaleConversionError;
            fn try_from(value: &#root::icu_locale::LanguageIdentifier) -> ::core::result::Result<Self, Self::Error> {
                Self::__iso_locale_from_canonical(#root::parse_language_identifier(&value.to_string())?)
            }
        }
        impl ::std::convert::TryFrom<#root::icu_locale::LanguageIdentifier> for #name {
            type Error = #root::LocaleConversionError;
            fn try_from(value: #root::icu_locale::LanguageIdentifier) -> ::core::result::Result<Self, Self::Error> {
                <Self as ::core::convert::TryFrom<&#root::icu_locale::LanguageIdentifier>>::try_from(&value)
            }
        }
        impl ::std::convert::TryFrom<&#root::LanguageIdentifier> for #name {
            type Error = #root::LocaleConversionError;
            fn try_from(value: &#root::LanguageIdentifier) -> ::core::result::Result<Self, Self::Error> {
                Self::__iso_locale_from_canonical(#root::canonicalize_language_identifier(value)?)
            }
        }
        impl ::std::convert::TryFrom<#root::LanguageIdentifier> for #name {
            type Error = #root::LocaleConversionError;
            fn try_from(value: #root::LanguageIdentifier) -> ::core::result::Result<Self, Self::Error> {
                <Self as ::core::convert::TryFrom<&#root::LanguageIdentifier>>::try_from(&value)
            }
        }
        impl ::std::str::FromStr for #name {
            type Err = #root::LocaleConversionError;
            fn from_str(input: &str) -> ::core::result::Result<Self, Self::Err> {
                Self::__iso_locale_from_canonical(#root::parse_language_identifier(input)?)
            }
        }
        impl ::std::fmt::Display for #name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result { f.write_str(self.english_name()) }
        }
        #fluent
    }
}

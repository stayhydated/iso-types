//! Emit implementations from a validated catalog selection.

use proc_macro2::TokenStream;
use quote::quote;

use crate::enum_input::facade;

use super::model::Model;

pub(super) fn emit(model: &Model) -> TokenStream {
    let Model {
        input,
        standard,
        type_name: typename,
        variants,
        complete,
    } = model;
    let name = &input.ident;
    let root = facade();
    let conversion_arms = variants
        .iter()
        .map(|v| quote!(Self::#v => #root::#typename::#v));
    let parsing_arms = variants
        .iter()
        .map(|v| quote!(#root::#typename::#v => ::core::result::Result::Ok(Self::#v)));
    let unsupported = if *complete {
        quote!()
    } else {
        quote!(
            _ => ::core::result::Result::Err(#root::ParseIsoCodeError { standard: #standard, input: value.code().to_owned() })
        )
    };
    let fluent = if cfg!(feature = "fluent") {
        quote! {
            impl #root::__fluent::FluentMessage for #name {
                fn to_fluent_string_with(&self, lookup: &mut #root::__fluent::FluentMessageLookup<'_>) -> ::std::string::String {
                    #root::__fluent::FluentMessage::to_fluent_string_with(&self.catalog_value(), lookup)
                }
            }
        }
    } else {
        quote!()
    };
    quote! {
        #input
        impl #name {
            pub const STANDARD: &'static str = #standard;
            pub const fn catalog_value(self) -> #root::#typename { match self { #(#conversion_arms),* } }
            pub const fn code(self) -> &'static str { self.into_str() }
            pub const LABEL_SOURCE: #root::LabelSource = #root::#typename::LABEL_SOURCE;
            pub const fn english_name(self) -> &'static str { self.catalog_value().english_name() }
        }
        impl ::core::convert::From<#name> for #root::#typename { fn from(value: #name) -> Self { value.catalog_value() } }
        impl ::std::convert::TryFrom<#root::#typename> for #name {
            type Error = #root::ParseIsoCodeError;
            fn try_from(value: #root::#typename) -> ::core::result::Result<Self, Self::Error> {
                match value { #(#parsing_arms,)* #unsupported }
            }
        }
        impl ::std::str::FromStr for #name {
            type Err = #root::ParseIsoCodeError;
            fn from_str(input: &str) -> ::core::result::Result<Self, #root::ParseIsoCodeError> {
                <Self as ::core::convert::TryFrom<#root::#typename>>::try_from(input.parse::<#root::#typename>()?)
            }
        }
        impl #root::IsoCode for #name {
            const STANDARD: &'static str = Self::STANDARD;
            const LABEL_SOURCE: #root::LabelSource = Self::LABEL_SOURCE;
            fn code(self) -> &'static str { self.code() }
            fn english_name(self) -> &'static str { self.english_name() }
        }
        impl ::std::fmt::Display for #name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result { f.write_str(self.english_name()) }
        }
        #fluent
    }
}

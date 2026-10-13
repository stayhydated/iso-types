//! Attribute macros re-exported by `iso-types`.
use proc_macro::TokenStream;
mod catalog;
#[cfg(feature = "locales")]
mod locale;

/// Fill an empty enum from an ISO catalog, optionally selecting `codes = [...]`.
#[proc_macro_attribute]
pub fn iso_enum(attr: TokenStream, item: TokenStream) -> TokenStream {
    catalog::expand(attr.into(), item.into())
        .unwrap_or_else(|error| error.to_compile_error())
        .into()
}

/// Fill an empty locale enum from the package's i18n.toml and locale directories.
#[cfg(feature = "locales")]
#[proc_macro_attribute]
pub fn iso_locale(attr: TokenStream, item: TokenStream) -> TokenStream {
    locale::expand(attr.into(), item.into())
        .unwrap_or_else(|error| error.to_compile_error())
        .into()
}

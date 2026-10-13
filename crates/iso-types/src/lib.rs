#![doc = include_str!("../README.md")]

extern crate self as iso_types;

mod generated;
// The catalog glob is empty when every catalog feature is disabled.
#[allow(unused_imports)]
pub use generated::*;

#[cfg(feature = "macros")]
pub use iso_types_macros::iso_enum;
#[cfg(feature = "locales")]
pub use iso_types_macros::iso_locale;
#[cfg(feature = "locales")]
pub use unic_langid::{LanguageIdentifier, langid};
#[cfg(feature = "locales")]
mod locale;
#[cfg(feature = "locales")]
pub use locale::{LocaleConversionError, locale_english_name};
#[cfg(feature = "fluent")]
pub mod i18n;
#[cfg(feature = "fluent")]
#[doc(hidden)]
pub use es_fluent as __fluent;

/// An unrecognized code in a specific ISO catalog.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseIsoCodeError {
    pub standard: &'static str,
    pub input: String,
}
impl std::fmt::Display for ParseIsoCodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unrecognized ISO {} code: {}", self.standard, self.input)
    }
}
impl std::error::Error for ParseIsoCodeError {}

/// Shared code and label contract for complete catalogs and declared subsets.
pub trait IsoCode: 'static + Copy + Eq + std::str::FromStr<Err = ParseIsoCodeError> {
    const STANDARD: &'static str;
    const ALL: &'static [Self];
    fn code(self) -> &'static str;
    fn english_name(self) -> &'static str;
}

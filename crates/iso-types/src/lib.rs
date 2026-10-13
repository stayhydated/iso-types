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
mod current;
#[cfg(feature = "locales")]
mod locale;
#[cfg(feature = "locales")]
pub use locale::{
    LocaleConversionError, LocaleNameError, canonicalize_language_identifier,
    icu_locale_english_name, locale_english_name, parse_language_identifier,
};
#[cfg(feature = "icu")]
pub use {icu_calendar, icu_locale};
#[cfg(feature = "fluent")]
pub mod i18n;
#[cfg(feature = "icu")]
mod icu;
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

/// Shared code and label contract for maintained catalogs and declared subsets.
pub trait IsoCode: 'static + Copy + Eq + std::str::FromStr<Err = ParseIsoCodeError> {
    const STANDARD: &'static str;
    const LABEL_SOURCE: LabelSource;
    const ALL: &'static [Self];
    fn code(self) -> &'static str;
    fn english_name(self) -> &'static str;
}

/// The authority that owns a catalog's English labels.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LabelSource {
    Icu4x,
    Fluent,
}

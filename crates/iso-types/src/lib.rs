#![doc = include_str!("../README.md")]

extern crate self as iso_types;

mod catalogs;
mod code;
pub use code::{IsoCode, LabelSource, ParseIsoCodeError};
// The catalog glob is empty when every catalog feature is disabled.
#[allow(unused_imports)]
pub use catalogs::*;

#[cfg(feature = "macros")]
pub use iso_types_macros::iso_enum;
#[cfg(feature = "locales")]
pub use iso_types_macros::iso_locale;
#[cfg(feature = "locales")]
pub use unic_langid::{LanguageIdentifier, langid};
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

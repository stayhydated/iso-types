#![doc = include_str!("../README.md")]

extern crate self as iso_types;

mod catalogs;
mod code;
pub use code::{IsoCode, LabelSource, ParseIsoCodeError};
pub use strum;
pub use strum::{EnumCount, IntoEnumIterator, VariantArray};
// The catalog glob is empty when every catalog feature is disabled.
#[allow(unused_imports)]
pub use catalogs::*;

mod geography;
mod supplemental;
#[cfg(feature = "time-zones")]
mod time_zone;
#[cfg(feature = "time-zones")]
pub use jiff;
#[cfg(feature = "time-zone-localization")]
pub use {icu_datetime, icu_time};
#[cfg(feature = "regional-preferences")]
mod regional;
#[cfg(feature = "regional-preferences")]
pub use regional::{
    CalendarAlgorithm, HijriCalendarAlgorithm, HourCycle, MeasurementSystem, PaperSize,
    RegionalPreferences, WeekPreferences,
};
#[cfg(feature = "unit-preferences")]
mod unit_preferences;
#[cfg(feature = "unit-preferences")]
pub use unit_preferences::{PreferredUnit, UnitPreference};
#[cfg(feature = "language-relations")]
mod language_relations;
#[cfg(feature = "language-relations")]
pub use language_relations::{LanguageStatus, LanguageUsage};
#[cfg(feature = "calling-codes")]
mod telephone;
#[cfg(feature = "phone-numbers")]
pub use phonenumber;
#[cfg(feature = "addresses")]
mod address;
#[cfg(feature = "addresses")]
pub use address::{AddressLabelOverride, AddressMetadata};
#[cfg(feature = "3166-2")]
pub use geography::SubdivisionKind;

#[cfg(feature = "unit-conversion")]
mod measurement;
#[cfg(feature = "unit-conversion")]
pub use icu_experimental;
#[cfg(feature = "unit-conversion")]
pub use measurement::UnitConversionError;

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

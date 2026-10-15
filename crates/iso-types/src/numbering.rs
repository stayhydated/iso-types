//! Numbering systems and resolved CLDR locale defaults.

mod data;
pub use data::NumberingSystemKind;

use crate::{LocaleConversionError, NumberingSystem};

/// CLDR defaults after locale inheritance and numbering-category fallback.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NumberingPreferences {
    pub default: NumberingSystem,
    pub native: NumberingSystem,
    pub traditional: NumberingSystem,
    pub finance: NumberingSystem,
}

/// Resolve a current/common language identifier to its CLDR numbering preferences.
///
/// Explicit application and Unicode `nu` preferences take precedence over these defaults.
pub fn numbering_preferences(input: &str) -> Result<NumberingPreferences, LocaleConversionError> {
    let canonical = crate::parse_language_identifier(input)?;
    let mut id: icu_locale::LanguageIdentifier = canonical
        .to_string()
        .parse()
        .expect("validated language identifier has ICU syntax");
    let find = |key: &str| {
        data::PREFERENCES
            .binary_search_by_key(&key, |(locale, _)| *locale)
            .ok()
            .map(|index| data::PREFERENCES[index].1)
    };
    if let Some(preferences) = find(&id.to_string()) {
        return Ok(preferences);
    }
    icu_locale::LocaleExpander::new_extended().minimize(&mut id);
    loop {
        if let Some(preferences) = find(&id.to_string()) {
            return Ok(preferences);
        }
        if id.region.take().is_some() || id.script.take().is_some() {
            continue;
        }
        return Err(LocaleConversionError::Unsupported(canonical));
    }
}

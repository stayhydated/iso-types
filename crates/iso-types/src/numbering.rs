//! Numbering systems and resolved CLDR locale defaults.

mod data;
pub use data::NumberingSystemKind;

use crate::{LocaleConversionError, NumberingSystemId};

/// CLDR defaults after locale inheritance and numbering-category fallback.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NumberingPreferences {
    pub default: NumberingSystemId,
    pub native: NumberingSystemId,
    pub traditional: NumberingSystemId,
    pub finance: NumberingSystemId,
}

/// Resolve a current/common language identifier to its CLDR numbering preferences.
///
/// Explicit application and Unicode `nu` preferences take precedence over these defaults.
pub fn numbering_preferences(input: &str) -> Result<NumberingPreferences, LocaleConversionError> {
    let canonical = crate::parse_language_identifier(input)?;
    let id: icu_locale::LanguageIdentifier = canonical
        .to_string()
        .parse()
        .expect("validated language identifier has ICU syntax");
    let find = |key: &str| {
        data::PREFERENCES
            .binary_search_by_key(&key, |(locale, _)| *locale)
            .ok()
            .map(|index| data::PREFERENCES[index].1)
    };
    // CLDR inheritance: likely-script normalization, then explicit and truncation parents.
    let mut chain = icu_locale::LocaleFallbacker::new()
        .for_config(Default::default())
        .fallback_for((&id).into());
    while !chain.get().is_unknown() {
        if let Some(preferences) = find(&chain.get().to_string()) {
            return Ok(preferences);
        }
        chain.step();
    }
    Err(LocaleConversionError::Unsupported(canonical))
}

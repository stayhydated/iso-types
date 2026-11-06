use icu_locale::names::{LanguageIdentifierDisplayName, LanguageIdentifierDisplayNameOptions};
use icu_locale::{LanguageIdentifier as IcuLanguageIdentifier, Locale, LocaleCanonicalizer};
use unic_langid::LanguageIdentifier;
use writeable::TryWriteable;

/// Parse a BCP 47 language identifier and normalize it with ICU4X's canonicalizer.
///
/// Deprecated representations map to their replacements, so `iw-IL` parses as
/// `he-IL` and `eng-US` as `en-US`.
pub fn parse_language_identifier(input: &str) -> Result<LanguageIdentifier, LocaleConversionError> {
    let id = input.parse::<IcuLanguageIdentifier>().map_err(|source| {
        LocaleConversionError::Invalid {
            input: input.to_owned(),
            source,
        }
    })?;
    Ok(to_fluent_identifier(canonicalize(id)))
}

fn canonicalize(id: IcuLanguageIdentifier) -> IcuLanguageIdentifier {
    let mut locale: Locale = id.into();
    LocaleCanonicalizer::new_extended().canonicalize(&mut locale);
    locale.id
}

fn to_fluent_identifier(id: IcuLanguageIdentifier) -> LanguageIdentifier {
    id.to_string()
        .parse()
        .expect("ICU language identifiers are valid Fluent identifiers")
}

fn to_icu_identifier(locale: &LanguageIdentifier) -> IcuLanguageIdentifier {
    locale
        .to_string()
        .parse()
        .expect("Fluent identifier has valid ICU syntax")
}

/// Canonicalize a language identifier with ICU4X, mapping deprecated subtags.
pub fn canonicalize_language_identifier(locale: &LanguageIdentifier) -> LanguageIdentifier {
    to_fluent_identifier(canonicalize(to_icu_identifier(locale)))
}

/// Format `target`'s name in `display` with ICU4X, requiring names for every component.
pub fn icu_locale_display_name(
    target: &IcuLanguageIdentifier,
    display: &IcuLanguageIdentifier,
) -> Result<String, LocaleNameError> {
    let name = LanguageIdentifierDisplayName::try_new_heavy(
        Locale::from(display.clone()).into(),
        target.clone(),
        LanguageIdentifierDisplayNameOptions::default(),
    )
    .map_err(LocaleNameError::Data)?;
    name.as_borrowed()
        .try_write_to_string()
        .map(|text| text.into_owned())
        .map_err(|_| LocaleNameError::Unavailable {
            locale: target.clone(),
            display: display.clone(),
        })
}

/// Format an English locale name from ICU4X, requiring names for every component.
pub fn icu_locale_english_name(locale: &IcuLanguageIdentifier) -> Result<String, LocaleNameError> {
    icu_locale_display_name(locale, &icu_locale::langid!("en"))
}

/// Format a Fluent-facing language identifier with ICU4X's English name data.
pub fn locale_english_name(locale: &LanguageIdentifier) -> Result<String, LocaleNameError> {
    icu_locale_english_name(&to_icu_identifier(locale))
}

/// Format a locale's name in that locale, such as `français (France)` for `fr-FR`.
pub fn locale_autonym(locale: &LanguageIdentifier) -> Result<String, LocaleNameError> {
    let id = to_icu_identifier(locale);
    icu_locale_display_name(&id, &id)
}

/// Format `target`'s name in the `display` language, such as `French (France)` in English.
pub fn locale_display_name(
    target: &LanguageIdentifier,
    display: &LanguageIdentifier,
) -> Result<String, LocaleNameError> {
    icu_locale_display_name(&to_icu_identifier(target), &to_icu_identifier(display))
}

/// Name data failures and incomplete component names remain explicit.
#[derive(Debug)]
pub enum LocaleNameError {
    /// ICU4X could not load display-name data.
    Data(icu_provider::DataError),
    /// At least one component has no name in the display language.
    Unavailable {
        locale: IcuLanguageIdentifier,
        display: IcuLanguageIdentifier,
    },
}
impl std::fmt::Display for LocaleNameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Data(error) => error.fmt(f),
            Self::Unavailable { locale, display } => {
                write!(f, "ICU {display} name unavailable for {locale}")
            }
        }
    }
}
impl std::error::Error for LocaleNameError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Data(error) => Some(error),
            Self::Unavailable { .. } => None,
        }
    }
}

/// Invalid syntax and valid but unconfigured locale identifiers remain distinct.
#[derive(Debug)]
pub enum LocaleConversionError {
    Invalid {
        input: String,
        source: icu_locale::ParseError,
    },
    Unsupported(LanguageIdentifier),
}
impl std::fmt::Display for LocaleConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid { input, source } => write!(f, "invalid locale '{input}': {source}"),
            Self::Unsupported(locale) => write!(f, "unsupported locale: {locale}"),
        }
    }
}
impl std::error::Error for LocaleConversionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Invalid { source, .. } => Some(source),
            Self::Unsupported(_) => None,
        }
    }
}

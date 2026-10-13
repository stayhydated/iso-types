use crate::LanguageIdentifier;
use icu_locale::names::{LanguageIdentifierDisplayName, LanguageIdentifierDisplayNameOptions};
use icu_locale::{
    LanguageIdentifier as IcuLanguageIdentifier, Locale, LocaleCanonicalizer, locale,
};
use writeable::TryWriteable;

/// Parse current, common locale data and normalize current ISO representations with ICU4X.
pub fn parse_language_identifier(input: &str) -> Result<LanguageIdentifier, LocaleConversionError> {
    let id = input.parse::<IcuLanguageIdentifier>().map_err(|source| {
        LocaleConversionError::Invalid {
            input: input.to_owned(),
            source,
        }
    })?;
    let id = current_identifier(id)?;
    Ok(to_fluent_identifier(id))
}

fn current_identifier(
    id: IcuLanguageIdentifier,
) -> Result<IcuLanguageIdentifier, LocaleConversionError> {
    if !crate::current::current_locale(&id) {
        return Err(LocaleConversionError::OutOfScope(to_fluent_identifier(id)));
    }
    let canonical = canonicalize(id);
    if !crate::current::current_locale(&canonical) {
        return Err(LocaleConversionError::OutOfScope(to_fluent_identifier(
            canonical,
        )));
    }
    Ok(canonical)
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

/// Canonicalize current identifiers; excluded and deprecated components return an error.
pub fn canonicalize_language_identifier(
    locale: &LanguageIdentifier,
) -> Result<LanguageIdentifier, LocaleConversionError> {
    let id = locale
        .to_string()
        .parse()
        .expect("Fluent identifier has valid ICU syntax");
    current_identifier(id).map(to_fluent_identifier)
}

/// Format an English locale name from ICU4X, requiring names for every component.
pub fn icu_locale_english_name(locale: &IcuLanguageIdentifier) -> Result<String, LocaleNameError> {
    let target = current_identifier(locale.clone())
        .map_err(|_| LocaleNameError::OutOfScope(locale.clone()))?;
    let name = LanguageIdentifierDisplayName::try_new_heavy(
        locale!("en").into(),
        target.clone(),
        LanguageIdentifierDisplayNameOptions::default(),
    )
    .map_err(LocaleNameError::Data)?;
    name.as_borrowed()
        .try_write_to_string()
        .map(|text| text.into_owned())
        .map_err(|_| LocaleNameError::Unavailable(target))
}

/// Format a Fluent-facing language identifier with ICU4X's English name data.
pub fn locale_english_name(locale: &LanguageIdentifier) -> Result<String, LocaleNameError> {
    let id = locale
        .to_string()
        .parse()
        .expect("Fluent identifier has valid ICU syntax");
    icu_locale_english_name(&id)
}

/// Name data failures and incomplete component names remain explicit.
#[derive(Debug)]
pub enum LocaleNameError {
    Data(icu_provider::DataError),
    Unavailable(IcuLanguageIdentifier),
    OutOfScope(IcuLanguageIdentifier),
}
impl std::fmt::Display for LocaleNameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Data(error) => error.fmt(f),
            Self::Unavailable(locale) => write!(f, "ICU English name unavailable for {locale}"),
            Self::OutOfScope(locale) => write!(f, "locale outside current data scope: {locale}"),
        }
    }
}
impl std::error::Error for LocaleNameError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Data(error) => Some(error),
            Self::Unavailable(_) | Self::OutOfScope(_) => None,
        }
    }
}

/// Invalid syntax and valid but unselected locale identifiers remain distinct.
#[derive(Debug)]
pub enum LocaleConversionError {
    Invalid {
        input: String,
        source: icu_locale::ParseError,
    },
    Unsupported(LanguageIdentifier),
    OutOfScope(LanguageIdentifier),
}
impl std::fmt::Display for LocaleConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid { input, source } => write!(f, "invalid locale '{input}': {source}"),
            Self::Unsupported(locale) => write!(f, "unsupported locale: {locale}"),
            Self::OutOfScope(locale) => write!(f, "locale outside current data scope: {locale}"),
        }
    }
}
impl std::error::Error for LocaleConversionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Invalid { source, .. } => Some(source),
            Self::Unsupported(_) | Self::OutOfScope(_) => None,
        }
    }
}

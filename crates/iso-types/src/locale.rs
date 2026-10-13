use crate::{Iso3166_1, Iso6391, Iso6392, Iso6393, Iso15924, LanguageIdentifier};

/// Convert an ISO-based language identifier to an English locale label.
/// Unassigned or private components retain their canonical code.
pub fn locale_english_name(locale: &LanguageIdentifier) -> String {
    let code = locale.language.to_string();
    let language = code
        .parse::<Iso6391>()
        .map(|v| v.english_name())
        .or_else(|_| code.parse::<Iso6392>().map(|v| v.english_name()))
        .or_else(|_| code.parse::<Iso6393>().map(|v| v.english_name()));
    let mut name = language.unwrap_or(&code).to_owned();
    let mut qualifiers = Vec::new();
    if let Some(script) = locale.script {
        let code = script.to_string();
        qualifiers.push(
            code.parse::<Iso15924>()
                .map_or(code, |v| v.english_name().to_owned()),
        );
    }
    if let Some(region) = locale.region {
        let code = region.to_string();
        qualifiers.push(
            code.parse::<Iso3166_1>()
                .map_or(code, |v| v.english_name().to_owned()),
        );
    }
    qualifiers.extend(locale.variants().map(|v| v.to_string()));
    if !qualifiers.is_empty() {
        name.push_str(" (");
        name.push_str(&qualifiers.join(", "));
        name.push(')');
    }
    name
}

/// Invalid syntax and valid but unselected locale identifiers remain distinct.
#[derive(Debug)]
pub enum LocaleConversionError {
    Invalid {
        input: String,
        source: unic_langid::LanguageIdentifierError,
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

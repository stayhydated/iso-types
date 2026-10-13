//! Validate configured identifiers, names, and Rust variants before expansion.

use std::collections::HashSet;

use heck::ToUpperCamelCase as _;
use icu_locale::{LanguageIdentifier, Locale, LocaleCanonicalizer};
use proc_macro2::{Span, TokenStream};
use syn::{Ident, ItemEnum, LitStr};
use writeable::TryWriteable as _;

use crate::enum_input::{prepare, set_variants};

pub(super) struct Model {
    pub input: ItemEnum,
    pub custom: bool,
    pub fallback: Ident,
    pub variants: Vec<Ident>,
    pub codes: Vec<LitStr>,
    pub labels: Vec<LitStr>,
}

fn variant(code: &str) -> syn::Result<Ident> {
    syn::parse_str(&code.replace('-', "_").to_upper_camel_case())
}

fn canonical_locale(
    code: &str,
    span: Span,
    canonicalizer: &LocaleCanonicalizer,
) -> syn::Result<LanguageIdentifier> {
    let id: LanguageIdentifier = code
        .parse()
        .map_err(|error| syn::Error::new(span, format!("invalid ICU locale {code}: {error}")))?;
    if !super::scope::includes(&id) {
        return Err(syn::Error::new(
            span,
            format!("locale outside current data scope: {code}"),
        ));
    }
    let mut locale: Locale = id.into();
    canonicalizer.canonicalize(&mut locale);
    if !super::scope::includes(&locale.id) {
        return Err(syn::Error::new(
            span,
            format!("locale outside current data scope: {code}"),
        ));
    }
    Ok(locale.id)
}

fn english_label(target: LanguageIdentifier, span: Span) -> syn::Result<String> {
    let code = target.to_string();
    let name = icu_locale::names::LanguageIdentifierDisplayName::try_new_heavy(
        icu_locale::locale!("en").into(),
        target,
        icu_locale::names::LanguageIdentifierDisplayNameOptions::default(),
    )
    .map_err(|error| syn::Error::new(span, error))?;
    name.as_borrowed()
        .try_write_to_string()
        .map(|text| text.into_owned())
        .map_err(|_| {
            syn::Error::new(
                span,
                format!("ICU English name unavailable for configured locale {code}"),
            )
        })
}

impl Model {
    pub fn parse(attr: TokenStream, item: TokenStream) -> syn::Result<Self> {
        let custom = if attr.is_empty() {
            false
        } else {
            let mode: Ident = syn::parse2(attr)?;
            if mode != "custom" {
                return Err(syn::Error::new_spanned(
                    mode,
                    "expected custom or no arguments",
                ));
            }
            if !cfg!(feature = "fluent") {
                return Err(syn::Error::new_spanned(
                    mode,
                    "custom labels require iso-types' fluent feature",
                ));
            }
            true
        };
        let mut input: ItemEnum = syn::parse2(item)?;
        prepare(&mut input)?;
        let config = es_fluent_toml::I18nConfig::read_from_manifest_dir()
            .map_err(|error| syn::Error::new_spanned(&input.ident, error))?;
        let locales = config
            .available_languages()
            .map_err(|error| syn::Error::new_spanned(&input.ident, error))?;
        let fallback = config
            .fallback_language_identifier()
            .map_err(|error| syn::Error::new_spanned(&input.ident, error))?;
        Self::build(
            input,
            custom,
            locales.iter().map(ToString::to_string).collect(),
            &fallback.to_string(),
        )
    }

    fn build(
        mut input: ItemEnum,
        custom: bool,
        mut locales: Vec<String>,
        fallback: &str,
    ) -> syn::Result<Self> {
        locales.push(fallback.to_owned());
        locales.sort();
        locales.dedup();
        let span = input.ident.span();
        let canonicalizer = LocaleCanonicalizer::new_extended();
        let mut seen = HashSet::new();
        let mut variants = Vec::with_capacity(locales.len());
        let mut codes = Vec::with_capacity(locales.len());
        let mut labels = Vec::with_capacity(locales.len());
        for code in locales {
            let canonical = canonical_locale(&code, span, &canonicalizer)?;
            let canonical_code = canonical.to_string();
            if canonical_code != code {
                return Err(syn::Error::new(
                    span,
                    format!(
                        "use ICU canonical locale {canonical_code} in i18n.toml and locale directories instead of {code}",
                    ),
                ));
            }
            let ident = variant(&code)?;
            if !seen.insert(ident.to_string()) {
                return Err(syn::Error::new(
                    span,
                    "locale names produce duplicate Rust variants",
                ));
            }
            variants.push(ident);
            codes.push(LitStr::new(&code, span));
            labels.push(LitStr::new(&english_label(canonical, span)?, span));
        }
        set_variants(&mut input, &variants, &codes);
        Ok(Self {
            input,
            custom,
            fallback: variant(fallback)?,
            variants,
            codes,
            labels,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn configured_locales_require_current_canonical_identifiers_with_icu_names() {
        let input = parse_quote!(
            enum Languages {}
        );
        let model = Model::build(input, false, vec!["zh-Hant-TW".into()], "en-US").unwrap();
        assert_eq!(model.labels[1].value(), "Traditional Chinese (Taiwan)");
        for code in [
            "aaa",
            "en-fonipa",
            "iw-IL",
            "la",
            "en-BU",
            "de-DE-1901",
            "en-abcde",
        ] {
            assert!(
                Model::build(
                    parse_quote!(
                        enum Languages {}
                    ),
                    false,
                    vec![code.into()],
                    "en-US"
                )
                .is_err(),
                "{code}"
            );
        }
        let error = Model::build(
            parse_quote!(
                enum Languages {}
            ),
            false,
            vec!["eng-US".into()],
            "en-US",
        )
        .err()
        .unwrap();
        assert!(error.to_string().contains("use ICU canonical locale en-US"));
    }

    #[test]
    fn fallback_participates_in_the_sorted_unique_locale_selection() {
        let model = Model::build(
            parse_quote!(
                enum Languages {}
            ),
            false,
            vec!["fr-FR".into(), "en-US".into(), "fr-FR".into()],
            "en-US",
        )
        .unwrap();
        assert_eq!(
            model.codes.iter().map(LitStr::value).collect::<Vec<_>>(),
            ["en-US", "fr-FR"]
        );
        assert_eq!(model.fallback, "EnUs");
    }
}

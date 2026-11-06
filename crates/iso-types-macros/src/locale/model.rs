//! Validate configured identifiers, names, and Rust variants before expansion.

use std::collections::HashSet;

use heck::ToUpperCamelCase as _;
use icu_locale::{LanguageIdentifier, Locale, LocaleCanonicalizer};
use proc_macro2::{Span, TokenStream};
use syn::{Ident, ItemEnum, LitStr, Path, Token, punctuated::Punctuated};
use writeable::TryWriteable as _;

use crate::enum_input::{prepare, set_variants};

pub(super) struct Model {
    pub input: ItemEnum,
    pub custom: bool,
    pub fallback: Ident,
    pub variants: Vec<Ident>,
    pub codes: Vec<LitStr>,
    pub labels: Vec<LitStr>,
    pub autonyms: Vec<LitStr>,
}

fn variant(code: &str) -> syn::Result<Ident> {
    syn::parse_str(&code.replace('-', "_").to_upper_camel_case())
}

/// Parse a configured locale and require ICU's canonical spelling.
fn canonical_locale(
    code: &str,
    span: Span,
    canonicalizer: &LocaleCanonicalizer,
) -> syn::Result<LanguageIdentifier> {
    let id: LanguageIdentifier = code
        .parse()
        .map_err(|error| syn::Error::new(span, format!("invalid ICU locale {code}: {error}")))?;
    let mut locale: Locale = id.into();
    canonicalizer.canonicalize(&mut locale);
    let canonical_code = locale.id.to_string();
    if canonical_code != code {
        return Err(syn::Error::new(
            span,
            format!(
                "use ICU canonical locale {canonical_code} in i18n.toml and locale directories instead of {code}",
            ),
        ));
    }
    Ok(locale.id)
}

/// Format `target` in `display`, returning `None` when a component has no name.
fn display_name(target: &LanguageIdentifier, display: &LanguageIdentifier) -> Option<String> {
    let name = icu_locale::names::LanguageIdentifierDisplayName::try_new_heavy(
        Locale::from(display.clone()).into(),
        target.clone(),
        icu_locale::names::LanguageIdentifierDisplayNameOptions::default(),
    )
    .ok()?;
    name.as_borrowed()
        .try_write_to_string()
        .ok()
        .map(|text| text.into_owned())
}

fn english_label(target: &LanguageIdentifier, span: Span) -> syn::Result<String> {
    display_name(target, &icu_locale::langid!("en")).ok_or_else(|| {
        syn::Error::new(
            span,
            format!("ICU English name unavailable for configured locale {target}"),
        )
    })
}

/// The application implements labels through this macro; a derived `EsFluent` would conflict.
fn reject_es_fluent_derive(input: &ItemEnum) -> syn::Result<()> {
    if !cfg!(feature = "fluent") {
        return Ok(());
    }
    for attr in input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("derive"))
    {
        for path in attr.parse_args_with(Punctuated::<Path, Token![,]>::parse_terminated)? {
            if path
                .segments
                .last()
                .is_some_and(|segment| segment.ident == "EsFluent")
            {
                return Err(syn::Error::new_spanned(
                    path,
                    "#[iso_locale] implements FluentMessage; remove the EsFluent derive",
                ));
            }
        }
    }
    Ok(())
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
        reject_es_fluent_derive(&input)?;
        prepare(&mut input)?;
        // Read through es-fluent-build's manifest so compiler caches track locale changes.
        let error = |message: String| syn::Error::new_spanned(&input.ident, message);
        let inputs = es_fluent_derive_core::macro_support::PackageInputs::current()
            .map_err(|e| error(e.to_string()))?;
        let config = inputs
            .config()
            .map_err(|e| error(format!("failed to read i18n configuration: {e}")))?;
        let locales = inputs
            .languages(&config)
            .map_err(|e| error(format!("failed to collect available languages: {e}")))?;
        let fallback = config
            .fallback_language_identifier()
            .map_err(|e| error(format!("failed to parse fallback language: {e}")))?;
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
        let mut autonyms = Vec::with_capacity(locales.len());
        for code in locales {
            let canonical = canonical_locale(&code, span, &canonicalizer)?;
            let ident = variant(&code)?;
            if !seen.insert(ident.to_string()) {
                return Err(syn::Error::new(
                    span,
                    "locale names produce duplicate Rust variants",
                ));
            }
            let english = english_label(&canonical, span)?;
            // A locale without complete self-names keeps its English label.
            let autonym = display_name(&canonical, &canonical).unwrap_or_else(|| english.clone());
            variants.push(ident);
            codes.push(LitStr::new(&code, span));
            labels.push(LitStr::new(&english, span));
            autonyms.push(LitStr::new(&autonym, span));
        }
        set_variants(&mut input, &variants, &codes);
        Ok(Self {
            input,
            custom,
            fallback: variant(fallback)?,
            variants,
            codes,
            labels,
            autonyms,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    fn build(locales: &[&str]) -> syn::Result<Model> {
        Model::build(
            parse_quote!(
                enum Languages {}
            ),
            false,
            locales.iter().map(|code| (*code).to_owned()).collect(),
            "en-US",
        )
    }

    #[test]
    fn configured_locales_require_canonical_identifiers_with_icu_names() {
        let model = build(&["zh-Hant-TW"]).unwrap();
        assert_eq!(model.labels[1].value(), "Traditional Chinese (Taiwan)");
        for code in ["iw-IL", "en-BU", "eng-US", "sh"] {
            let error = build(&[code]).err().unwrap();
            assert!(
                error.to_string().contains("use ICU canonical locale"),
                "{code}: {error}"
            );
        }
        for code in ["not a locale", "en-abcdefghi"] {
            assert!(build(&[code]).is_err(), "{code}");
        }
    }

    #[test]
    fn every_locale_with_icu_english_names_is_accepted() {
        let model = build(&["mt", "lb", "fo", "eo", "la", "ca-ES-valencia", "be-tarask"]).unwrap();
        let labels = model.labels.iter().map(LitStr::value).collect::<Vec<_>>();
        assert!(labels.contains(&"Maltese".to_owned()), "{labels:?}");
        assert!(labels.contains(&"Luxembourgish".to_owned()), "{labels:?}");
    }

    #[test]
    fn autonyms_name_each_locale_in_itself() {
        let model = build(&["fr-FR", "de", "ja"]).unwrap();
        let autonyms = model
            .codes
            .iter()
            .zip(&model.autonyms)
            .map(|(code, autonym)| (code.value(), autonym.value()))
            .collect::<Vec<_>>();
        assert!(autonyms.contains(&("fr-FR".into(), "français (France)".into())));
        assert!(autonyms.contains(&("de".into(), "Deutsch".into())));
        assert!(autonyms.contains(&("ja".into(), "日本語".into())));
        assert!(autonyms.contains(&("en-US".into(), "American English".into())));
    }

    #[test]
    fn fallback_participates_in_the_sorted_unique_locale_selection() {
        let model = build(&["fr-FR", "en-US", "fr-FR"]).unwrap();
        assert_eq!(
            model.codes.iter().map(LitStr::value).collect::<Vec<_>>(),
            ["en-US", "fr-FR"]
        );
        assert_eq!(model.fallback, "EnUs");
    }

    #[test]
    #[cfg(feature = "fluent")]
    fn es_fluent_derive_is_rejected() {
        let input: ItemEnum = parse_quote!(
            #[derive(Clone, es_fluent::EsFluent)]
            enum Languages {}
        );
        let error = reject_es_fluent_derive(&input).err().unwrap();
        assert!(error.to_string().contains("remove the EsFluent derive"));
    }
}

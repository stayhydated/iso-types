#![cfg(feature = "icu")]

#[cfg(any(
    feature = "639-1",
    feature = "639-2",
    feature = "639-3",
    feature = "3166-1",
    feature = "3166-2",
    feature = "15924",
    feature = "4217",
    feature = "8601"
))]
use iso_types::VariantArray as _;

#[cfg(feature = "8601")]
use iso_types::icu_calendar;
#[cfg(any(
    feature = "639-1",
    feature = "639-2",
    feature = "639-3",
    feature = "3166-1",
    feature = "3166-2",
    feature = "15924",
    feature = "4217"
))]
use iso_types::icu_locale;

#[test]
fn subtags_round_trip_assigned_codes_and_reject_unassigned_codes() {
    macro_rules! round_trip {
        ($feature:literal, $catalog:ident, $type:path) => {
            #[cfg(feature = $feature)]
            for &value in iso_types::$catalog::VARIANTS {
                let subtag: $type = value.into();
                assert_eq!(iso_types::$catalog::try_from(subtag).unwrap(), value);
            }
        };
    }
    round_trip!("639-1", Iso6391, icu_locale::subtags::Language);
    round_trip!("639-2", Iso6392, icu_locale::subtags::Language);
    round_trip!("639-3", Iso6393, icu_locale::subtags::Language);
    round_trip!("3166-1", Iso3166_1, icu_locale::subtags::Region);
    round_trip!(
        "3166-2",
        Iso3166_2,
        icu_locale::extensions::unicode::SubdivisionId
    );
    round_trip!("15924", Iso15924, icu_locale::subtags::Script);
    round_trip!(
        "4217",
        Iso4217,
        icu_locale::preferences::extensions::unicode::keywords::CurrencyType
    );
    #[cfg(feature = "3166-1")]
    assert!(iso_types::Iso3166_1::try_from(icu_locale::subtags::region!("ZZ")).is_err());
    #[cfg(feature = "639-1")]
    assert!(iso_types::Iso6391::try_from(icu_locale::subtags::language!("zz")).is_err());
    #[cfg(feature = "4217")]
    assert!(
        iso_types::Iso4217::try_from(
            icu_locale::preferences::extensions::unicode::keywords::CurrencyType::try_from_str(
                "ZZZ"
            )
            .unwrap()
        )
        .is_err()
    );
}

#[cfg(feature = "8601")]
#[test]
fn iso_calendar_conversions_preserve_numbers_and_reject_leap_months() {
    for &value in iso_types::Iso8601Weekday::VARIANTS {
        let day: icu_calendar::types::Weekday = value.into();
        assert_eq!((day as u8).to_string(), value.code());
        assert_eq!(iso_types::Iso8601Weekday::from(day), value);
    }
    for &value in iso_types::Iso8601Month::VARIANTS {
        let month: icu_calendar::types::Month = value.into();
        assert_eq!(format!("{:02}", month.number()), value.code());
        assert_eq!(iso_types::Iso8601Month::try_from(month).unwrap(), value);
    }
    for month in [
        icu_calendar::types::Month::new(0),
        icu_calendar::types::Month::new(13),
        icu_calendar::types::Month::leap(2),
    ] {
        assert!(iso_types::Iso8601Month::try_from(month).is_err());
    }
}

#[cfg(all(feature = "639-3", feature = "15924", feature = "4217"))]
#[test]
fn items_without_icu_names_are_excluded_from_public_catalogs() {
    assert!("aaa".parse::<iso_types::Iso6393>().is_err());
    assert!("Qaaa".parse::<iso_types::Iso15924>().is_err());
    assert!("XAD".parse::<iso_types::Iso4217>().is_err());
    assert_eq!(iso_types::Iso4217::Usd.english_name(), "US dollar");
    assert_eq!(iso_types::Iso15924::Hant.english_name(), "Traditional");
    assert_eq!(iso_types::Iso6393::Eng.english_name(), "English");
    assert_eq!(
        iso_types::Iso6393::LABEL_SOURCE,
        iso_types::LabelSource::Icu4x
    );
}

#[cfg(all(feature = "fluent", feature = "locales"))]
#[test]
fn locale_module_serves_only_owned_canonical_locale_keys() {
    use es_fluent_manager_core::I18nModuleRegistration;
    let module = inventory::iter::<&'static dyn I18nModuleRegistration>()
        .find(|module| module.data().owner == "iso-types")
        .unwrap();
    assert!(!module.contributes_to_language_selection());
    let localizer = module.create_localizer(&"fr-FR".parse().unwrap()).unwrap();
    let key = |owner, id| {
        es_fluent::registry::__macro::static_message_key(
            owner,
            es_fluent::registry::__macro::static_domain("iso-types"),
            es_fluent::registry::__macro::static_entry_id(id),
        )
    };
    // The selected language names itself identically with or without `localized-langs`.
    assert_eq!(
        localizer
            .localize(key("iso-types", "iso-locale-fr-FR"), None)
            .as_deref(),
        Some("français (France)")
    );
    for id in ["iso-locale-iw-IL", "iso-639-1-en", "iso-3166-1-US"] {
        assert!(
            localizer.localize(key("iso-types", id), None).is_none(),
            "{id}"
        );
    }
    assert!(
        localizer
            .localize(key("other-owner", "iso-locale-fr-FR"), None)
            .is_none()
    );
}

#[cfg(feature = "locales")]
#[test]
fn locale_names_and_canonical_aliases_follow_icu() {
    for (input, expected) in [
        ("en-US", "American English"),
        ("fr-CA", "Canadian French"),
        ("zh-Hant-TW", "Traditional Chinese (Taiwan)"),
        ("en-001", "English (world)"),
    ] {
        assert_eq!(
            iso_types::locale_english_name(&input.parse().unwrap()).unwrap(),
            expected
        );
    }
    for (input, expected) in [("mt", "Maltese"), ("lb", "Luxembourgish"), ("la", "Latin")] {
        assert_eq!(
            iso_types::locale_english_name(&input.parse().unwrap()).unwrap(),
            expected
        );
    }
    for code in ["aaa", "en-XX"] {
        assert!(
            matches!(
                iso_types::locale_english_name(&code.parse().unwrap()),
                Err(iso_types::LocaleNameError::Unavailable { .. })
            ),
            "{code}"
        );
    }
    for (input, canonical) in [
        ("eng-US", "en-US"),
        ("fre-FR", "fr-FR"),
        ("iw-IL", "he-IL"),
        ("in", "id"),
        ("sh", "sr-Latn"),
        ("tl", "fil"),
        ("en-BU", "en-MM"),
        ("fr-fr", "fr-FR"),
    ] {
        assert_eq!(
            iso_types::parse_language_identifier(input)
                .unwrap()
                .to_string(),
            canonical,
            "{input}"
        );
    }
    assert_eq!(
        iso_types::locale_autonym(&iso_types::unic_langid::langid!("fr-FR")).unwrap(),
        "français (France)"
    );
    assert_eq!(
        iso_types::locale_display_name(
            &iso_types::unic_langid::langid!("de"),
            &iso_types::unic_langid::langid!("fr")
        )
        .unwrap(),
        "allemand"
    );
    assert!(iso_types::parse_language_identifier("invalid!").is_err());
}

#[cfg(all(
    feature = "639-1",
    feature = "639-2",
    feature = "639-3",
    feature = "15924",
    feature = "4217",
    feature = "3166-2"
))]
#[test]
fn catalogs_exclude_historical_rare_special_and_retired_data() {
    for code in ["la", "cu", "sh", "tl", "eo"] {
        assert!(code.parse::<iso_types::Iso6391>().is_err(), "{code}");
    }
    for code in ["ang", "akk", "grc", "lat", "tlh", "und", "mul", "fre"] {
        assert!(code.parse::<iso_types::Iso6392>().is_err(), "{code}");
    }
    for code in ["ang", "akk", "grc", "lat", "tlh", "und", "mul", "cnr"] {
        assert!(code.parse::<iso_types::Iso6393>().is_err(), "{code}");
    }
    for code in [
        "Egyp", "Goth", "Runr", "Glag", "Zzzz", "Zyyy", "Qaaa", "Bopo",
    ] {
        assert!(code.parse::<iso_types::Iso15924>().is_err(), "{code}");
    }
    for code in [
        "BGN", "HRK", "ZWL", "XTS", "XXX", "XAU", "XDR", "USN", "SVC",
    ] {
        assert!(code.parse::<iso_types::Iso4217>().is_err(), "{code}");
    }
    for code in ["FR-NC", "FI-01", "US-PR"] {
        assert!(code.parse::<iso_types::Iso3166_2>().is_err(), "{code}");
    }
    assert_eq!(iso_types::Iso6393::Eng.kind(), Some("L"));
    assert_eq!(iso_types::Iso4217::Zwg.numeric(), Some("924"));
    assert_eq!(iso_types::Iso15924::Hant.english_name(), "Traditional");
    assert!("US-TX".parse::<iso_types::Iso3166_2>().is_ok());
}

#[cfg(feature = "numbering-systems")]
#[test]
fn numbering_system_ids_round_trip_through_icu_nu_keywords() {
    use iso_types::icu_locale::preferences::extensions::unicode::keywords::NumberingSystem;
    use iso_types::{NumberingSystemId, VariantArray as _};
    for &system in NumberingSystemId::VARIANTS {
        let keyword = NumberingSystem::from(system);
        assert_eq!(keyword.as_str(), system.code());
        assert_eq!(NumberingSystemId::try_from(keyword).unwrap(), system);
    }
    let locale: iso_types::icu_locale::Locale = "ar-u-nu-latn".parse().unwrap();
    let keyword = NumberingSystem::try_from(
        locale
            .extensions
            .unicode
            .keywords
            .get(&iso_types::icu_locale::extensions::unicode::key!("nu"))
            .unwrap()
            .clone(),
    )
    .unwrap();
    assert_eq!(
        NumberingSystemId::try_from(keyword).unwrap(),
        NumberingSystemId::Latn
    );
}

#![cfg(any(
    feature = "639-1",
    feature = "639-2",
    feature = "639-3",
    feature = "3166-1",
    feature = "3166-2",
    feature = "4217",
    feature = "15924",
    feature = "5218",
    feature = "8601",
    feature = "units"
))]

use iso_types::{EnumCount as _, IntoEnumIterator as _, IsoCode, VariantArray as _};

fn verify<C: IsoCode + std::fmt::Debug + Into<&'static str>>() {
    assert!(!C::VARIANTS.is_empty());
    assert_eq!(C::COUNT, C::VARIANTS.len());
    assert_eq!(C::iter().collect::<Vec<_>>(), C::VARIANTS);
    let mut codes = std::collections::HashSet::new();
    for &value in C::VARIANTS {
        assert!(codes.insert(value.code()));
        assert_eq!(value.code().parse::<C>().unwrap(), value);
        assert_eq!(Into::<&'static str>::into(value), value.code());
        assert!(!value.english_name().is_empty());
    }
    for input in ["", "invalid!"] {
        assert_eq!(
            input.parse::<C>().unwrap_err(),
            iso_types::ParseIsoCodeError {
                standard: C::STANDARD,
                input: input.to_owned(),
            }
        );
    }
}

macro_rules! catalog_test {
    ($feature:literal, $name:ident, $kind:ident, $count:literal) => {
        #[cfg(feature = $feature)]
        #[test]
        fn $name() {
            const FIRST_CODE: &str = iso_types::$kind::VARIANTS[0].into_str();
            const FIRST: Option<iso_types::$kind> = iso_types::$kind::from_repr(0);
            assert_eq!(iso_types::$kind::VARIANTS.len(), $count);
            assert_eq!(FIRST_CODE, iso_types::$kind::VARIANTS[0].code());
            assert_eq!(FIRST, Some(iso_types::$kind::VARIANTS[0]));
            for (ordinal, &value) in iso_types::$kind::VARIANTS.iter().enumerate() {
                assert_eq!(iso_types::$kind::from_repr(ordinal as u16), Some(value));
                assert_eq!(iso_types::$kind::try_from(value.code()).unwrap(), value);
                assert_eq!(<&'static str>::from(&value), value.code());
            }
            assert_eq!(iso_types::$kind::from_repr($count), None);
            assert_eq!(iso_types::$kind::from_repr(u16::MAX), None);
            verify::<iso_types::$kind>();
        }
    };
}
catalog_test!("639-1", languages_1, Iso6391, 93);
catalog_test!("639-2", languages_2, Iso6392, 100);
catalog_test!("639-3", languages_3, Iso6393, 121);
catalog_test!("3166-1", countries, Iso3166_1, 249);
catalog_test!("3166-2", subdivisions, Iso3166_2, 5027);
catalog_test!("4217", currencies, Iso4217, 153);
catalog_test!("15924", scripts, Iso15924, 34);
catalog_test!("5218", sex_codes, Iso5218, 4);
catalog_test!("8601", weekdays, Iso8601Weekday, 7);
catalog_test!("8601", months, Iso8601Month, 12);
catalog_test!("units", measurement_units, MeasurementUnit, 87);

macro_rules! alias_test {
    ($feature:literal, $name:ident, $kind:ident, $($getter:ident),+) => {
        #[cfg(feature = $feature)]
        #[test]
        fn $name() {
            for &value in iso_types::$kind::VARIANTS {
                for alias in [$(value.$getter()),+].into_iter().flatten() {
                    assert_eq!(alias.parse::<iso_types::$kind>().unwrap(), value, "{alias}");
                }
            }
        }
    };
}
alias_test!("639-1", language_1_aliases, Iso6391, alpha_3);
alias_test!("639-2", language_2_aliases, Iso6392, alpha_2);
alias_test!("639-3", language_3_aliases, Iso6393, alpha_2);
alias_test!("3166-1", country_aliases, Iso3166_1, alpha_3, numeric);
alias_test!("4217", currency_aliases, Iso4217, numeric);
alias_test!("15924", script_aliases, Iso15924, numeric);

#[cfg(all(
    feature = "3166-1",
    feature = "4217",
    feature = "639-1",
    feature = "15924"
))]
#[test]
fn alternate_codes_and_metadata_preserve_standard_representations() {
    use iso_types::{Iso3166_1, Iso4217, Iso6391, Iso15924};
    assert_eq!("USA".parse::<Iso3166_1>().unwrap(), Iso3166_1::Us);
    assert_eq!("840".parse::<Iso3166_1>().unwrap(), Iso3166_1::Us);
    assert_eq!(Iso3166_1::Us.alpha_3(), Some("USA"));
    assert_eq!(Iso4217::Usd.numeric(), Some("840"));
    assert_eq!("eng".parse::<Iso6391>().unwrap(), Iso6391::En);
    assert_eq!("fra".parse::<Iso6391>().unwrap(), Iso6391::Fr);
    assert!("fre".parse::<Iso6391>().is_err());
    assert_eq!("215".parse::<Iso15924>().unwrap(), Iso15924::Latn);
    assert!("us".parse::<Iso3166_1>().is_err());
    for input in ["Us", "United States", " US", "US "] {
        assert_eq!(Iso3166_1::try_from(input).unwrap_err().input, input);
    }
    for input in ["En", "EN", "English", " en"] {
        assert!(Iso6391::try_from(input).is_err());
    }
}

#[cfg(feature = "5218")]
#[test]
fn sex_code_ordinals_follow_declaration_order() {
    use iso_types::Iso5218;
    let unspecified = "9".parse::<Iso5218>().unwrap();
    assert_eq!(Iso5218::from_repr(3), Some(unspecified));
    assert_eq!(Iso5218::from_repr(9), None);
    assert_eq!(unspecified.into_str(), "9");
}

#[cfg(all(feature = "3166-1", feature = "macros"))]
#[iso_types::iso_enum(standard = "3166-1", codes = ["US", "CA"])]
enum Markets {}

#[cfg(all(feature = "3166-1", feature = "macros"))]
const MARKETS: &[Markets] = Markets::VARIANTS;

#[cfg(all(feature = "3166-1", feature = "macros"))]
const MARKET_CODE: &str = Markets::Us.into_str();

#[cfg(all(feature = "3166-1", feature = "macros"))]
#[test]
fn declared_subset_preserves_selection_and_rejects_other_valid_codes() {
    use iso_types::Iso3166_1;
    assert_eq!(MARKETS, &[Markets::Us, Markets::Ca]);
    assert_eq!(MARKET_CODE, "US");
    assert_eq!(Markets::COUNT, 2);
    assert_eq!(Markets::iter().collect::<Vec<_>>(), MARKETS);
    assert_eq!(Markets::from_repr(0), Some(Markets::Us));
    assert_eq!(Markets::from_repr(2), None);
    assert_eq!("USA".parse::<Markets>().unwrap(), Markets::Us);
    assert!("FR".parse::<Markets>().is_err());
    assert_eq!(Markets::Ca.catalog_value(), Iso3166_1::Ca);
    assert_eq!(Markets::Us.to_string(), "United States");
}

#[cfg(feature = "fluent")]
#[test]
fn fluent_catalogs_match_every_english_display_label_and_keep_owner_scoping() {
    use es_fluent::FluentMessage;
    use es_fluent_manager_core::I18nModuleRegistration;
    let registration = inventory::iter::<&'static dyn I18nModuleRegistration>()
        .find(|m| m.data().owner == "iso-types")
        .unwrap();
    assert!(!registration.contributes_to_language_selection());
    let localizer = registration.create_localizer().unwrap();
    localizer
        .select_language(&"fr-FR".parse().unwrap())
        .unwrap();
    macro_rules! check {
        ($feature:literal, $kind:ident) => {
            #[cfg(feature = $feature)]
            for value in iso_types::$kind::VARIANTS {
                assert_eq!(
                    localizer.localize(value.fluent_key(), None).as_deref(),
                    Some(value.english_name()),
                    "{}",
                    value.code()
                );
                let label = value.english_name();
                let actual = value.to_fluent_string_with(&mut |key, _args| {
                    localizer.localize(key, None).unwrap()
                });
                assert_eq!(actual, label, "{}", value.code());
                #[cfg(feature = "fmt")]
                assert_eq!(actual, value.to_string());
            }
        };
    }
    check!("639-1", Iso6391);
    check!("639-2", Iso6392);
    check!("639-3", Iso6393);
    check!("3166-1", Iso3166_1);
    check!("3166-2", Iso3166_2);
    check!("4217", Iso4217);
    check!("15924", Iso15924);
    check!("5218", Iso5218);
    check!("8601", Iso8601Weekday);
    check!("8601", Iso8601Month);
    check!("units", MeasurementUnit);
    let foreign = es_fluent::registry::__macro::static_message_key(
        "other-owner",
        es_fluent::registry::__macro::static_domain("iso-types"),
        es_fluent::registry::__macro::static_entry_id("iso-639-1-en"),
    );
    assert!(localizer.localize(foreign, None).is_none());
}

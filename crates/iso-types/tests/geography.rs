#![cfg(any(feature = "3166-2", feature = "4217"))]

#[cfg(any(feature = "3166-2", all(feature = "3166-1", feature = "4217")))]
use iso_types::Iso3166_1;
use iso_types::VariantArray as _;

#[cfg(feature = "3166-2")]
use iso_types::{Iso3166_2, SubdivisionKind};

#[cfg(feature = "3166-2")]
const US_DIVISIONS: &[Iso3166_2] = Iso3166_1::Us.subdivisions();

#[cfg(feature = "3166-2")]
const CALIFORNIA_COUNTRY: Iso3166_1 = Iso3166_2::UsCa.country();

#[cfg(feature = "3166-2")]
const LONDON_PARENT: Option<Iso3166_2> = Iso3166_2::GbLnd.parent();

#[cfg(feature = "3166-2")]
#[test]
fn countries_supply_local_codes_and_typed_subdivision_categories() {
    assert_eq!(US_DIVISIONS.len(), 51);
    assert_eq!(CALIFORNIA_COUNTRY, Iso3166_1::Us);
    assert_eq!(Iso3166_1::Ca.subdivisions().len(), 13);
    assert_eq!(Iso3166_1::Au.subdivisions().len(), 8);
    assert_eq!(Iso3166_1::Us.subdivision("CA"), Some(Iso3166_2::UsCa));
    assert_eq!(Iso3166_1::Ca.subdivision("QC"), Some(Iso3166_2::CaQc));
    assert_eq!(Iso3166_2::UsCa.kind(), SubdivisionKind::State);
    assert_eq!(Iso3166_2::CaQc.kind(), SubdivisionKind::Province);
    assert_eq!(Iso3166_2::AuNsw.kind(), SubdivisionKind::State);
    assert_eq!(Iso3166_2::GbEng.kind(), SubdivisionKind::Country);
    assert_eq!(Iso3166_2::UsCa.local_code(), "CA");
    assert_eq!(Iso3166_1::Us.subdivision("WA"), Some(Iso3166_2::UsWa));
    assert_eq!(Iso3166_1::Au.subdivision("WA"), Some(Iso3166_2::AuWa));
    for input in ["ca", "US-CA", "INVALID"] {
        assert_eq!(Iso3166_1::Us.subdivision(input), None);
    }
    assert_eq!(Iso3166_1::Ca.subdivision("CA"), None);
}

#[cfg(feature = "3166-2")]
#[test]
fn subdivision_lists_and_hierarchy_cover_the_entire_current_catalog() {
    let mut all = std::collections::HashSet::new();
    for &country in Iso3166_1::VARIANTS {
        let subdivisions = country.subdivisions();
        assert!(
            subdivisions
                .windows(2)
                .all(|pair| pair[0].code() < pair[1].code())
        );
        for &subdivision in subdivisions {
            assert!(all.insert(subdivision));
            assert_eq!(subdivision.country(), country);
            assert_eq!(
                subdivision.code(),
                format!("{}-{}", country.code(), subdivision.local_code())
            );
        }
        let mut pending = country.root_subdivisions().to_vec();
        let mut descendants = std::collections::HashSet::new();
        for root in &pending {
            assert_eq!(root.parent(), None);
        }
        while let Some(subdivision) = pending.pop() {
            assert_eq!(subdivision.country(), country);
            assert!(
                descendants.insert(subdivision),
                "hierarchy repeats {subdivision:?}"
            );
            for &child in subdivision.children() {
                assert_eq!(child.parent(), Some(subdivision));
                pending.push(child);
            }
        }
        assert_eq!(descendants, subdivisions.iter().copied().collect());
    }
    assert_eq!(all, Iso3166_2::VARIANTS.iter().copied().collect());
    assert_eq!(LONDON_PARENT, Some(Iso3166_2::GbEng));
    assert!(Iso3166_2::GbEng.children().contains(&Iso3166_2::GbLnd));
    assert_eq!(Iso3166_2::UsCa.parent(), None);
    assert_eq!(SubdivisionKind::Province.into_str(), "Province");
}

#[cfg(all(feature = "3166-1", feature = "4217"))]
#[test]
fn country_currencies_preserve_shared_tender_and_multiple_currency_countries() {
    use iso_types::Iso4217;
    const US_CURRENCIES: &[Iso4217] = Iso3166_1::Us.currencies();
    assert_eq!(US_CURRENCIES, &[Iso4217::Usd]);
    assert_eq!(Iso3166_1::Ca.currencies(), &[Iso4217::Cad]);
    assert_eq!(Iso3166_1::Pa.currencies(), &[Iso4217::Pab, Iso4217::Usd]);
    assert_eq!(Iso3166_1::Zw.currencies(), &[Iso4217::Usd, Iso4217::Zwg]);
    assert!(Iso3166_1::Aq.currencies().is_empty());
    assert!(Iso4217::Eur.countries().contains(&Iso3166_1::De));
    assert!(Iso4217::Eur.countries().contains(&Iso3166_1::Fr));
    for &country in Iso3166_1::VARIANTS {
        let currencies = country.currencies();
        assert!(
            currencies
                .windows(2)
                .all(|pair| pair[0].code() < pair[1].code())
        );
        for &currency in currencies {
            assert!(currency.countries().contains(&country));
        }
    }
    for &currency in Iso4217::VARIANTS {
        assert!(
            currency
                .countries()
                .windows(2)
                .all(|pair| pair[0].code() < pair[1].code())
        );
        for &country in currency.countries() {
            assert!(country.currencies().contains(&currency));
        }
    }
}

#[cfg(feature = "4217")]
#[test]
fn currency_precision_and_cash_rounding_follow_icu_formatting_data() {
    use iso_types::Iso4217;
    const YEN_DIGITS: u8 = Iso4217::Jpy.fraction_digits();
    assert_eq!(Iso4217::Usd.fraction_digits(), 2);
    assert_eq!(YEN_DIGITS, 0);
    assert_eq!(Iso4217::Kwd.fraction_digits(), 3);
    assert_eq!(Iso4217::Cad.cash_fraction_digits(), 2);
    assert_eq!(Iso4217::Cad.cash_rounding_increment(), 5);
    assert_eq!(Iso4217::Chf.cash_rounding_increment(), 5);
    assert_eq!(Iso4217::Usd.rounding_increment(), 1);
    for &currency in Iso4217::VARIANTS {
        assert!(currency.fraction_digits() <= 4);
        assert!(currency.cash_fraction_digits() <= 4);
        assert!([1, 5, 20, 50].contains(&currency.rounding_increment()));
        assert!([1, 5, 20, 50].contains(&currency.cash_rounding_increment()));
    }
}

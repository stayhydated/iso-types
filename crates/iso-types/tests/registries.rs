#[cfg(feature = "numbering-systems")]
#[test]
fn numbering_preferences_preserve_locale_inheritance_and_algorithmic_systems() {
    use iso_types::{
        NumberingSystem as System, NumberingSystemKind as Kind, VariantArray as _,
        numbering_preferences,
    };
    assert_eq!(
        numbering_preferences("en-Latn-US").unwrap().default,
        System::Latn
    );
    assert_eq!(
        numbering_preferences("ar-MA").unwrap().default,
        System::Latn
    );
    assert_eq!(numbering_preferences("ar-MA").unwrap().native, System::Arab);
    assert_eq!(
        numbering_preferences("fa").unwrap().default,
        System::Arabext
    );
    assert_eq!(numbering_preferences("hi").unwrap().native, System::Deva);
    let chinese = numbering_preferences("zh-Hant-TW").unwrap();
    assert_eq!(chinese.traditional, System::Hant);
    assert_eq!(chinese.finance, System::Hantfin);
    assert_eq!(System::Latn.digits(), Some("0123456789"));
    for system in System::VARIANTS {
        match system.kind() {
            Kind::Numeric => {
                assert_eq!(system.digits().unwrap().chars().count(), 10);
                assert_eq!(system.rules(), None);
            }
            Kind::Algorithmic => {
                assert_eq!(system.digits(), None);
                assert!(!system.rules().unwrap().is_empty());
            }
        }
    }
    for invalid in ["iw-IL", "la", "en-Egyp", "en-u-nu-arab", "en--US"] {
        assert!(numbering_preferences(invalid).is_err(), "{invalid}");
    }
}

#[cfg(feature = "216")]
#[test]
fn iso_paper_sizes_have_exact_portrait_dimensions_and_registry_keywords() {
    use iso_types::{Iso216, VariantArray as _};
    assert_eq!((Iso216::A4.width_mm(), Iso216::A4.height_mm()), (210, 297));
    assert_eq!((Iso216::A0.width_mm(), Iso216::A0.height_mm()), (841, 1189));
    assert_eq!((Iso216::B5.width_mm(), Iso216::B5.height_mm()), (176, 250));
    assert_eq!(Iso216::A4.pwg_name(), "iso_a4_210x297mm");
    for paper in Iso216::VARIANTS {
        assert!(paper.width_mm() < paper.height_mm());
        assert!(paper.pwg_name().ends_with("mm"));
    }
    assert!("Letter".parse::<Iso216>().is_err());
    #[cfg(feature = "regional-preferences")]
    {
        assert_eq!(iso_types::PaperSize::A4.iso216(), Some(Iso216::A4));
        assert_eq!(iso_types::PaperSize::UsLetter.iso216(), None);
    }
}

#[cfg(feature = "639-5")]
#[test]
fn language_group_hierarchy_is_closed_and_preserves_the_published_order() {
    use iso_types::{Iso6395 as Group, LabelSource, VariantArray as _};
    assert_eq!(Group::LABEL_SOURCE, LabelSource::LibraryOfCongress);
    assert_eq!(Group::Ine.parent(), None);
    assert_eq!(Group::Gmw.parent(), Some(Group::Gem));
    assert_eq!(Group::Gmw.ancestors(), &[Group::Ine, Group::Gem]);
    assert_eq!(Group::Alg.ancestors(), &[Group::Nai, Group::Aql]);
    for &group in Group::VARIANTS {
        if let Some(parent) = group.parent() {
            assert!(parent.children().contains(&group));
        }
        for &child in group.children() {
            assert_eq!(child.parent(), Some(group));
        }
        assert!(!group.ancestors().contains(&group));
        assert_eq!(group.ancestors().last().copied(), group.parent());
    }
}

#[cfg(all(feature = "language-relations", feature = "639-3"))]
#[test]
fn macrolanguage_links_keep_individual_iso_identities() {
    use iso_types::{Iso6393 as Language, VariantArray as _};
    assert_eq!(Language::Cmn.macrolanguage(), Some(Language::Zho));
    assert_eq!(Language::Arb.macrolanguage(), Some(Language::Ara));
    assert_eq!(Language::Cmn.code(), "cmn");
    assert_eq!(Language::Zho.code(), "zho");
    for &language in Language::VARIANTS {
        if let Some(parent) = language.macrolanguage() {
            assert!(parent.members().contains(&language));
            assert_ne!(parent, language);
        }
        for &member in language.members() {
            assert_eq!(member.macrolanguage(), Some(language));
        }
    }
}

#[cfg(all(
    feature = "numbering-systems",
    feature = "216",
    feature = "639-5",
    feature = "macros"
))]
#[test]
fn registry_catalog_subsets_use_the_facade_labels() {
    use iso_types::{VariantArray as _, iso_enum};
    #[iso_enum(standard = "216", codes = ["A4", "A3"])]
    enum Paper {}
    #[iso_enum(standard = "numbering-systems", codes = ["latn", "arab"])]
    enum Digits {}
    #[iso_enum(standard = "639-5", codes = ["ine", "gem"])]
    enum Groups {}
    assert_eq!(Paper::VARIANTS, &[Paper::A4, Paper::A3]);
    assert_eq!(
        Digits::Arab.catalog_value(),
        iso_types::NumberingSystem::Arab
    );
    assert_eq!(
        Groups::Gem.catalog_value().parent(),
        Some(iso_types::Iso6395::Ine)
    );
}

#[cfg(feature = "13616")]
#[test]
fn registered_iban_examples_validate_and_territories_keep_the_registered_prefix() {
    use iso_types::{Iban, IbanFormat, Iso3166_1 as Country, ParseIbanError};
    assert_eq!(IbanFormat::ALL.len(), 88);
    for format in IbanFormat::ALL {
        let iban: Iban = format
            .electronic_example
            .parse()
            .unwrap_or_else(|error| panic!("{}: {error}", format.prefix_country.code()));
        assert_eq!(iban.as_str(), format.electronic_example);
        assert_eq!(iban.format(), format);
        assert_eq!(format.print_example.parse::<Iban>().unwrap(), iban);
        assert_eq!(iban.print_format().parse::<Iban>().unwrap(), iban);
        assert_eq!(iban.bban().len(), usize::from(format.iban_length) - 4);
        for &country in format.countries {
            assert_eq!(country.iban_format(), Some(format));
        }
    }
    assert_eq!(Country::Gg.iban_format(), Country::Gb.iban_format());
    assert_eq!(Country::Gf.iban_format(), Country::Fr.iban_format());
    assert_eq!(Country::Ax.iban_format(), Country::Fi.iban_format());
    assert_eq!(Country::Us.iban_format(), None);
    let iban: Iban = "gb82 west 1234 5698 7654 32".parse().unwrap();
    assert_eq!(iban.as_str(), "GB82WEST12345698765432");
    assert_eq!(iban.bank_identifier(), Some("WEST"));
    assert_eq!(iban.branch_identifier(), Some("123456"));
    assert_eq!(iban.country(), Country::Gb);
    assert_eq!(iban.print_format(), "GB82 WEST 1234 5698 7654 32");
    for (input, expected) in [
        ("GB00WEST12345698765432", ParseIbanError::InvalidChecksum),
        ("GB82W3ST12345698765432", ParseIbanError::InvalidStructure),
        (
            "GB82WEST1234569876543",
            ParseIbanError::InvalidLength {
                expected: 22,
                actual: 21,
            },
        ),
        ("US82WEST12345698765432", ParseIbanError::UnsupportedCountry),
        ("GF82WEST12345698765432", ParseIbanError::UnsupportedCountry),
        (
            "GB82\tWEST12345698765432",
            ParseIbanError::InvalidCharacters,
        ),
        (
            "GB８２WEST12345698765432",
            ParseIbanError::InvalidCharacters,
        ),
    ] {
        assert_eq!(input.parse::<Iban>().unwrap_err(), expected, "{input}");
    }
}

#[cfg(all(feature = "trade-units", feature = "packaging", feature = "units"))]
#[test]
fn trade_units_keep_explicit_physical_equivalences_and_package_count_codes() {
    use iso_types::{MeasurementUnit as Unit, PackagingType, TradeUnit, VariantArray as _};
    assert_eq!(TradeUnit::Kgm.catalog_unit(), Some(Unit::Kilogram));
    assert_eq!(TradeUnit::Gll.catalog_unit(), Some(Unit::Gallon));
    assert_eq!(TradeUnit::Gli.catalog_unit(), Some(Unit::GallonImperial));
    assert!(Unit::Meter.trade_codes().contains(&TradeUnit::Mtr));
    for &package in PackagingType::VARIANTS {
        let unit = package.trade_unit();
        assert_eq!(unit.packaging_type(), Some(package));
        assert_eq!(unit.code(), format!("X{}", package.code()));
        assert_eq!(unit.catalog_unit(), None);
    }
    assert_eq!(PackagingType::Bx.english_name(), "Box");
    assert_eq!(PackagingType::Bx.trade_unit(), TradeUnit::Xbx);
    #[cfg(feature = "macros")]
    {
        #[iso_types::iso_enum(standard = "trade-units", codes = ["KGM", "LTR"])]
        enum InvoiceUnits {}
        assert_eq!(
            InvoiceUnits::Kgm.catalog_value().catalog_unit(),
            Some(Unit::Kilogram)
        );
    }
}

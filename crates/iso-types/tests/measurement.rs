#![cfg(feature = "units")]

use iso_types::{MeasurementUnit, UnitCategory, VariantArray as _};

#[test]
fn common_units_have_canonical_cldr_ids_names_and_categories() {
    const METER_CATEGORY: UnitCategory = MeasurementUnit::Meter.category();
    assert_eq!(METER_CATEGORY, UnitCategory::Length);
    assert_eq!(MeasurementUnit::Kilogram.category(), UnitCategory::Mass);
    assert_eq!(
        MeasurementUnit::Celsius.category(),
        UnitCategory::Temperature
    );
    assert_eq!(MeasurementUnit::Megabyte.category(), UnitCategory::Digital);
    assert_eq!(MeasurementUnit::Meter.english_name(), "meters");
    assert_eq!(MeasurementUnit::GallonImperial.code(), "gallon-imperial");
    assert_eq!(MeasurementUnit::LABEL_SOURCE, iso_types::LabelSource::Cldr);
    assert_eq!(
        "meter".parse::<MeasurementUnit>().unwrap(),
        MeasurementUnit::Meter
    );
    for code in [
        "Meter",
        "m",
        "length-meter",
        "metric-ton",
        "furlong",
        "light-year",
        "xxx-widget",
    ] {
        assert!(code.parse::<MeasurementUnit>().is_err(), "{code}");
    }
    for &unit in MeasurementUnit::VARIANTS {
        assert!(!unit.english_name().is_empty());
        assert_eq!(unit.into_str(), unit.code());
    }
}

#[cfg(feature = "unit-conversion")]
#[test]
fn icu_conversions_cover_offsets_reciprocals_and_customary_unit_distinctions() {
    use MeasurementUnit::*;
    let cases = [
        (Meter, 1.0, Centimeter, 100.0),
        (Foot, 1.0, Meter, 0.3048),
        (Pound, 1.0, Kilogram, 0.45359237),
        (Celsius, 0.0, Fahrenheit, 32.0),
        (Fahrenheit, 212.0, Celsius, 100.0),
        (Celsius, 0.0, Kelvin, 273.15),
        (Gallon, 1.0, Liter, 3.785411784),
        (GallonImperial, 1.0, Liter, 4.54609),
        (Megabyte, 1.0, Byte, 1_000_000.0),
        (KilowattHour, 1.0, Joule, 3_600_000.0),
        (
            MilePerGallon,
            20.0,
            LiterPer100Kilometer,
            11.760729166666667,
        ),
    ];
    for (from, amount, to, expected) in cases {
        let actual = from.convert(amount, to).unwrap();
        assert!(
            (actual - expected).abs() <= 1e-9 * expected.abs().max(1.0),
            "{from:?} -> {to:?}: {actual} != {expected}"
        );
    }
    assert_eq!(
        Meter.convert(1.0, Kilogram).unwrap_err(),
        iso_types::UnitConversionError {
            from: Meter,
            to: Kilogram
        }
    );
    assert!(Ampere.convert(1.0, Volt).is_err());
    let converter = Meter.converter_to::<f64>(Foot).unwrap();
    assert!((converter.convert(2.0) - 6.561679790026246).abs() < 1e-12);
    for &unit in MeasurementUnit::VARIANTS {
        assert!(
            (unit.convert(34.75, unit).unwrap() - 34.75).abs() < 1e-10,
            "{unit:?}"
        );
        let parsed = iso_types::icu_experimental::measure::measureunit::MeasureUnit::from(unit);
        assert_eq!(&parsed, unit.icu_unit());
    }
}

#[cfg(all(feature = "macros", feature = "fmt"))]
#[iso_types::iso_enum(standard = "units", codes = ["meter", "foot"])]
enum Distances {}

#[cfg(all(feature = "macros", feature = "fmt"))]
#[test]
fn declared_units_use_the_catalog_for_labels_and_metadata() {
    assert_eq!(Distances::VARIANTS, &[Distances::Meter, Distances::Foot]);
    assert_eq!(
        Distances::Foot.catalog_value().category(),
        UnitCategory::Length
    );
    assert_eq!(Distances::Meter.to_string(), "meters");
    assert!("mile".parse::<Distances>().is_err());
}

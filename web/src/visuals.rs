mod addresses;
mod icons;
mod units;

use iso_types::{
    AddressField, AddressFieldLabel, CallingCode, GeographicRegion, Iso3166_1 as Country,
    Iso4217 as Currency, Iso5218, Iso15924, IsoCode, MeasurementUnit, PreferredUnit, UnitUsage,
    icu_locale::Direction,
};

use crate::{flag_assets::SUBDIVISION_FLAGS, i18n::Ui, site::Page};
pub(crate) use icons::Icon;

pub(crate) fn asset(path: &str) -> String {
    use stayhydated_site::routing::{asset_href, dioxus_base_href};
    asset_href(&dioxus_base_href(), path).into_string()
}

fn flag(code: &str) -> String {
    asset(&format!("assets/flags/{}.svg", code.to_ascii_lowercase()))
}

pub(crate) fn icon(symbol: Icon) -> String {
    asset(&format!("assets/icons/{}.svg", symbol.into_str()))
}

/// A theme for the catalog itself, before an individual value is selected.
pub(crate) fn standard_icon(standard: &str) -> Icon {
    match standard {
        "3166-1" | "calling-codes" => Icon::Globe,
        "3166-2" => Icon::Location,
        "4217" => Icon::Currency,
        "639-1" | "639-2" | "639-3" => Icon::Language,
        "15924" => Icon::Script,
        "5218" => Icon::Person,
        "units" | "unit-preferences" => Icon::Ruler,
        "time-zones" => Icon::Clock,
        "8601-weekday" => Icon::CalendarWeek,
        "8601-month" => Icon::CalendarMonth,
        "address-fields" | "address-labels" => Icon::Address,
        "regions" => Icon::Map,
        _ => Icon::Catalog,
    }
}

pub(crate) fn catalog_icon(standard: &str, code: &str) -> String {
    let symbol = match standard {
        "3166-1" => return flag(code),
        "3166-2"
            if SUBDIVISION_FLAGS
                .binary_search(&code.to_ascii_lowercase().as_str())
                .is_ok() =>
        {
            return flag(code);
        }
        "4217" => {
            if code == "EUR" {
                return flag("EU");
            }
            // Use a national flag only when the ISO prefix names a current
            // tender country. Regional currencies retain a currency icon.
            let country = code
                .get(..2)
                .and_then(|prefix| prefix.parse::<Country>().ok());
            let currency = code.parse::<Currency>().ok();
            match (country, currency) {
                (Some(country), Some(currency)) if currency.countries().contains(&country) => {
                    return flag(country.code());
                }
                _ => Icon::Currency,
            }
        }
        "calling-codes" => match code.parse::<CallingCode>().ok().map(|c| c.countries()) {
            Some([country]) => return flag(country.code()),
            _ => Icon::Globe,
        },
        "units" => code
            .parse::<MeasurementUnit>()
            .map_or(Icon::Ruler, units::unit_icon),
        "unit-preferences" => code
            .parse::<UnitUsage>()
            .map_or(Icon::Ruler, units::usage_icon),
        "address-fields" => code
            .parse::<AddressField>()
            .map_or(Icon::Address, addresses::field_icon),
        "address-labels" => code
            .parse::<AddressFieldLabel>()
            .map_or(Icon::Address, addresses::label_icon),
        "15924" => code.parse::<Iso15924>().map_or(Icon::Script, script_icon),
        "5218" => match code.parse::<Iso5218>() {
            Ok(Iso5218::NotKnown) => Icon::Unknown,
            Ok(Iso5218::Male) => Icon::Male,
            Ok(Iso5218::Female) => Icon::Female,
            Ok(Iso5218::NotApplicable) => Icon::NotApplicable,
            Err(_) => Icon::Person,
        },
        "regions" if code.parse::<GeographicRegion>() == Ok(GeographicRegion::World) => Icon::Globe,
        _ => standard_icon(standard),
    };
    icon(symbol)
}

pub(crate) fn for_value<T: IsoCode>(value: T) -> String {
    catalog_icon(T::STANDARD, value.code())
}

pub(crate) fn script_icon(script: Iso15924) -> Icon {
    match script.direction() {
        Some(Direction::LeftToRight) => Icon::TextLtr,
        Some(Direction::RightToLeft) => Icon::TextRtl,
        _ => Icon::Script,
    }
}

pub(crate) fn preference_icon(unit: PreferredUnit, usage: UnitUsage) -> String {
    icon(
        unit.catalog_unit()
            .map_or_else(|| units::category_icon(usage.category()), units::unit_icon),
    )
}

pub(crate) fn detail_icon(label: Ui) -> Option<Icon> {
    Some(match label {
        Ui::Script => Icon::Script,
        Ui::LanguageCode | Ui::Language => Icon::Language,
        Ui::Direction => Icon::Script,
        Ui::FirstWeekday | Ui::Weekend | Ui::MinWeekDays => Icon::CalendarWeek,
        Ui::HourCycle | Ui::ZonedTime | Ui::ZoneName | Ui::Instant => Icon::Clock,
        Ui::Calendars => Icon::CalendarMonth,
        Ui::MeasurementSystem => Icon::Ruler,
        Ui::TemperatureSystem => Icon::Thermometer,
        Ui::Paper => Icon::Paper,
        Ui::Parent | Ui::Children | Ui::Level => Icon::Hierarchy,
        Ui::Region | Ui::Kind => Icon::Map,
        Ui::CallingCode | Ui::CallingCodes => Icon::Phone,
        Ui::SharedCountries | Ui::Countries | Ui::CurrencyCountries => Icon::Globe,
        Ui::PostalExamples => Icon::Mail,
        Ui::Template => Icon::Address,
        Ui::Code => Icon::Tag,
        Ui::Digits | Ui::CashDigits | Ui::CashRounding => Icon::Currency,
        _ => return None,
    })
}

pub(crate) fn page_icon(page: Page) -> String {
    icon(match page {
        Page::Geography => Icon::Location,
        Page::Addresses => Icon::Address,
        Page::Currencies => Icon::Currency,
        Page::Languages => Icon::Language,
        Page::Units => Icon::Ruler,
        Page::Preferences => Icon::Calendar,
        Page::Regions => Icon::Globe,
        Page::TimeZones => Icon::Clock,
        Page::Phone => Icon::Phone,
        Page::Catalogs => Icon::Catalog,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_codes_and_regional_currencies_keep_their_own_identity() {
        assert!(for_value(Currency::Cad).ends_with("/flags/ca.svg"));
        assert!(for_value(Currency::Usd).ends_with("/flags/us.svg"));
        assert!(for_value(Currency::Eur).ends_with("/flags/eu.svg"));
        assert!(for_value(Currency::Xaf).ends_with("/icons/currency.svg"));
        assert!(for_value(CallingCode::C1).ends_with("/icons/globe.svg"));
        assert!(for_value(CallingCode::C800).ends_with("/icons/globe.svg"));
        assert!(for_value(CallingCode::C33).ends_with("/flags/fr.svg"));
        assert!(SUBDIVISION_FLAGS.is_sorted());
    }

    #[test]
    fn physical_quantities_and_usage_contexts_have_distinct_visuals() {
        assert!(for_value(MeasurementUnit::Volt).ends_with("/icons/bolt.svg"));
        assert_ne!(
            for_value(MeasurementUnit::Volt),
            for_value(MeasurementUnit::Ampere)
        );
        assert_ne!(
            for_value(MeasurementUnit::Volt),
            for_value(MeasurementUnit::Ohm)
        );
        assert_eq!(
            for_value(MeasurementUnit::Celsius),
            for_value(MeasurementUnit::Fahrenheit)
        );
        assert_ne!(
            for_value(MeasurementUnit::Celsius),
            for_value(MeasurementUnit::Meter)
        );
        assert_ne!(
            for_value(MeasurementUnit::Gigabyte),
            for_value(MeasurementUnit::Kilohertz)
        );
        assert_ne!(
            for_value(UnitUsage::LengthRainfall),
            for_value(UnitUsage::LengthRoad)
        );
        assert_ne!(
            for_value(AddressField::Recipient),
            for_value(AddressField::PostalCode)
        );
        assert_ne!(for_value(Iso15924::Arab), for_value(Iso15924::Latn));
        let category_icons = <iso_types::UnitCategory as iso_types::VariantArray>::VARIANTS
            .iter()
            .map(|category| units::category_icon(*category).into_str())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            category_icons.len(),
            <iso_types::UnitCategory as iso_types::EnumCount>::COUNT
        );
    }

    #[test]
    fn every_declared_pictogram_has_a_local_svg() {
        for symbol in <Icon as iso_types::VariantArray>::VARIANTS {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("public/assets/icons")
                .join(format!("{}.svg", symbol.into_str()));
            let svg = std::fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
            assert!(svg.starts_with("<svg "), "{}", path.display());
        }
    }
}

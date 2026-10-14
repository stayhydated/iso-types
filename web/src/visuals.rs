use iso_types::{CallingCode, Iso3166_1 as Country, Iso4217 as Currency, IsoCode};

use crate::{flag_assets::SUBDIVISION_FLAGS, site::Page};

pub(crate) fn asset(path: &str) -> String {
    use stayhydated_site::routing::{asset_href, dioxus_base_href};
    asset_href(&dioxus_base_href(), path).into_string()
}

fn flag(code: &str) -> String {
    asset(&format!("assets/flags/{}.svg", code.to_ascii_lowercase()))
}

pub(crate) fn icon(name: &str) -> String {
    asset(&format!("assets/icons/{name}.svg"))
}

pub(crate) fn catalog_icon(standard: &str, code: &str) -> String {
    match standard {
        "3166-1" => flag(code),
        "3166-2"
            if SUBDIVISION_FLAGS
                .binary_search(&code.to_ascii_lowercase().as_str())
                .is_ok() =>
        {
            flag(code)
        }
        "3166-2" => icon("location"),
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
                    flag(country.code())
                }
                _ => icon("currency"),
            }
        }
        "calling-codes" => match code.parse::<CallingCode>().ok().map(|c| c.countries()) {
            Some([country]) => flag(country.code()),
            _ => icon("globe"),
        },
        "639-1" | "639-2" | "639-3" | "15924" => icon("language"),
        "units" | "unit-preferences" => icon("ruler"),
        "time-zones" => icon("clock"),
        "8601-weekday" | "8601-month" => icon("calendar"),
        "address-fields" | "address-labels" => icon("address"),
        "regions" => icon("globe"),
        _ => icon("catalog"),
    }
}

pub(crate) fn for_value<T: IsoCode>(value: T) -> String {
    catalog_icon(T::STANDARD, value.code())
}

pub(crate) fn page_icon(page: Page) -> String {
    icon(match page {
        Page::Geography => "location",
        Page::Addresses => "address",
        Page::Currencies => "currency",
        Page::Languages => "language",
        Page::Units => "ruler",
        Page::Preferences => "calendar",
        Page::Regions => "globe",
        Page::TimeZones => "clock",
        Page::Phone => "phone",
        Page::Catalogs => "catalog",
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
}

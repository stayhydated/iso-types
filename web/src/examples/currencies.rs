use dioxus::prelude::*;
use es_fluent_manager_dioxus::use_i18n;
use iso_types::{IntoEnumIterator as _, Iso3166_1 as Country, Iso4217 as Currency};

use crate::{
    components::{CatalogSelect, DemoFrame, Fact, names},
    i18n::Ui,
    site::Page,
};

#[component]
pub(crate) fn Demo() -> Element {
    let i18n = use_i18n()?;
    let mut country = use_signal(|| Country::Ca);
    let mut currency = use_signal(|| Currency::Cad);
    rsx! {
        DemoFrame { page: Page::Currencies, description: Ui::CurrenciesIntro, source: include_str!("currencies.rs"),
            div { class: "control-grid",
                CatalogSelect::<Country> { label: Ui::Country, value: country, values: Country::iter().collect(),
                    on_change: move |next: Country| { country.set(next); if let Some(next) = next.currencies().first() { currency.set(*next); } },
                }
                if !country().currencies().is_empty() {
                    CatalogSelect::<Currency> { key: "{country().code()}", label: Ui::Currency, value: currency, values: country().currencies().to_vec(), on_change: move |next| currency.set(next) }
                }
            }
            if country().currencies().is_empty() {
                p { "{i18n.localize_message(&Ui::NoData)}" }
            } else {
                p { class: "country-currencies", aria_live: "polite", "{names(country().currencies())}" }
                dl { class: "fact-grid", aria_live: "polite",
                    Fact { label: Ui::Digits, value: currency().fraction_digits().to_string() }
                    Fact { label: Ui::CashDigits, value: currency().cash_fraction_digits().to_string() }
                    Fact { label: Ui::CashRounding, value: currency().cash_rounding_increment().to_string() }
                    Fact { label: Ui::CurrencyCountries, value: names(currency().countries()) }
                }
            }
            p { class: "data-note", "{i18n.localize_message(&Ui::CurrencyNote)}" }
        }
    }
}

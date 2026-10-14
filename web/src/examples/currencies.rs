use dioxus::prelude::*;
use es_fluent_manager_dioxus::use_i18n;
use iso_types::{IntoEnumIterator as _, Iso3166_1 as Country, Iso4217 as Currency};

use crate::{
    components::{CatalogFact, CatalogSelect, DemoFrame, Fact, Identity},
    i18n::Ui,
    site::Page,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Selection {
    country: Country,
    restricted: bool,
    currency: Option<Currency>,
}

impl Selection {
    fn currencies(self) -> Vec<Currency> {
        if self.restricted {
            self.country.currencies().to_vec()
        } else {
            Currency::iter().collect()
        }
    }

    fn reconcile(mut self) -> Self {
        let currencies = self.currencies();
        self.currency = self
            .currency
            .filter(|value| currencies.contains(value))
            .or_else(|| currencies.first().copied());
        self
    }
}

#[component]
pub(crate) fn Demo() -> Element {
    let i18n = use_i18n()?;
    let mut selection = use_signal(|| Selection {
        country: Country::Ca,
        restricted: true,
        currency: Some(Currency::Cad),
    });
    let country = use_memo(move || selection().country);
    let currency = use_memo(move || selection().currency.unwrap_or(Currency::Cad));
    rsx! {
        DemoFrame { page: Page::Currencies, description: Ui::CurrenciesIntro, source: include_str!("currencies.rs"),
            label { class: "scope-toggle",
                input { r#type: "checkbox", checked: selection().restricted,
                    onchange: move |event| {
                        let next = Selection { restricted: event.checked(), ..selection() }.reconcile();
                        selection.set(next);
                    },
                }
                span { "{i18n.localize_message(&Ui::RestrictToCountry)}" }
            }
            div { class: "control-grid",
                if selection().restricted {
                    CatalogSelect::<Country> { label: Ui::Country, value: country, values: Country::iter().collect(),
                        on_change: move |country| { let next = Selection { country, ..selection() }.reconcile(); selection.set(next); },
                    }
                }
                if selection().currency.is_some() {
                    CatalogSelect::<Currency> { label: Ui::Currency, value: currency, values: selection().currencies(),
                        on_change: move |next| selection.write().currency = Some(next),
                    }
                }
            }
            if !selection().restricted {
                p { class: "catalog-count", "{i18n.localize_message(&Ui::AllCurrencies)}" }
            }
            if let Some(currency) = selection().currency {
                Identity::<Currency> { value: currency, caption: i18n.localize_message(&Ui::Currency) }
                dl { class: "fact-grid", aria_live: "polite",
                    Fact { label: Ui::Digits, value: currency.fraction_digits().to_string() }
                    Fact { label: Ui::CashDigits, value: currency.cash_fraction_digits().to_string() }
                    Fact { label: Ui::CashRounding, value: currency.cash_rounding_increment().to_string() }
                    CatalogFact::<Country> { label: Ui::CurrencyCountries, values: currency.countries().to_vec() }
                }
            } else { p { "{i18n.localize_message(&Ui::NoData)}" } }
            p { class: "data-note", "{i18n.localize_message(&Ui::CurrencyNote)}" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn currency_scope_preserves_valid_choices_and_recovers_from_empty_countries() {
        let canada = Selection {
            country: Country::Ca,
            restricted: true,
            currency: Some(Currency::Cad),
        };
        let all = Selection {
            restricted: false,
            ..canada
        }
        .reconcile();
        assert_eq!(all.currency, Some(Currency::Cad));
        assert_eq!(all.currencies(), Currency::iter().collect::<Vec<_>>());
        let euro = Selection {
            currency: Some(Currency::Eur),
            ..all
        }
        .reconcile();
        assert_eq!(euro.currency, Some(Currency::Eur));
        let restricted = Selection {
            restricted: true,
            ..euro
        }
        .reconcile();
        assert_eq!(restricted.currency, Some(Currency::Cad));
        let us = Selection {
            country: Country::Us,
            ..restricted
        }
        .reconcile();
        assert_eq!(us.currency, Some(Currency::Usd));
        let antarctica = Selection {
            country: Country::Aq,
            ..us
        }
        .reconcile();
        assert_eq!(antarctica.currency, None);
        let all = Selection {
            restricted: false,
            ..antarctica
        }
        .reconcile();
        assert!(all.currency.is_some());
        assert_eq!(
            Selection {
                country: Country::Us,
                restricted: true,
                currency: Some(Currency::Usd)
            }
            .reconcile()
            .currency,
            Some(Currency::Usd)
        );
    }
}

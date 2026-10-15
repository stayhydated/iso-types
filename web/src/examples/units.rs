use dioxus::prelude::*;
use es_fluent_manager_dioxus::use_i18n;
use iso_types::{IntoEnumIterator as _, Iso3166_1 as Country, MeasurementUnit as Unit, UnitUsage};

use crate::{
    components::{CatalogFact, CatalogSelect, CountryPicker, DemoFrame},
    i18n::{QuantityResult, Ui},
    site::Page,
    visuals,
};

fn convert(input: &str, from: Unit, to: Unit) -> Result<f64, Ui> {
    let value: f64 = input.parse().map_err(|_| Ui::InvalidNumber)?;
    if !value.is_finite() {
        return Err(Ui::InvalidNumber);
    }
    let result = from.convert(value, to).map_err(|_| Ui::IncompatibleUnits)?;
    if !result.is_finite() {
        return Err(Ui::InvalidNumber);
    }
    Ok(result)
}

#[component]
pub(crate) fn Demo() -> Element {
    let i18n = use_i18n()?;
    let mut from = use_signal(|| Unit::Meter);
    let mut to = use_signal(|| Unit::Centimeter);
    let mut quantity = use_signal(|| "1".to_owned());
    let country = use_signal(|| Country::Us);
    let mut usage = use_signal(|| UnitUsage::TemperatureWeather);
    let converted = convert(&quantity(), from(), to());
    let result_icon = converted.as_ref().map_or_else(
        |_| visuals::icon(visuals::Icon::Warning),
        |_| visuals::for_value(to()),
    );
    let result = match converted {
        Ok(amount) => i18n.localize_message(&QuantityResult {
            amount,
            unit: to().code(),
        }),
        Err(message) => i18n.localize_message(&message),
    };
    rsx! {
        DemoFrame { page: Page::Units, description: Ui::UnitsIntro, source: include_str!("units.rs"),
            div { class: "control-grid",
                CatalogSelect::<Unit> { label: Ui::FromUnit, value: from, values: Unit::iter().collect(), on_change: move |next| from.set(next) }
                CatalogSelect::<Unit> { label: Ui::ToUnit, value: to, values: Unit::iter().collect(), on_change: move |next| to.set(next) }
                label { class: "catalog-control", r#for: "quantity", span { class: "control-label", "{i18n.localize_message(&Ui::Quantity)}" }
                    input { id: "quantity", r#type: "text", inputmode: "decimal", value: quantity, oninput: move |event| quantity.set(event.value()) }
                }
            }
            output { class: "conversion-result catalog-inline-value", aria_live: "polite",
                img { src: result_icon, alt: "", aria_hidden: "true", width: 24, height: 24 }
                span { "{i18n.localize_message(&Ui::Result)}: {result}" }
            }
            dl { class: "fact-grid", CatalogFact::<iso_types::TradeUnit> { label: Ui::TradeCodes, values: from().trade_codes().to_vec() } }
            div { class: "control-grid",
                CountryPicker { value: country }
                CatalogSelect::<UnitUsage> { label: Ui::Usage, value: usage, values: UnitUsage::iter().collect(), on_change: move |next| usage.set(next) }
            }
            div { class: "table-scroll", table {
                thead { tr { th { "{i18n.localize_message(&Ui::Components)}" } th { "{i18n.localize_message(&Ui::Threshold)}" } th { "{i18n.localize_message(&Ui::Skeleton)}" } } }
                tbody { for (index, preference) in country().unit_preferences(usage()).iter().enumerate() {
                    tr { key: "{index}",
                        td { div { class: "unit-components",
                            for (index, unit) in preference.units.iter().copied().enumerate() {
                                if index > 0 { span { "+" } }
                                span { class: "catalog-inline-value",
                                    img { src: visuals::preference_icon(unit, usage()), alt: "", aria_hidden: "true", width: 24, height: 24 }
                                    code { "{unit.code()}" }
                                }
                            }
                        } }
                        td { "{preference.threshold}" }
                        td { "{preference.skeleton.unwrap_or(\"—\")}" }
                    }
                } }
            } }
            p { class: "data-note", "{i18n.localize_message(&Ui::UnitNote)}" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversion_reports_user_input_and_dimension_errors() {
        assert_eq!(convert("1", Unit::Meter, Unit::Centimeter).unwrap(), 100.0);
        assert!(matches!(
            convert("NaN", Unit::Meter, Unit::Centimeter),
            Err(Ui::InvalidNumber)
        ));
        assert!(matches!(
            convert("", Unit::Meter, Unit::Centimeter),
            Err(Ui::InvalidNumber)
        ));
        assert!(matches!(
            convert("1", Unit::Meter, Unit::Celsius),
            Err(Ui::IncompatibleUnits)
        ));
    }
}

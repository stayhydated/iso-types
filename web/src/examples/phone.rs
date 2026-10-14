use dioxus::prelude::*;
use es_fluent_manager_dioxus::use_i18n;
use iso_types::{CallingCode, IntoEnumIterator as _, Iso3166_1 as Country, phonenumber};

use crate::{
    components::{CatalogSelect, CountryPicker, DemoFrame, Fact, names},
    i18n::Ui,
    site::Page,
};

fn parse_phone(country: Country, input: &str) -> Result<String, Ui> {
    let region = phonenumber::country::Id::try_from(country).map_err(|_| Ui::InvalidPhone)?;
    let phone = phonenumber::parse(Some(region), input).map_err(|_| Ui::InvalidPhone)?;
    if !phonenumber::is_valid(&phone) {
        return Err(Ui::InvalidPhone);
    }
    Ok(phone.format().mode(phonenumber::Mode::E164).to_string())
}

#[component]
pub(crate) fn Demo() -> Element {
    let i18n = use_i18n()?;
    let country = use_signal(|| Country::Us);
    let mut calling_code = use_signal(|| CallingCode::C1);
    let mut number = use_signal(|| "202-555-0123".to_owned());
    let parsed = parse_phone(country(), &number());
    rsx! {
        DemoFrame { page: Page::Phone, description: Ui::PhoneIntro, source: include_str!("phone.rs"),
            div { class: "control-grid",
                CountryPicker { value: country }
                CatalogSelect::<CallingCode> { label: Ui::CallingCode, value: calling_code, values: CallingCode::iter().collect(), on_change: move |next| calling_code.set(next) }
                label { class: "catalog-control", r#for: "phone-number", span { class: "control-label", "{i18n.localize_message(&Ui::PhoneNumber)}" }
                    input { id: "phone-number", r#type: "tel", value: number, oninput: move |event| number.set(event.value()) }
                }
            }
            dl { class: "fact-grid", aria_live: "polite",
                Fact { label: Ui::CallingCodes, value: names(country().calling_codes()) }
                Fact { label: Ui::SharedCountries, value: if calling_code().countries().is_empty() { i18n.localize_message(&Ui::GlobalCode) } else { names(calling_code().countries()) } }
                Fact { label: Ui::Result, value: parsed.unwrap_or_else(|message| i18n.localize_message(&message)) }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sample_number_uses_selected_region_and_rejects_invalid_input() {
        assert_eq!(
            parse_phone(Country::Us, "202-555-0123").unwrap(),
            "+12025550123"
        );
        assert!(matches!(
            parse_phone(Country::Us, "abc"),
            Err(Ui::InvalidPhone)
        ));
    }
}

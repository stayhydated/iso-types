use dioxus::prelude::*;
use es_fluent_manager_dioxus::use_i18n;
use iso_types::{CallingCode, IntoEnumIterator as _, Iso3166_1 as Country, phonenumber};

use crate::{
    components::{CatalogSelect, CountryFact, CountryPicker, DemoFrame, Fact, names},
    i18n::Ui,
    site::Page,
    visuals,
};

#[derive(Debug, Eq, PartialEq)]
struct ParsedPhone {
    number: String,
    country: Option<Country>,
}

fn parse_phone(country: Country, input: &str) -> Result<ParsedPhone, Ui> {
    let region = phonenumber::country::Id::try_from(country).map_err(|_| Ui::InvalidPhone)?;
    let phone = phonenumber::parse(Some(region), input).map_err(|_| Ui::InvalidPhone)?;
    if !phonenumber::is_valid(&phone) {
        return Err(Ui::InvalidPhone);
    }
    Ok(ParsedPhone {
        number: phone.format().mode(phonenumber::Mode::E164).to_string(),
        country: phone
            .country()
            .id()
            .and_then(|id| Country::try_from(id).ok()),
    })
}

#[component]
pub(crate) fn Demo() -> Element {
    let i18n = use_i18n()?;
    let country = use_signal(|| Country::Us);
    let mut calling_code = use_signal(|| CallingCode::C1);
    let mut number = use_signal(|| "202-555-0123".to_owned());
    let parsed = parse_phone(country(), &number());
    let number_icon = parsed.as_ref().ok().map_or_else(
        || visuals::icon("phone"),
        |parsed| {
            parsed
                .country
                .map_or_else(|| visuals::icon("globe"), visuals::for_value)
        },
    );
    rsx! {
        DemoFrame { page: Page::Phone, description: Ui::PhoneIntro, source: include_str!("phone.rs"),
            div { class: "control-grid",
                CountryPicker { value: country }
                CatalogSelect::<CallingCode> { label: Ui::CallingCode, value: calling_code, values: CallingCode::iter().collect(), on_change: move |next| calling_code.set(next) }
                label { class: "catalog-control", r#for: "phone-number", span { class: "control-label", "{i18n.localize_message(&Ui::PhoneNumber)}" }
                    div { class: "phone-field",
                        img { src: number_icon, alt: "", aria_hidden: "true", width: 24, height: 24 }
                        input { id: "phone-number", r#type: "tel", value: number, oninput: move |event| number.set(event.value()) }
                    }
                }
            }
            match parsed {
                Ok(parsed) => rsx! {
                    div { class: "catalog-identity phone-result", aria_live: "polite",
                        img { src: parsed.country.map_or_else(|| visuals::icon("globe"), visuals::for_value), alt: "", aria_hidden: "true", width: 56, height: 56 }
                        div { small { "{i18n.localize_message(&Ui::Result)}" } strong { "{parsed.number}" }
                            span { "{i18n.localize_message(&Ui::DetectedCountry)}: {parsed.country.map_or_else(|| i18n.localize_message(&Ui::GlobalCode), |country| country.english_name().to_owned())}" }
                        }
                    }
                },
                Err(message) => rsx! { p { role: "status", "{i18n.localize_message(&message)}" } },
            }
            dl { class: "fact-grid", aria_live: "polite",
                Fact { label: Ui::CallingCodes, value: names(country().calling_codes()) }
                if calling_code().countries().is_empty() { Fact { label: Ui::SharedCountries, value: i18n.localize_message(&Ui::GlobalCode) } }
                else { CountryFact { label: Ui::SharedCountries, values: calling_code().countries().to_vec() } }
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
            parse_phone(Country::Us, "202-555-0123").unwrap().number,
            "+12025550123"
        );
        assert_eq!(
            parse_phone(Country::Us, "+16137827274").unwrap().country,
            Some(Country::Ca)
        );
        assert_eq!(
            parse_phone(Country::Ca, "+12025550123").unwrap().country,
            Some(Country::Us)
        );
        assert_eq!(
            parse_phone(Country::Us, "+80012345678").unwrap().country,
            None
        );
        assert!(matches!(
            parse_phone(Country::Us, "abc"),
            Err(Ui::InvalidPhone)
        ));
    }
}

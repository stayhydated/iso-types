use dioxus::prelude::*;
use es_fluent_manager_dioxus::use_i18n;
use iso_types::{AddressField, Iso3166_1 as Country};

use crate::{
    components::{CountryPicker, DemoFrame, Fact},
    i18n::Ui,
    site::Page,
    visuals,
};

#[component]
pub(crate) fn Demo() -> Element {
    let i18n = use_i18n()?;
    let country = use_signal(|| Country::Ca);
    let metadata = country().address_metadata();
    rsx! {
        DemoFrame { page: Page::Addresses, description: Ui::AddressesIntro, source: include_str!("addresses.rs"),
            CountryPicker { value: country }
            p { class: "data-note", "{i18n.localize_message(&Ui::AddressNote)}" }
            if let Some(metadata) = metadata {
                div { class: "address-preview", key: "{country().code()}",
                    for field in metadata.fields.iter().copied() {
                        {
                            let label = i18n.localize_message(&metadata.field_label(field));
                            let id = format!("address-{}", field.code());
                            let required = metadata.is_required(field);
                            let requirement = i18n.localize_message(&if required { Ui::Required } else { Ui::Optional });
                            rsx! {
                                label { class: "address-field", r#for: id.clone(),
                                    span {
                                        img { src: visuals::for_value(metadata.field_label(field)), alt: "", aria_hidden: "true", width: 20, height: 20 }
                                        "{label}" small { "{requirement}" }
                                    }
                                    input { id, r#type: "text", required, autocomplete: "off",
                                        placeholder: if field == AddressField::PostalCode { metadata.postal_code_examples.first().copied().unwrap_or("") } else { "" },
                                    }
                                }
                            }
                        }
                    }
                }
                dl { class: "fact-grid",
                    Fact { label: Ui::Template, value: metadata.format.to_owned() }
                    Fact { label: Ui::PostalExamples, value: metadata.postal_code_examples.join(", ") }
                }
            } else { p { "{i18n.localize_message(&Ui::NoData)}" } }
        }
    }
}

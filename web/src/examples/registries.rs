use dioxus::prelude::*;
use es_fluent_manager_dioxus::use_i18n;
use iso_types::{Iban, Iso10383, MarketRegistry, MicDate};
use iso_types_locations::UnLocode;

use crate::{
    components::{DemoFrame, Fact},
    i18n::Ui,
    site::Page,
};

#[component]
pub(crate) fn Demo() -> Element {
    let i18n = use_i18n()?;
    let mut iban = use_signal(|| "GB82 WEST 1234 5698 7654 32".to_owned());
    let mut location = use_signal(|| "FIHEL".to_owned());
    let mut csv = use_signal(String::new);
    let mut date = use_signal(|| "2026-10-15".to_owned());
    let mut mic = use_signal(String::new);
    let account = iban().parse::<Iban>();
    let listings = location().parse::<UnLocode>();
    let markets = date().parse::<MicDate>().ok().and_then(|date| {
        (!csv().is_empty()).then(|| MarketRegistry::from_csv(csv().as_bytes(), date))
    });
    let market = markets
        .as_ref()
        .and_then(|result| result.as_ref().ok())
        .and_then(|registry| {
            mic()
                .parse::<Iso10383>()
                .ok()
                .and_then(|code| registry.get(code))
        });
    rsx! {
        DemoFrame { page: Page::Registries, description: Ui::RegistriesIntro, source: include_str!("registries.rs"),
            section {
                h3 { "{i18n.localize_message(&Ui::Iban)}" }
                label { class: "catalog-control", r#for: "iban", span { class: "control-label", "{i18n.localize_message(&Ui::Iban)}" }
                    input { id: "iban", value: iban, oninput: move |event| iban.set(event.value()) }
                }
                if let Ok(account) = account {
                    dl { class: "fact-grid", aria_live: "polite",
                        Fact { label: Ui::Code, value: account.print_format() }
                        Fact { label: Ui::Country, value: account.country().english_name().to_owned() }
                        Fact { label: Ui::BankIdentifier, value: account.bank_identifier().unwrap_or("—").to_owned() }
                        Fact { label: Ui::BranchIdentifier, value: account.branch_identifier().unwrap_or("—").to_owned() }
                    }
                } else { p { role: "status", "{i18n.localize_message(&Ui::InvalidIban)}" } }
                p { class: "data-note", "{i18n.localize_message(&Ui::IbanNote)}" }
            }
            section {
                h3 { "UN/LOCODE" }
                label { class: "catalog-control", r#for: "locode", span { class: "control-label", "{i18n.localize_message(&Ui::LocationCode)}" }
                    input { id: "locode", value: location, oninput: move |event| location.set(event.value()) }
                }
                if let Ok(location) = listings {
                    div { class: "table-scroll", aria_live: "polite", table {
                        thead { tr {
                            th { "{i18n.localize_message(&Ui::LocationName)}" }
                            th { "{i18n.localize_message(&Ui::Subdivision)}" }
                            th { "{i18n.localize_message(&Ui::Functions)}" }
                            th { "{i18n.localize_message(&Ui::Status)}" }
                            th { "{i18n.localize_message(&Ui::Coordinates)}" }
                        } }
                        tbody { for (index, record) in location.records().enumerate() {
                            tr { key: "{index}",
                                td { "{record.name}" }
                                td { "{record.subdivision().map_or(record.subdivision_code.unwrap_or(\"—\"), |value| value.english_name())}" }
                                td { code { "{record.function_code}" } }
                                td { "{record.status.map_or(\"—\", |value| value.code())}" }
                                td { "{record.coordinate_text.unwrap_or(\"—\")}" }
                            }
                        } }
                    } }
                } else { p { role: "status", "{i18n.localize_message(&Ui::UnknownCode)}" } }
                p { class: "data-note", "{i18n.localize_message(&Ui::LocationNote)}" }
            }
            section {
                h3 { "{i18n.localize_message(&Ui::MicRegistry)}" }
                p { class: "data-note", "{i18n.localize_message(&Ui::MicNote)}" }
                label { class: "catalog-control", r#for: "mic-csv", span { class: "control-label", "{i18n.localize_message(&Ui::MicCsv)}" }
                    textarea { id: "mic-csv", rows: "6", spellcheck: "false", value: csv, oninput: move |event| csv.set(event.value()) }
                }
                div { class: "control-grid",
                    label { class: "catalog-control", r#for: "mic-date", span { class: "control-label", "{i18n.localize_message(&Ui::AsOf)}" }
                        input { id: "mic-date", r#type: "date", value: date, oninput: move |event| date.set(event.value()) }
                    }
                    label { class: "catalog-control", r#for: "mic-code", span { class: "control-label", "MIC" }
                        input { id: "mic-code", value: mic, oninput: move |event| mic.set(event.value()) }
                    }
                }
                if let Some(Err(error)) = &markets { p { role: "status", "{error}" } }
                else if date().parse::<MicDate>().is_err() { p { role: "status", "{i18n.localize_message(&Ui::InvalidDate)}" } }
                if let Some(record) = market {
                    dl { class: "fact-grid", aria_live: "polite",
                        Fact { label: Ui::MarketName, value: record.name().to_owned() }
                        Fact { label: Ui::Country, value: record.country.english_name().to_owned() }
                        Fact { label: Ui::OperatingMic, value: record.operating_mic.to_string() }
                        Fact { label: Ui::Status, value: record.field("STATUS").unwrap_or("—").to_owned() }
                    }
                } else if matches!(markets, Some(Ok(_))) { p { role: "status", "{i18n.localize_message(&Ui::UnknownCode)}" } }
            }
        }
    }
}

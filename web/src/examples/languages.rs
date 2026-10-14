use dioxus::prelude::*;
use es_fluent_manager_dioxus::use_i18n;
use iso_types::{
    IntoEnumIterator as _, Iso3166_1 as Country, Iso6391 as Language, Iso15924 as Script,
};

use crate::{
    components::{CatalogSelect, CountryPicker, DemoFrame, Fact, names},
    i18n::Ui,
    site::Page,
};

#[component]
pub(crate) fn Demo() -> Element {
    let i18n = use_i18n()?;
    let country = use_signal(|| Country::Ca);
    let mut language = use_signal(|| Language::En);
    let mut script = use_signal(|| Script::Latn);
    rsx! {
        DemoFrame { page: Page::Languages, description: Ui::LanguagesIntro, source: include_str!("languages.rs"),
            div { class: "control-grid",
                CountryPicker { value: country }
                CatalogSelect::<Language> { label: Ui::LanguageCode, value: language, values: Language::iter().collect(), on_change: move |next| language.set(next) }
                CatalogSelect::<Script> { label: Ui::Script, value: script, values: Script::iter().collect(), on_change: move |next| script.set(next) }
            }
            dl { class: "fact-grid", aria_live: "polite",
                Fact { label: Ui::Script, value: names(language().scripts()) }
                Fact { label: Ui::Countries, value: names(language().countries()) }
                Fact { label: Ui::Direction, value: format!("{:?}", script().direction()) }
                Fact { label: Ui::LanguageCode, value: names(script().languages()) }
            }
            div { class: "table-scroll", table {
                thead { tr { th { "{i18n.localize_message(&Ui::LanguageCode)}" } th { "{i18n.localize_message(&Ui::Population)}" } th { "{i18n.localize_message(&Ui::Status)}" } } }
                tbody { for usage in country().language_usage() {
                    tr { key: "{usage.locale}", td { "{usage.locale} · {usage.language.english_name()}" } td { "{usage.population_percent}" } td { "{usage.status.map_or(\"—\", |status| status.into_str())}" } }
                } }
            } }
            p { class: "data-note", "{i18n.localize_message(&Ui::LanguageNote)}" }
        }
    }
}

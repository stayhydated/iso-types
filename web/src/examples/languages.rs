use dioxus::prelude::*;
use es_fluent_manager_dioxus::use_i18n;
use iso_types::{
    IntoEnumIterator as _, Iso3166_1 as Country, Iso6391 as Language, Iso6395 as Group,
    Iso15924 as Script,
};

use crate::{
    components::{CatalogFact, CatalogSelect, CountryPicker, DemoFrame, Fact},
    i18n::Ui,
    site::Page,
    visuals,
};

#[component]
pub(crate) fn Demo() -> Element {
    let i18n = use_i18n()?;
    let country = use_signal(|| Country::Ca);
    let mut language = use_signal(|| Language::En);
    let mut script = use_signal(|| Script::Latn);
    let mut group = use_signal(|| Group::Ine);
    let direction = match script().direction() {
        Some(iso_types::icu_locale::Direction::LeftToRight) => Ui::LeftToRight,
        Some(iso_types::icu_locale::Direction::RightToLeft) => Ui::RightToLeft,
        _ => Ui::None,
    };
    rsx! {
        DemoFrame { page: Page::Languages, description: Ui::LanguagesIntro, source: include_str!("languages.rs"),
            div { class: "control-grid",
                CountryPicker { value: country }
                CatalogSelect::<Language> { label: Ui::LanguageCode, value: language, values: Language::iter().collect(), on_change: move |next| language.set(next) }
                CatalogSelect::<Script> { label: Ui::Script, value: script, values: Script::iter().collect(), on_change: move |next| script.set(next) }
            }
            dl { class: "fact-grid", aria_live: "polite",
                Fact { label: Ui::Direction, value: i18n.localize_message(&direction), symbol: visuals::script_icon(script()) }
                CatalogFact::<Script> { label: Ui::Script, values: language().scripts().to_vec() }
                CatalogFact::<Language> { label: Ui::LanguageCode, values: script().languages().to_vec() }
                CatalogFact::<Country> { label: Ui::Countries, values: language().countries().to_vec() }
                CatalogFact::<Language> { label: Ui::Macrolanguage, values: language().macrolanguage().into_iter().collect() }
                CatalogFact::<Language> { label: Ui::Members, values: language().members().to_vec() }
            }
            CatalogSelect::<Group> { label: Ui::LanguageGroup, value: group, values: Group::iter().collect(), on_change: move |next| group.set(next) }
            dl { class: "fact-grid", aria_live: "polite",
                CatalogFact::<Group> { label: Ui::Parent, values: group().parent().into_iter().collect() }
                CatalogFact::<Group> { label: Ui::Children, values: group().children().to_vec() }
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

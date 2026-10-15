use dioxus::prelude::*;
use es_fluent_manager_dioxus::use_i18n;
use iso_types::{IntoEnumIterator as _, Iso3166_1 as Country, Iso8601Weekday};

use crate::{
    components::{CatalogFact, CountryPicker, DemoFrame, Fact},
    i18n::Ui,
    site::Page,
};

#[component]
pub(crate) fn Demo() -> Element {
    let i18n = use_i18n()?;
    let country = use_signal(|| Country::Us);
    let mut locale = use_signal(|| "ar-MA".to_owned());
    let numbering = iso_types::numbering_preferences(&locale());
    let prefs = country().regional_preferences();
    let weekend = Iso8601Weekday::iter()
        .filter(|day| prefs.week.is_weekend(*day))
        .map(|day| i18n.localize_message(&day))
        .collect::<Vec<_>>()
        .join(", ");
    rsx! {
        DemoFrame { page: Page::Preferences, description: Ui::PreferencesIntro, source: include_str!("preferences.rs"),
            CountryPicker { value: country }
            dl { class: "fact-grid", aria_live: "polite",
                Fact { label: Ui::FirstWeekday, value: i18n.localize_message(&prefs.week.first_weekday) }
                Fact { label: Ui::Weekend, value: weekend }
                Fact { label: Ui::MinWeekDays, value: prefs.week.min_week_days.to_string() }
                Fact { label: Ui::HourCycle, value: format!("{:?}", prefs.hour_cycle) }
                Fact { label: Ui::Calendars, value: format!("{:?}", prefs.preferred_calendars) }
                Fact { label: Ui::MeasurementSystem, value: format!("{:?}", prefs.measurement_system) }
                Fact { label: Ui::TemperatureSystem, value: format!("{:?}", prefs.temperature_measurement_system) }
                Fact { label: Ui::Paper, value: prefs.paper_size.iso216().map_or_else(|| prefs.paper_size.into_str().to_owned(), |paper| format!("{} · {} × {} mm", paper.code(), paper.width_mm(), paper.height_mm())) }
            }
            label { class: "catalog-control", r#for: "numbering-locale", span { class: "control-label", "{i18n.localize_message(&Ui::Locale)}" }
                input { id: "numbering-locale", value: locale, oninput: move |event| locale.set(event.value()) }
            }
            if let Ok(prefs) = numbering {
                dl { class: "fact-grid", aria_live: "polite",
                    CatalogFact::<iso_types::NumberingSystem> { label: Ui::DefaultNumbering, values: vec![prefs.default] }
                    CatalogFact::<iso_types::NumberingSystem> { label: Ui::NativeNumbering, values: vec![prefs.native] }
                    CatalogFact::<iso_types::NumberingSystem> { label: Ui::TraditionalNumbering, values: vec![prefs.traditional] }
                    CatalogFact::<iso_types::NumberingSystem> { label: Ui::FinanceNumbering, values: vec![prefs.finance] }
                }
            } else { p { role: "status", "{i18n.localize_message(&Ui::InvalidLocale)}" } }
        }
    }
}

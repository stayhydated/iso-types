use dioxus::prelude::*;
use es_fluent_manager_dioxus::use_i18n;
use iso_types::{IntoEnumIterator as _, Iso3166_1 as Country, Iso8601Weekday};

use crate::{
    components::{CountryPicker, DemoFrame, Fact},
    i18n::Ui,
    site::Page,
};

#[component]
pub(crate) fn Demo() -> Element {
    let i18n = use_i18n()?;
    let country = use_signal(|| Country::Us);
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
                Fact { label: Ui::Paper, value: prefs.paper_size.into_str().to_owned() }
            }
        }
    }
}

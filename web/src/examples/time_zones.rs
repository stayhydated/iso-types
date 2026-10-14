use dioxus::prelude::*;
use es_fluent_manager_dioxus::use_i18n;
use iso_types::{
    IntoEnumIterator as _, Iso3166_1 as Country, TimeZoneId, icu_locale::Locale, jiff,
};

use crate::{
    components::{CatalogSelect, DemoFrame, Fact},
    i18n::Ui,
    site::Page,
};

fn at_instant(zone: TimeZoneId, input: &str) -> Result<String, Ui> {
    let instant = input
        .parse::<jiff::Timestamp>()
        .map_err(|_| Ui::InvalidInstant)?;
    zone.at(instant)
        .map(|time| time.to_string())
        .map_err(|_| Ui::ZoneUnavailable)
}

#[component]
pub(crate) fn Demo() -> Element {
    let i18n = use_i18n()?;
    let mut country = use_signal(|| Country::Us);
    let mut zone = use_signal(|| TimeZoneId::AmericaNewYork);
    let mut instant = use_signal(|| "2026-07-01T12:00:00Z".to_owned());
    let locale: Locale = i18n.requested_language().to_string().parse()?;
    let localized = zone()
        .localized_name(&locale)
        .map_err(|_| Ui::ZoneUnavailable);
    let local = at_instant(zone(), &instant());
    rsx! {
        DemoFrame { page: Page::TimeZones, description: Ui::TimeZonesIntro, source: include_str!("time_zones.rs"),
            div { class: "control-grid",
                CatalogSelect::<Country> { label: Ui::Country, value: country, values: Country::iter().collect(), on_change: move |next: Country| {
                    country.set(next); zone.set(next.time_zones().first().copied().unwrap_or(TimeZoneId::EtcUtc));
                } }
                if !country().time_zones().is_empty() {
                    CatalogSelect::<TimeZoneId> { key: "{country().code()}", label: Ui::TimeZone, value: zone,
                        values: country().time_zones().to_vec(), on_change: move |next| zone.set(next),
                    }
                } else {
                    p { "{i18n.localize_message(&Ui::NoData)}" }
                }
            }
            label { class: "catalog-control", r#for: "instant", span { class: "control-label", "{i18n.localize_message(&Ui::Instant)}" }
                input { id: "instant", r#type: "text", value: instant, oninput: move |event| instant.set(event.value()) }
            }
            if !country().time_zones().is_empty() {
                dl { class: "fact-grid", aria_live: "polite",
                    Fact { label: Ui::ZonedTime, value: local.unwrap_or_else(|message| i18n.localize_message(&message)) }
                    Fact { label: Ui::ZoneName, value: localized.unwrap_or_else(|message| i18n.localize_message(&message)) }
                    Fact { label: Ui::Code, value: zone().bcp47().unwrap_or("—").to_owned() }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instant_conversion_observes_dst_and_reports_invalid_input() {
        assert!(
            at_instant(TimeZoneId::AmericaNewYork, "2026-01-01T12:00:00Z")
                .unwrap()
                .contains("07:00:00-05:00")
        );
        assert!(
            at_instant(TimeZoneId::AmericaNewYork, "2026-07-01T12:00:00Z")
                .unwrap()
                .contains("08:00:00-04:00")
        );
        assert!(matches!(
            at_instant(TimeZoneId::AmericaNewYork, "invalid"),
            Err(Ui::InvalidInstant)
        ));
    }
}

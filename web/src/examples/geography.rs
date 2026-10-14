use dioxus::prelude::*;
use es_fluent_manager_dioxus::use_i18n;
use iso_types::{IntoEnumIterator as _, Iso3166_1 as Country, Iso3166_2 as Subdivision};

use crate::{
    components::{CatalogFact, CatalogSelect, DemoFrame, Fact, Identity},
    i18n::Ui,
    site::Page,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Selection {
    country: Country,
    subdivision: Option<Subdivision>,
}

impl Selection {
    fn with_country(self, country: Country) -> Self {
        let subdivision = self
            .subdivision
            .filter(|area| area.country() == country)
            .or_else(|| country.root_subdivisions().first().copied());
        Self {
            country,
            subdivision,
        }
    }
}

#[component]
pub(crate) fn Demo() -> Element {
    let i18n = use_i18n()?;
    let mut selection = use_signal(|| Selection {
        country: Country::Ca,
        subdivision: Some(Subdivision::CaOn),
    });
    let country = use_memo(move || selection().country);
    let subdivision = use_memo(move || selection().subdivision);
    let selected_area = use_memo(move || selection().subdivision.unwrap_or(Subdivision::CaOn));
    rsx! {
        DemoFrame { page: Page::Geography, description: Ui::GeographyIntro, source: include_str!("geography.rs"),
            div { class: "control-grid",
                CatalogSelect::<Country> { label: Ui::Country, value: country, values: Country::iter().collect(),
                    on_change: move |next| { let next = selection().with_country(next); selection.set(next); },
                }
                if subdivision().is_some() {
                    CatalogSelect::<Subdivision> { key: "{country().code()}", label: Ui::Subdivision,
                        value: selected_area, values: country().subdivisions().to_vec(),
                        on_change: move |next| selection.write().subdivision = Some(next),
                    }
                }
            }
            if let Some(area) = subdivision() {
                div { class: "identity-grid", aria_live: "polite",
                    Identity::<Country> { value: country(), caption: i18n.localize_message(&Ui::Country) }
                    Identity::<Subdivision> { value: area, caption: format!("{} · {}", country().english_name(), area.kind().into_str()) }
                }
                dl { class: "fact-grid", aria_live: "polite",
                    Fact { label: Ui::Code, value: area.code().to_owned() }
                    Fact { label: Ui::Kind, value: area.kind().into_str().to_owned() }
                    CatalogFact::<Subdivision> { label: Ui::Parent, values: area.parent().into_iter().collect() }
                    CatalogFact::<Subdivision> { label: Ui::Children, values: area.children().to_vec() }
                }
            } else { p { "{i18n.localize_message(&Ui::NoData)}" } }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn country_switch_clears_foreign_subdivision_and_handles_empty_catalog() {
        let canada = Selection {
            country: Country::Ca,
            subdivision: Some(Subdivision::CaOn),
        };
        assert_eq!(canada.with_country(Country::Ca), canada);
        let us = canada.with_country(Country::Us);
        assert_eq!(us.subdivision.unwrap().country(), Country::Us);
        let antarctica = us.with_country(Country::Aq);
        assert_eq!(antarctica.subdivision, None);
        assert_eq!(
            antarctica
                .with_country(Country::Ca)
                .subdivision
                .unwrap()
                .country(),
            Country::Ca
        );
    }
}

use dioxus::prelude::*;
use es_fluent_manager_dioxus::use_i18n;
use iso_types::{GeographicRegion as Region, IntoEnumIterator as _, Iso3166_1 as Country};

use crate::{
    components::{CatalogSelect, CountryChips, CountryPicker, DemoFrame, Fact, names},
    i18n::Ui,
    site::Page,
};

#[component]
pub(crate) fn Demo() -> Element {
    let i18n = use_i18n()?;
    let mut region = use_signal(|| Region::World);
    let country = use_signal(|| Country::Ca);
    let none = i18n.localize_message(&Ui::None);
    rsx! {
        DemoFrame { page: Page::Regions, description: Ui::RegionsIntro, source: include_str!("regions.rs"),
            div { class: "control-grid",
                CatalogSelect::<Region> { label: Ui::Region, value: region, values: Region::iter().collect(), on_change: move |next| region.set(next) }
                CountryPicker { value: country }
            }
            dl { class: "fact-grid", aria_live: "polite",
                Fact { label: Ui::Level, value: format!("{:?}", region().level()) }
                Fact { label: Ui::Parent, value: region().parent().map_or(none, |parent| names(&[parent])) }
                Fact { label: Ui::Children, value: names(region().children()) }
                Fact { label: Ui::Region, value: names(country().geographic_regions()) }
            }
            section { class: "region-countries", h4 { "{i18n.localize_message(&Ui::Countries)}" }
                CountryChips { values: region().countries().to_vec() }
            }
        }
    }
}

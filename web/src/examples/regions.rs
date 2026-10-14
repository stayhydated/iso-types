use dioxus::prelude::*;
use es_fluent_manager_dioxus::use_i18n;
use iso_types::{GeographicRegion as Region, IntoEnumIterator as _, Iso3166_1 as Country};

use crate::{
    components::{CatalogChips, CatalogFact, CatalogSelect, CountryPicker, DemoFrame, Fact},
    i18n::Ui,
    site::Page,
};

#[component]
pub(crate) fn Demo() -> Element {
    let i18n = use_i18n()?;
    let mut region = use_signal(|| Region::World);
    let country = use_signal(|| Country::Ca);
    rsx! {
        DemoFrame { page: Page::Regions, description: Ui::RegionsIntro, source: include_str!("regions.rs"),
            div { class: "control-grid",
                CatalogSelect::<Region> { label: Ui::Region, value: region, values: Region::iter().collect(), on_change: move |next| region.set(next) }
                CountryPicker { value: country }
            }
            dl { class: "fact-grid", aria_live: "polite",
                Fact { label: Ui::Level, value: format!("{:?}", region().level()) }
                CatalogFact::<Region> { label: Ui::Parent, values: region().parent().into_iter().collect() }
                CatalogFact::<Region> { label: Ui::Children, values: region().children().to_vec() }
                CatalogFact::<Region> { label: Ui::Region, values: country().geographic_regions().to_vec() }
            }
            section { class: "region-countries", h4 { "{i18n.localize_message(&Ui::Countries)}" }
                CatalogChips::<Country> { values: region().countries().to_vec() }
            }
        }
    }
}

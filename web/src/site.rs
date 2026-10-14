use dioxus::prelude::*;
use es_fluent::EsFluent;
use es_fluent_manager_dioxus::{DioxusAssetI18nProvider, use_i18n};
use iso_types::VariantArray as _;
use stayhydated_dioxus::{
    DemoGallery, DemoGalleryColumns, DemoGalleryItem, NavigationTarget, Project, ProjectSite,
    StayhydatedProjectApp, StayhydatedProjectPageMetadata, StayhydatedProjectSitePortal,
};

use crate::{
    components::ExampleShell,
    examples,
    i18n::{DemoLanguage, Ui},
};

pub(crate) const PROJECT: Project = Project::new("iso-types", "Typed codes. Connected data.");
pub(crate) const VERSION: &str = env!("CARGO_PKG_VERSION");

pub(crate) fn site() -> ProjectSite {
    ProjectSite::builder()
        .project(PROJECT)
        .site_url("https://stayhydated.github.io/iso-types/")
        .rustdoc_url("https://github.com/stayhydated/iso-types/tree/master/crates/iso-types")
        .source_url("https://github.com/stayhydated/iso-types")
        .version(VERSION)
        .site_stylesheet_path("assets/site.css")
        .build()
}

#[component]
pub fn App() -> Element {
    rsx! {
        DioxusAssetI18nProvider {
            initial_language: DemoLanguage::default().into(),
            StayhydatedProjectApp::<Route> { site: site() }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, EsFluent, strum::VariantArray)]
pub(crate) enum Page {
    Geography,
    Addresses,
    Currencies,
    Languages,
    Units,
    Preferences,
    Regions,
    TimeZones,
    Phone,
    Catalogs,
}

impl Page {
    pub(crate) fn route(self) -> Route {
        match self {
            Self::Geography => Route::Geography {},
            Self::Addresses => Route::Addresses {},
            Self::Currencies => Route::Currencies {},
            Self::Languages => Route::Languages {},
            Self::Units => Route::Units {},
            Self::Preferences => Route::Preferences {},
            Self::Regions => Route::Regions {},
            Self::TimeZones => Route::TimeZones {},
            Self::Phone => Route::Phone {},
            Self::Catalogs => Route::Catalogs {},
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Routable)]
pub(crate) enum Route {
    #[route("/")]
    Home {},
    #[route("/examples/")]
    Gallery {},
    #[route("/examples/geography/")]
    Geography {},
    #[route("/examples/addresses/")]
    Addresses {},
    #[route("/examples/currencies/")]
    Currencies {},
    #[route("/examples/languages/")]
    Languages {},
    #[route("/examples/units/")]
    Units {},
    #[route("/examples/preferences/")]
    Preferences {},
    #[route("/examples/regions/")]
    Regions {},
    #[route("/examples/time-zones/")]
    TimeZones {},
    #[route("/examples/phone/")]
    Phone {},
    #[route("/examples/catalogs/")]
    Catalogs {},
}

#[component]
fn Home() -> Element {
    rsx! {
        StayhydatedProjectPageMetadata { project: PROJECT, page_title: "Home", description: "Typed ISO and regional catalogs for Rust, with interactive Dioxus examples." }
        StayhydatedProjectSitePortal::<Route> {
            site: site(), home: NavigationTarget::Internal(Route::Home {}),
            demos: NavigationTarget::Internal(Route::Gallery {}),
        }
    }
}

#[component]
fn Gallery() -> Element {
    let i18n = use_i18n()?;
    let items = Page::VARIANTS
        .iter()
        .map(|page| {
            DemoGalleryItem::route(
                page.route(),
                i18n.localize_message(page),
                format!("example-{page:?}"),
            )
        })
        .collect();
    rsx! {
        ExampleShell { title: Ui::Examples, description: Ui::Intro,
            DemoGallery::<Route> { items, columns: DemoGalleryColumns::Three }
        }
    }
}

#[component]
fn Geography() -> Element {
    rsx! { examples::geography::Demo {} }
}
#[component]
fn Addresses() -> Element {
    rsx! { examples::addresses::Demo {} }
}
#[component]
fn Currencies() -> Element {
    rsx! { examples::currencies::Demo {} }
}
#[component]
fn Languages() -> Element {
    rsx! { examples::languages::Demo {} }
}
#[component]
fn Units() -> Element {
    rsx! { examples::units::Demo {} }
}
#[component]
fn Preferences() -> Element {
    rsx! { examples::preferences::Demo {} }
}
#[component]
fn Regions() -> Element {
    rsx! { examples::regions::Demo {} }
}
#[component]
fn TimeZones() -> Element {
    rsx! { examples::time_zones::Demo {} }
}
#[component]
fn Phone() -> Element {
    rsx! { examples::phone::Demo {} }
}
#[component]
fn Catalogs() -> Element {
    rsx! { examples::catalogs::Demo {} }
}

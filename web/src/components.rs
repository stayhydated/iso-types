use dioxus::prelude::*;
use es_fluent_manager_dioxus::use_i18n;
use iso_types::{IntoEnumIterator as _, Iso3166_1, IsoCode};
use stayhydated_dioxus::{
    CodeBlock, NavigationTarget, StayhydatedProjectPageMetadata, StayhydatedProjectPortalShell,
    select::{SearchableSelect, SelectItem},
};

use crate::{
    i18n::{DemoLanguage, Ui},
    site::{PROJECT, Page, Route, VERSION},
};

#[component]
pub(crate) fn ExampleShell(
    title: Ui,
    description: Ui,
    page: Option<Page>,
    children: Element,
) -> Element {
    let i18n = use_i18n()?;
    let heading = page.map_or_else(
        || i18n.localize_message(&title),
        |page| i18n.localize_message(&page),
    );
    let description = i18n.localize_message(&description);
    let selected = use_memo({
        let i18n = i18n.clone();
        move || Some(DemoLanguage::try_from(i18n.requested_language()).unwrap_or_default())
    });
    let languages = DemoLanguage::iter()
        .map(|lang| SelectItem::new(lang, i18n.localize_message(&lang)))
        .collect();
    let locale = i18n.requested_language().to_string();
    let switch = i18n.clone();
    rsx! {
        StayhydatedProjectPageMetadata { project: PROJECT, page_title: heading.clone(), description: description.clone() }
        StayhydatedProjectPortalShell { project: PROJECT, version: VERSION, home: NavigationTarget::Internal(Route::Home {}),
            div { class: "catalog-workbench", lang: locale,
                nav { class: "example-toolbar", aria_label: i18n.localize_message(&Ui::Examples),
                    Link { to: Route::Gallery {}, "← {i18n.localize_message(&Ui::Examples)}" }
                    div { class: "locale-picker",
                        SearchableSelect::<DemoLanguage> {
                            items: languages, value: Some(selected.into()),
                            aria_label: i18n.localize_message(&Ui::Language),
                            placeholder: i18n.localize_message(&Ui::Search), empty_message: i18n.localize_message(&Ui::NoMatches),
                            on_value_change: move |next: Option<DemoLanguage>| { if let Some(next) = next { let _ = switch.select_language(next); } },
                        }
                    }
                }
                header { class: "example-heading", h2 { "{heading}" } p { "{description}" } }
                {children}
                p { class: "data-note", "{i18n.localize_message(&Ui::EnglishLabels)}" }
            }
        }
    }
}

#[component]
pub(crate) fn DemoFrame(
    page: Page,
    description: Ui,
    source: &'static str,
    children: Element,
) -> Element {
    let i18n = use_i18n()?;
    rsx! {
        ExampleShell { title: Ui::Examples, description, page,
            div { class: "example-content", {children} }
            details { class: "example-source", summary { "{i18n.localize_message(&Ui::Source)}" } CodeBlock { code: source } }
        }
    }
}

#[component]
pub(crate) fn CatalogSelect<T: IsoCode + 'static>(
    label: Ui,
    value: ReadSignal<T>,
    values: Vec<T>,
    on_change: EventHandler<T>,
) -> Element {
    let i18n = use_i18n()?;
    let selected = use_memo(move || Some(value()));
    let items = values
        .into_iter()
        .map(|v| SelectItem::new(v, format!("{} · {}", v.code(), v.english_name())))
        .collect();
    rsx! {
        div { class: "catalog-control",
            span { class: "control-label", "{i18n.localize_message(&label)}" }
            SearchableSelect::<T> {
                items, value: Some(selected.into()), aria_label: i18n.localize_message(&label),
                placeholder: i18n.localize_message(&Ui::Search), empty_message: i18n.localize_message(&Ui::NoMatches),
                on_value_change: move |next| { if let Some(next) = next { on_change.call(next); } },
            }
        }
    }
}

#[component]
pub(crate) fn CountryPicker(value: Signal<Iso3166_1>) -> Element {
    let mut value = value;
    rsx! { CatalogSelect::<Iso3166_1> { label: Ui::Country, value, values: Iso3166_1::iter().collect(), on_change: move |next| value.set(next) } }
}

#[component]
pub(crate) fn Fact(label: Ui, value: String) -> Element {
    let i18n = use_i18n()?;
    rsx! { div { class: "catalog-fact", dt { "{i18n.localize_message(&label)}" } dd { "{value}" } } }
}

pub(crate) fn names<T: IsoCode>(values: &[T]) -> String {
    values
        .iter()
        .map(|v| format!("{} · {}", v.code(), v.english_name()))
        .collect::<Vec<_>>()
        .join(", ")
}

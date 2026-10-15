use dioxus::prelude::*;
use es_fluent_manager_dioxus::use_i18n;
use iso_types::{
    AddressField, AddressFieldLabel, CallingCode, EnumCount as _, GeographicRegion, Iso216,
    Iso3166_1, Iso3166_2, Iso4217, Iso5218, Iso6391, Iso6392, Iso6393, Iso6395, Iso8601Month,
    Iso8601Weekday, Iso15924, IsoCode, MeasurementUnit, NumberingSystem, PackagingType,
    ParseIsoCodeError, TimeZoneId, TradeUnit, UnitUsage, VariantArray as _, iso_enum,
};
use stayhydated_dioxus::select::{SearchableSelect, SelectItem};

use crate::{
    components::{DemoFrame, names},
    i18n::{CatalogCount, Ui},
    site::Page,
    visuals,
};

#[derive(Clone, Copy, Debug, PartialEq, strum::VariantArray)]
enum Catalog {
    Language1,
    Language2,
    Language3,
    Countries,
    Subdivisions,
    Currencies,
    Scripts,
    SexCodes,
    Weekdays,
    Months,
    Units,
    Regions,
    TimeZones,
    UnitUsages,
    CallingCodes,
    AddressFields,
    AddressLabels,
    Paper,
    LanguageGroups,
    NumberingSystems,
    TradeUnits,
    Packaging,
}

#[derive(Debug, Eq, PartialEq)]
struct Entry {
    code: &'static str,
    name: &'static str,
}

fn entry<T: IsoCode>(value: T) -> Entry {
    Entry {
        code: value.code(),
        name: value.english_name(),
    }
}
fn entries<T: IsoCode>() -> Vec<Entry> {
    T::iter().map(entry).collect()
}
fn parsed<T: IsoCode>(input: &str) -> Result<Entry, ParseIsoCodeError> {
    input.parse::<T>().map(entry)
}

impl Catalog {
    fn standard(self) -> &'static str {
        match self {
            Self::Language1 => Iso6391::STANDARD,
            Self::Language2 => Iso6392::STANDARD,
            Self::Language3 => Iso6393::STANDARD,
            Self::Countries => Iso3166_1::STANDARD,
            Self::Subdivisions => Iso3166_2::STANDARD,
            Self::Currencies => Iso4217::STANDARD,
            Self::Scripts => Iso15924::STANDARD,
            Self::SexCodes => Iso5218::STANDARD,
            Self::Weekdays => Iso8601Weekday::STANDARD,
            Self::Months => Iso8601Month::STANDARD,
            Self::Units => MeasurementUnit::STANDARD,
            Self::Regions => GeographicRegion::STANDARD,
            Self::TimeZones => TimeZoneId::STANDARD,
            Self::UnitUsages => UnitUsage::STANDARD,
            Self::CallingCodes => CallingCode::STANDARD,
            Self::AddressFields => AddressField::STANDARD,
            Self::AddressLabels => AddressFieldLabel::STANDARD,
            Self::Packaging => PackagingType::STANDARD,
            Self::TradeUnits => TradeUnit::STANDARD,
            Self::NumberingSystems => NumberingSystem::STANDARD,
            Self::LanguageGroups => Iso6395::STANDARD,
            Self::Paper => Iso216::STANDARD,
        }
    }
    fn entries(self) -> Vec<Entry> {
        match self {
            Self::Language1 => entries::<Iso6391>(),
            Self::Language2 => entries::<Iso6392>(),
            Self::Language3 => entries::<Iso6393>(),
            Self::Countries => entries::<Iso3166_1>(),
            Self::Subdivisions => entries::<Iso3166_2>(),
            Self::Currencies => entries::<Iso4217>(),
            Self::Scripts => entries::<Iso15924>(),
            Self::SexCodes => entries::<Iso5218>(),
            Self::Weekdays => entries::<Iso8601Weekday>(),
            Self::Months => entries::<Iso8601Month>(),
            Self::Units => entries::<MeasurementUnit>(),
            Self::Regions => entries::<GeographicRegion>(),
            Self::TimeZones => entries::<TimeZoneId>(),
            Self::UnitUsages => entries::<UnitUsage>(),
            Self::CallingCodes => entries::<CallingCode>(),
            Self::AddressFields => entries::<AddressField>(),
            Self::AddressLabels => entries::<AddressFieldLabel>(),
            Self::Packaging => entries::<PackagingType>(),
            Self::TradeUnits => entries::<TradeUnit>(),
            Self::NumberingSystems => entries::<NumberingSystem>(),
            Self::LanguageGroups => entries::<Iso6395>(),
            Self::Paper => entries::<Iso216>(),
        }
    }
    fn parse(self, input: &str) -> Result<Entry, ParseIsoCodeError> {
        match self {
            Self::Language1 => parsed::<Iso6391>(input),
            Self::Language2 => parsed::<Iso6392>(input),
            Self::Language3 => parsed::<Iso6393>(input),
            Self::Countries => parsed::<Iso3166_1>(input),
            Self::Subdivisions => parsed::<Iso3166_2>(input),
            Self::Currencies => parsed::<Iso4217>(input),
            Self::Scripts => parsed::<Iso15924>(input),
            Self::SexCodes => parsed::<Iso5218>(input),
            Self::Weekdays => parsed::<Iso8601Weekday>(input),
            Self::Months => parsed::<Iso8601Month>(input),
            Self::Units => parsed::<MeasurementUnit>(input),
            Self::Regions => parsed::<GeographicRegion>(input),
            Self::TimeZones => parsed::<TimeZoneId>(input),
            Self::UnitUsages => parsed::<UnitUsage>(input),
            Self::CallingCodes => parsed::<CallingCode>(input),
            Self::AddressFields => parsed::<AddressField>(input),
            Self::AddressLabels => parsed::<AddressFieldLabel>(input),
            Self::Packaging => parsed::<PackagingType>(input),
            Self::TradeUnits => parsed::<TradeUnit>(input),
            Self::NumberingSystems => parsed::<NumberingSystem>(input),
            Self::LanguageGroups => parsed::<Iso6395>(input),
            Self::Paper => parsed::<Iso216>(input),
        }
    }
}

#[iso_enum(standard = "3166-1", codes = ["CA", "US"])]
enum Market {}
#[iso_enum(standard = "3166-2", country = "US")]
#[allow(
    clippy::enum_variant_names,
    reason = "Subdivision variants retain the canonical ISO country prefix."
)]
enum UsAreas {}
#[iso_enum(standard = "units", category = "length")]
enum LengthUnits {}

#[component]
pub(crate) fn Demo() -> Element {
    let i18n = use_i18n()?;
    let mut catalog = use_signal(|| Catalog::Countries);
    let selected = use_memo(move || Some(catalog()));
    let mut filter = use_signal(String::new);
    let mut input = use_signal(|| "USA".to_owned());
    let query = filter().to_lowercase();
    let entries = catalog().entries();
    let count = entries.len();
    let filtered = entries
        .into_iter()
        .filter(|v| {
            v.code.to_lowercase().contains(&query) || v.name.to_lowercase().contains(&query)
        })
        .collect::<Vec<_>>();
    let count_label = i18n.localize_message(&CatalogCount {
        visible: filtered.len() as i64,
        total: count as i64,
    });
    let parsed = catalog().parse(&input());
    let result_icon = parsed.as_ref().map_or_else(
        |_| visuals::icon(visuals::Icon::Unknown),
        |value| visuals::catalog_icon(catalog().standard(), value.code),
    );
    let result = parsed
        .map(|v| format!("{} · {}", v.code, v.name))
        .unwrap_or_else(|_| i18n.localize_message(&Ui::UnknownCode));
    let items = Catalog::VARIANTS
        .iter()
        .map(|v| {
            SelectItem::new(*v, v.standard())
                .with_icon(visuals::icon(visuals::standard_icon(v.standard())))
        })
        .collect();
    let market = Market::VARIANTS
        .iter()
        .map(|v| {
            format!(
                "{} · {} → {}",
                v.code(),
                i18n.localize_message(v),
                names(v.catalog_value().currencies())
            )
        })
        .collect::<Vec<_>>()
        .join("; ");
    rsx! {
        DemoFrame { page: Page::Catalogs, description: Ui::CatalogsIntro, source: include_str!("catalogs.rs"),
            div { class: "control-grid",
                div { class: "catalog-control",
                    span { class: "control-label", "{i18n.localize_message(&Ui::Catalog)}" }
                    SearchableSelect::<Catalog> {
                        items, value: Some(selected.into()), aria_label: i18n.localize_message(&Ui::Catalog),
                        placeholder: i18n.localize_message(&Ui::Search), empty_message: i18n.localize_message(&Ui::NoMatches),
                        on_value_change: move |next| { if let Some(next) = next { catalog.set(next); filter.set(String::new()); input.set(String::new()); } },
                    }
                }
                label { class: "catalog-control", r#for: "parse-code", span { class: "control-label", "{i18n.localize_message(&Ui::ParseInput)}" }
                    input { id: "parse-code", value: input, oninput: move |event| input.set(event.value()) }
                }
            }
            output { class: "conversion-result catalog-inline-value", aria_live: "polite",
                img { src: result_icon, alt: "", aria_hidden: "true", width: 24, height: 24 }
                span { "{result}" }
            }
            label { class: "catalog-control", r#for: "filter", span { class: "control-label", "{i18n.localize_message(&Ui::Filter)}" }
                input { id: "filter", value: filter, oninput: move |event| filter.set(event.value()) }
            }
            p { class: "catalog-count", aria_live: "polite", "{count_label}" }
            div { class: "table-scroll catalog-table", table {
                thead { tr { th { "{i18n.localize_message(&Ui::Code)}" } th { "{i18n.localize_message(&Ui::EnglishName)}" } } }
                tbody { for value in filtered {
                    tr { key: "{value.code}", td { code { "{value.code}" } } td {
                        span { class: "catalog-entry-name",
                            img { src: visuals::catalog_icon(catalog().standard(), value.code), alt: "", aria_hidden: "true", width: 24, height: 24, loading: "lazy", decoding: "async" }
                            "{value.name}"
                        }
                    } }
                } }
            } }
            section { class: "subset-results", h4 { "{i18n.localize_message(&Ui::Subset)}" }
                p { "Market: {market}" }
                p { "US: {UsAreas::COUNT}; length: {LengthUnits::COUNT}" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_catalog_round_trips_and_subsets_keep_runtime_relationships() {
        assert_eq!(Catalog::VARIANTS.len(), 22);
        for catalog in Catalog::VARIANTS {
            assert_ne!(
                visuals::standard_icon(catalog.standard()),
                visuals::Icon::Catalog
            );
            let entries = catalog.entries();
            assert!(!entries.is_empty());
            for entry in entries {
                assert_eq!(catalog.parse(entry.code).unwrap(), entry);
            }
        }
        assert_eq!(Market::Ca.catalog_value(), Iso3166_1::Ca);
        assert_eq!(Market::Us.catalog_value().currencies(), &[Iso4217::Usd]);
        assert_eq!(UsAreas::COUNT, 51);
        assert!(
            LengthUnits::VARIANTS
                .iter()
                .all(|v| v.catalog_value().category() == iso_types::UnitCategory::Length)
        );
    }

    #[test]
    fn every_catalog_value_resolves_to_a_shipped_visual_asset() {
        let mut paths = std::collections::BTreeSet::new();
        for catalog in Catalog::VARIANTS {
            for entry in catalog.entries() {
                let path = visuals::catalog_icon(catalog.standard(), entry.code);
                paths.insert(path);
            }
        }
        for path in paths {
            let relative = path.strip_prefix('/').expect("root-relative asset URL");
            let asset = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("public")
                .join(relative);
            assert!(asset.is_file(), "{}", asset.display());
        }
    }
}

//! English Fluent catalogs registered as a runtime utility module.
use es_fluent_manager_core::{
    FluentArgumentMap, I18nModule, I18nModuleDescriptor, I18nModuleRegistration, LocalizationError,
    Localizer, ModuleData, ModuleDomain, StaticFluentMessageKey,
};
use fluent_bundle::{FluentResource, concurrent::FluentBundle};
use std::sync::OnceLock;

mod resources;
use resources::{ENGLISH_RESOURCES, catalog_english_name};

static DATA: ModuleData = ModuleData {
    name: "iso-types",
    owner: es_fluent_manager_core::__macro::static_domain("iso-types"),
    supported_languages: &[],
    domains: &[ModuleDomain {
        domain: es_fluent_manager_core::__macro::static_domain("iso-types"),
        namespaces: &[],
    }],
};

struct IsoModule;
static MODULE: IsoModule = IsoModule;
inventory::submit! { &MODULE as &dyn I18nModuleRegistration }

impl I18nModuleDescriptor for IsoModule {
    fn data(&self) -> &'static ModuleData {
        &DATA
    }
}
impl I18nModule for IsoModule {
    fn create_localizer(&self) -> Box<dyn Localizer> {
        Box::new(EnglishLocalizer)
    }
    fn contributes_to_language_selection(&self) -> bool {
        false
    }
}
struct EnglishLocalizer;
fn bundle() -> &'static FluentBundle<FluentResource> {
    static BUNDLE: OnceLock<FluentBundle<FluentResource>> = OnceLock::new();
    BUNDLE.get_or_init(|| {
        let mut bundle = FluentBundle::new_concurrent(vec!["en".parse().expect("English locale")]);
        bundle.set_use_isolating(false);
        for source in ENGLISH_RESOURCES {
            let resource =
                FluentResource::try_new((*source).to_owned()).expect("validated ISO English FTL");
            bundle
                .add_resource(resource)
                .expect("unique ISO catalog message IDs");
        }
        bundle
    })
}
impl Localizer for EnglishLocalizer {
    fn select_language(
        &self,
        _: &es_fluent::unic_langid::LanguageIdentifier,
    ) -> Result<(), LocalizationError> {
        Ok(())
    }
    fn localize<'a>(
        &self,
        key: StaticFluentMessageKey,
        _: Option<&FluentArgumentMap<'a>>,
    ) -> Option<String> {
        if key.owner() != DATA.owner || !DATA.owns_domain(key.domain()) {
            return None;
        }
        #[cfg(feature = "locales")]
        if let Some(locale) = key.id().as_str().strip_prefix("iso-locale-") {
            return locale
                .parse()
                .ok()
                .and_then(|locale| crate::locale_english_name(&locale).ok());
        }
        if let Some(label) = catalog_english_name(key.id().as_str()) {
            return Some(label.to_owned());
        }
        let bundle = bundle();
        let pattern = bundle.get_message(key.id().as_str())?.value()?;
        let mut errors = Vec::new();
        let text = bundle.format_pattern(pattern, None, &mut errors);
        errors.is_empty().then(|| text.into_owned())
    }
}

/// Keep the catalog's inventory registration linked in optimized applications.
#[inline(never)]
pub fn force_link() -> usize {
    let module: &'static dyn I18nModuleRegistration = &MODULE;
    usize::from(!module.data().owner.as_str().is_empty())
}
/// Marker used to verify optimized WASM demo artifacts.
pub const WASM_FORCE_LINK_MARKER: &str = "iso-types-wasm-force-link";
#[cfg(target_arch = "wasm32")]
#[used]
static FORCE_LINK: fn() -> usize = force_link;
#[cfg(target_arch = "wasm32")]
#[used]
static MARKER: [u8; WASM_FORCE_LINK_MARKER.len()] = *b"iso-types-wasm-force-link";

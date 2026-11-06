//! Locale names for `#[iso_locale]` labels, registered as a runtime utility module.
use es_fluent::unic_langid::LanguageIdentifier;
use es_fluent_manager_core::{
    FluentArgumentMap, I18nModule, I18nModuleDescriptor, I18nModuleRegistration, LocalizationError,
    Localizer, ModuleData, ModuleDomain, StaticFluentMessageKey,
};

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
    fn create_localizer(
        &self,
        lang: &LanguageIdentifier,
    ) -> Result<Box<dyn Localizer>, LocalizationError> {
        Ok(Box::new(IsoLocalizer::new(lang)))
    }
    fn contributes_to_language_selection(&self) -> bool {
        false
    }
}

/// Locale names for `iso-locale-*` keys.
struct IsoLocalizer {
    locale_names: LocaleNames,
}

impl IsoLocalizer {
    fn new(lang: &LanguageIdentifier) -> Self {
        Self {
            locale_names: LocaleNames::new(lang),
        }
    }
}

/// Locale names cached per target: autonyms, or names in the selected
/// language with `localized-langs`.
struct LocaleNames {
    #[cfg(feature = "localized-langs")]
    display: LanguageIdentifier,
    cache: std::sync::RwLock<std::collections::HashMap<String, Option<String>>>,
}

impl LocaleNames {
    #[cfg_attr(not(feature = "localized-langs"), allow(unused_variables))]
    fn new(display: &LanguageIdentifier) -> Self {
        Self {
            #[cfg(feature = "localized-langs")]
            display: display.clone(),
            cache: Default::default(),
        }
    }

    fn get(&self, target: &str) -> Option<String> {
        use std::sync::PoisonError;
        if let Some(name) = self
            .cache
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .get(target)
        {
            return name.clone();
        }
        let name = self.format(target);
        self.cache
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .entry(target.to_owned())
            .or_insert(name)
            .clone()
    }

    /// Fall back to the English name when ICU lacks a component.
    fn format(&self, target: &str) -> Option<String> {
        let target = target.parse().ok()?;
        // Generated keys are canonical; other spellings are not iso-types messages.
        if crate::canonicalize_language_identifier(&target).to_string() != target.to_string() {
            return None;
        }
        #[cfg(feature = "localized-langs")]
        let name = crate::locale_display_name(&target, &self.display);
        #[cfg(not(feature = "localized-langs"))]
        let name = crate::locale_autonym(&target);
        name.or_else(|_| crate::locale_english_name(&target)).ok()
    }
}

impl Localizer for IsoLocalizer {
    fn localize<'a>(
        &self,
        key: StaticFluentMessageKey,
        _: Option<&FluentArgumentMap<'a>>,
    ) -> Option<String> {
        if key.owner() != DATA.owner || !DATA.owns_domain(key.domain()) {
            return None;
        }
        let locale = key.id().as_str().strip_prefix("iso-locale-")?;
        self.locale_names.get(locale)
    }
}

/// Keep the locale-name module's inventory registration linked in optimized applications.
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

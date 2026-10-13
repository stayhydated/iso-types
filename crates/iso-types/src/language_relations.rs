mod data;
pub use data::{LanguageStatus, LanguageUsage};

impl crate::Iso15924 {
    /// Resolve text direction through ICU4X's script data.
    pub fn direction(self) -> Option<icu_locale::Direction> {
        let script: icu_locale::subtags::Script = self.into();
        icu_locale::LocaleDirectionality::new_extended()
            .get(&icu_locale::LanguageIdentifier::from(Some(script)))
    }
}

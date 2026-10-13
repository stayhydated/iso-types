//! Real consumer coverage for renamed dependencies and configured locale enums.
#[codes::iso_locale]
pub enum Languages {}

#[codes::iso_locale(custom)]
pub enum CustomLabels {}

#[codes::iso_enum(standard = "639-1")]
pub enum AllLanguages {}

#[cfg(test)]
mod tests {
    use super::*;
    use es_fluent::FluentMessage;
    use es_fluent_manager_core::I18nModuleRegistration;

    #[test]
    fn renamed_facade_supports_both_macros_and_locale_conversions() {
        assert_eq!(Languages::default(), Languages::EnUs);
        assert_eq!(Languages::ALL.len(), 4);
        assert_eq!("fr-fr".parse::<Languages>().unwrap(), Languages::FrFr);
        assert_eq!("eng-US".parse::<Languages>().unwrap(), Languages::EnUs);
        assert_eq!(
            codes::icu_locale::LanguageIdentifier::from(Languages::FrFr),
            codes::icu_locale::langid!("fr-FR")
        );
        assert_eq!(
            Languages::try_from(codes::icu_locale::langid!("eng-US")).unwrap(),
            Languages::EnUs
        );
        assert_eq!(
            codes::LanguageIdentifier::from(&Languages::ZhHantTw),
            codes::langid!("zh-Hant-TW")
        );
        assert_eq!(
            Languages::try_from(codes::langid!("en-US")).unwrap(),
            Languages::EnUs
        );
        assert_eq!(
            Languages::try_from(&codes::langid!("fr-FR")).unwrap(),
            Languages::FrFr
        );
        assert!(matches!(
            "invalid!".parse::<Languages>(),
            Err(codes::LocaleConversionError::Invalid { .. })
        ));
        assert!(matches!(
            "de".parse::<Languages>(),
            Err(codes::LocaleConversionError::Unsupported(_))
        ));
        for code in ["iw-IL", "sh", "en-BU", "de-DE-1901", "la", "en-Goth"] {
            assert!(
                matches!(
                    code.parse::<Languages>(),
                    Err(codes::LocaleConversionError::OutOfScope(_))
                ),
                "{code}"
            );
        }
        assert_eq!(AllLanguages::ALL.len(), 93);
        assert_eq!("eng".parse::<AllLanguages>().unwrap(), AllLanguages::En);
        assert_eq!(
            Languages::ZhHantTw.to_string(),
            "Traditional Chinese (Taiwan)"
        );
    }

    #[test]
    fn builtin_locale_fluent_and_display_agree_for_every_configured_locale() {
        let module = inventory::iter::<&'static dyn I18nModuleRegistration>()
            .find(|m| m.data().owner == "iso-types")
            .unwrap();
        let localizer = module.create_localizer().unwrap();
        localizer.select_language(&codes::langid!("fr-FR")).unwrap();
        for locale in Languages::ALL {
            let actual =
                locale.to_fluent_string_with(&mut |key, _| localizer.localize(key, None).unwrap());
            assert_eq!(actual, locale.to_string());
        }
    }

    #[test]
    fn custom_locale_inventory_belongs_to_the_consumer_and_uses_exact_locale_keys() {
        let info = inventory::iter::<es_fluent::registry::RegisteredFtlType>()
            .find(|entry| entry.0.type_name() == "CustomLabels")
            .unwrap()
            .0;
        assert_eq!(info.variants().len(), 4);
        let mut key = None;
        CustomLabels::FrFr.to_fluent_string_with(&mut |actual, _| {
            key = Some(actual);
            "French label".to_owned()
        });
        let key = key.unwrap();
        assert_eq!(key.owner(), "iso-types-locale-consumer");
        assert_eq!(key.domain(), "iso-types-locale-consumer");
        assert_eq!(key.id(), "fr-FR");
    }
}

#[cfg(test)]
mod hygiene {
    // Application names must not capture paths emitted by either macro.
    mod core {}
    mod std {}
    type String = ();
    type Result = ();

    #[codes::iso_enum(standard = "639-1", codes = ["en", "fr"])]
    pub enum PickerLanguages {}

    #[codes::iso_locale]
    pub enum ConfiguredLanguages {}

    #[codes::iso_enum(standard = "639-1", codes = ["en"])]
    #[cfg(any())]
    pub enum DisabledLanguages {}

    #[test]
    fn application_names_do_not_capture_generated_traits_or_types() {
        let _: (String, Result) = ((), ());
        assert_eq!(
            "en".parse::<PickerLanguages>().unwrap(),
            PickerLanguages::En
        );
        assert_eq!(PickerLanguages::Fr.to_string(), "French");
        assert_eq!(ConfiguredLanguages::default(), ConfiguredLanguages::EnUs);
        assert_eq!(
            "eng-US".parse::<ConfiguredLanguages>().unwrap(),
            ConfiguredLanguages::EnUs
        );
        assert_eq!(
            codes::LanguageIdentifier::from(ConfiguredLanguages::EnUs),
            codes::langid!("en-US")
        );
        let module =
            inventory::iter::<&'static dyn es_fluent_manager_core::I18nModuleRegistration>()
                .find(|entry| entry.data().owner == "iso-types")
                .unwrap();
        let localizer = module.create_localizer().unwrap();
        assert_eq!(
            es_fluent::FluentMessage::to_fluent_string_with(&PickerLanguages::En, &mut |key, _| {
                localizer.localize(key, None).unwrap()
            }),
            "English"
        );
    }
}

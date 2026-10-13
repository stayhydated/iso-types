pub(crate) const ENGLISH_RESOURCES: &[&str] = &[
    #[cfg(feature = "639-1")]
    include_str!("../i18n/en/iso-types/639-1.ftl"),
    #[cfg(feature = "639-2")]
    include_str!("../i18n/en/iso-types/639-2.ftl"),
    #[cfg(feature = "639-3")]
    include_str!("../i18n/en/iso-types/639-3.ftl"),
    #[cfg(feature = "639-5")]
    include_str!("../i18n/en/iso-types/639-5.ftl"),
    #[cfg(feature = "3166-1")]
    include_str!("../i18n/en/iso-types/3166-1.ftl"),
    #[cfg(feature = "3166-2")]
    include_str!("../i18n/en/iso-types/3166-2.ftl"),
    #[cfg(feature = "3166-3")]
    include_str!("../i18n/en/iso-types/3166-3.ftl"),
    #[cfg(feature = "4217")]
    include_str!("../i18n/en/iso-types/4217.ftl"),
    #[cfg(feature = "15924")]
    include_str!("../i18n/en/iso-types/15924.ftl"),
    #[cfg(feature = "5218")]
    include_str!("../i18n/en/iso-types/5218.ftl"),
    #[cfg(feature = "8601")]
    include_str!("../i18n/en/iso-types/8601-weekday.ftl"),
    #[cfg(feature = "8601")]
    include_str!("../i18n/en/iso-types/8601-month.ftl"),
];

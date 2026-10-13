//! Conversions preserve assigned ISO codes; ICU's syntax types can also represent unassigned codes.

macro_rules! subtag_conversion {
    ($feature:literal, $catalog:ident, $subtag:path) => {
        #[cfg(feature = $feature)]
        impl From<crate::$catalog> for $subtag {
            fn from(value: crate::$catalog) -> Self {
                value
                    .code()
                    .parse()
                    .expect("assigned ISO code has ICU subtag syntax")
            }
        }
        #[cfg(feature = $feature)]
        impl TryFrom<$subtag> for crate::$catalog {
            type Error = crate::ParseIsoCodeError;
            fn try_from(value: $subtag) -> Result<Self, Self::Error> {
                value.as_str().parse()
            }
        }
    };
}
subtag_conversion!("639-1", Iso6391, icu_locale::subtags::Language);
subtag_conversion!("639-2", Iso6392, icu_locale::subtags::Language);
subtag_conversion!("639-3", Iso6393, icu_locale::subtags::Language);
subtag_conversion!("3166-1", Iso3166_1, icu_locale::subtags::Region);
subtag_conversion!("15924", Iso15924, icu_locale::subtags::Script);

#[cfg(feature = "3166-2")]
impl From<crate::Iso3166_2> for icu_locale::extensions::unicode::SubdivisionId {
    fn from(value: crate::Iso3166_2) -> Self {
        value
            .code()
            .replace('-', "")
            .to_ascii_lowercase()
            .parse()
            .expect("assigned ISO subdivision has ICU subdivision syntax")
    }
}
#[cfg(feature = "3166-2")]
impl TryFrom<icu_locale::extensions::unicode::SubdivisionId> for crate::Iso3166_2 {
    type Error = crate::ParseIsoCodeError;
    fn try_from(
        value: icu_locale::extensions::unicode::SubdivisionId,
    ) -> Result<Self, Self::Error> {
        format!(
            "{}-{}",
            value.region,
            value.suffix.as_str().to_ascii_uppercase()
        )
        .parse()
    }
}
#[cfg(feature = "4217")]
impl From<crate::Iso4217> for icu_locale::preferences::extensions::unicode::keywords::CurrencyType {
    fn from(value: crate::Iso4217) -> Self {
        Self::try_from_str(value.code()).expect("assigned ISO currency has ICU currency syntax")
    }
}
#[cfg(feature = "4217")]
impl TryFrom<icu_locale::preferences::extensions::unicode::keywords::CurrencyType>
    for crate::Iso4217
{
    type Error = crate::ParseIsoCodeError;
    fn try_from(
        value: icu_locale::preferences::extensions::unicode::keywords::CurrencyType,
    ) -> Result<Self, Self::Error> {
        value.iso_code().as_str().parse()
    }
}

#[cfg(feature = "8601")]
impl From<crate::Iso8601Weekday> for icu_calendar::types::Weekday {
    fn from(value: crate::Iso8601Weekday) -> Self {
        use crate::Iso8601Weekday::*;
        match value {
            Monday => Self::Monday,
            Tuesday => Self::Tuesday,
            Wednesday => Self::Wednesday,
            Thursday => Self::Thursday,
            Friday => Self::Friday,
            Saturday => Self::Saturday,
            Sunday => Self::Sunday,
        }
    }
}
#[cfg(feature = "8601")]
impl From<icu_calendar::types::Weekday> for crate::Iso8601Weekday {
    fn from(value: icu_calendar::types::Weekday) -> Self {
        use icu_calendar::types::Weekday::*;
        match value {
            Monday => Self::Monday,
            Tuesday => Self::Tuesday,
            Wednesday => Self::Wednesday,
            Thursday => Self::Thursday,
            Friday => Self::Friday,
            Saturday => Self::Saturday,
            Sunday => Self::Sunday,
        }
    }
}

#[cfg(feature = "8601")]
impl From<crate::Iso8601Month> for icu_calendar::types::Month {
    fn from(value: crate::Iso8601Month) -> Self {
        Self::new(value.code().parse().expect("generated ISO month number"))
    }
}
#[cfg(feature = "8601")]
impl TryFrom<icu_calendar::types::Month> for crate::Iso8601Month {
    type Error = crate::ParseIsoCodeError;
    fn try_from(value: icu_calendar::types::Month) -> Result<Self, Self::Error> {
        if value.is_leap() {
            return Err(crate::ParseIsoCodeError {
                standard: Self::STANDARD,
                input: value.code().to_string(),
            });
        }
        format!("{:02}", value.number()).parse()
    }
}

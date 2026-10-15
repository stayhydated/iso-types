pub use icu_locale::preferences::extensions::unicode::keywords::{
    CalendarAlgorithm, HijriCalendarAlgorithm, HourCycle, MeasurementSystem,
};
mod data;

/// Week conventions from CLDR territory data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WeekPreferences {
    pub first_weekday: crate::Iso8601Weekday,
    pub weekend_start: crate::Iso8601Weekday,
    pub weekend_end: crate::Iso8601Weekday,
    /// Minimum days in the new year for its first calendar week.
    pub min_week_days: u8,
}

impl WeekPreferences {
    /// Test membership in the inclusive weekend, including ranges across Sunday.
    pub const fn is_weekend(self, day: crate::Iso8601Weekday) -> bool {
        let (day, start, end) = (
            day as u16,
            self.weekend_start as u16,
            self.weekend_end as u16,
        );
        if start <= end {
            day >= start && day <= end
        } else {
            day >= start || day <= end
        }
    }
}

/// Paper sizes selected by CLDR's territory defaults.
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    PartialEq,
    strum::EnumString,
    strum::IntoStaticStr,
    strum::VariantArray,
    strum::EnumIter,
    strum::EnumCount,
    strum::FromRepr,
)]
#[strum(const_into_str)]
#[cfg_attr(feature = "fmt", derive(strum::Display))]
pub enum PaperSize {
    #[strum(to_string = "A4")]
    A4,
    #[strum(to_string = "US-Letter")]
    UsLetter,
}

#[cfg(feature = "216")]
impl PaperSize {
    /// The corresponding ISO 216 size, when the regional default uses that series.
    pub const fn iso216(self) -> Option<crate::Iso216> {
        match self {
            Self::A4 => Some(crate::Iso216::A4),
            Self::UsLetter => None,
        }
    }
}

/// Defaults for a country; applications can combine these with explicit ICU4X preferences.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RegionalPreferences {
    pub week: WeekPreferences,
    pub hour_cycle: HourCycle,
    /// Calendar identifiers in CLDR preference order.
    pub preferred_calendars: &'static [CalendarAlgorithm],
    pub measurement_system: MeasurementSystem,
    /// Category-specific temperature override, resolved against the general system.
    pub temperature_measurement_system: MeasurementSystem,
    pub paper_size: PaperSize,
}

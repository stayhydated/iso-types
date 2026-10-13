use crate::TimeZoneId;

impl TimeZoneId {
    /// Load transition rules from Jiff's bundled IANA database.
    pub fn time_zone(self) -> Result<jiff::tz::TimeZone, jiff::Error> {
        jiff::tz::TimeZone::get(self.code())
    }

    /// Convert an instant to this zone, using Jiff's transition rules.
    pub fn at(self, timestamp: jiff::Timestamp) -> Result<jiff::Zoned, jiff::Error> {
        Ok(timestamp.to_zoned(self.time_zone()?))
    }

    /// Convert to ICU4X's BCP47 time-zone identity.
    #[cfg(feature = "time-zone-localization")]
    pub fn icu_time_zone(self) -> icu_time::TimeZone {
        icu_time::TimeZone::from_iana_id(self.code())
    }

    /// Format a location name, such as “New York Time”, with ICU4X.
    #[cfg(feature = "time-zone-localization")]
    pub fn localized_name(
        self,
        locale: &icu_locale::Locale,
    ) -> Result<String, icu_datetime::DateTimeFormatterLoadError> {
        use writeable::Writeable as _;
        let formatter = icu_datetime::NoCalendarFormatter::try_new(
            locale.into(),
            icu_datetime::fieldsets::zone::Location,
        )?;
        Ok(formatter
            .format(&self.icu_time_zone().without_offset())
            .write_to_string()
            .into_owned())
    }
}

impl TryFrom<TimeZoneId> for jiff::tz::TimeZone {
    type Error = jiff::Error;
    fn try_from(value: TimeZoneId) -> Result<Self, Self::Error> {
        value.time_zone()
    }
}

#[cfg(feature = "time-zone-localization")]
impl From<TimeZoneId> for icu_time::TimeZone {
    fn from(value: TimeZoneId) -> Self {
        value.icu_time_zone()
    }
}

#[cfg(feature = "time-zone-localization")]
impl TryFrom<icu_time::TimeZone> for TimeZoneId {
    type Error = crate::ParseIsoCodeError;
    fn try_from(value: icu_time::TimeZone) -> Result<Self, Self::Error> {
        let parser = icu_time::zone::iana::IanaParserExtended::new();
        let canonical = parser
            .iter()
            .find(|entry| entry.time_zone == value)
            .ok_or_else(|| crate::ParseIsoCodeError {
                standard: Self::STANDARD,
                input: value.as_str().to_owned(),
            })?;
        canonical.canonical.parse()
    }
}

impl TryFrom<&jiff::tz::TimeZone> for TimeZoneId {
    type Error = crate::ParseIsoCodeError;
    fn try_from(value: &jiff::tz::TimeZone) -> Result<Self, Self::Error> {
        if value.is_unknown() {
            return Err(crate::ParseIsoCodeError {
                standard: Self::STANDARD,
                input: "Etc/Unknown".to_owned(),
            });
        }
        if value == &jiff::tz::TimeZone::UTC {
            return Ok(Self::EtcUtc);
        }
        let input = value
            .iana_name()
            .map_or_else(|| format!("{value:?}"), str::to_owned);
        input.parse()
    }
}

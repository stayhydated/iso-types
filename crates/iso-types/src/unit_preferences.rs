mod data;

/// One ranked CLDR preference for a measurement usage and region.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UnitPreference {
    /// Ordered components: the first is largest, such as feet followed by inches.
    pub units: &'static [PreferredUnit],
    /// Lower bound measured in the first unit; the final choice covers smaller quantities.
    pub threshold: f64,
    /// CLDR number-formatting skeleton, such as `precision-increment/50`.
    pub skeleton: Option<&'static str>,
}

/// A CLDR core unit, including composed identifiers used by regional preferences.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PreferredUnit {
    pub(crate) code: &'static str,
    pub(crate) catalog_unit: Option<crate::MeasurementUnit>,
}

impl PreferredUnit {
    pub const fn code(self) -> &'static str {
        self.code
    }
    pub const fn catalog_unit(self) -> Option<crate::MeasurementUnit> {
        self.catalog_unit
    }

    /// Parse this identifier using ICU4X's unit grammar and conversion data.
    #[cfg(feature = "unit-conversion")]
    pub fn icu_unit(
        self,
    ) -> Result<
        icu_experimental::measure::measureunit::MeasureUnit,
        icu_experimental::measure::parser::InvalidUnitError,
    > {
        icu_experimental::measure::measureunit::MeasureUnit::try_from_str(self.code)
    }
}

//! Typed unit conversions through the pinned ICU4X conversion provider.

use std::sync::OnceLock;

use icu_experimental::{
    measure::measureunit::MeasureUnit,
    units::{
        converter::UnitsConverter, converter_factory::ConverterFactory, convertible::Convertible,
    },
};

use crate::{MeasurementUnit, VariantArray as _};

/// ICU cannot construct a conversion between these measurement units.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnitConversionError {
    /// The source unit.
    pub from: MeasurementUnit,
    /// The requested destination unit.
    pub to: MeasurementUnit,
}

impl std::fmt::Display for UnitConversionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "cannot convert {} to {}",
            self.from.code(),
            self.to.code()
        )
    }
}

impl std::error::Error for UnitConversionError {}

fn units() -> &'static [MeasureUnit] {
    static UNITS: OnceLock<Box<[MeasureUnit]>> = OnceLock::new();
    UNITS.get_or_init(|| {
        MeasurementUnit::VARIANTS
            .iter()
            .map(|unit| {
                MeasureUnit::try_from_str(unit.code()).expect("validated CLDR unit has ICU syntax")
            })
            .collect()
    })
}

impl MeasurementUnit {
    /// The parsed ICU measurement unit, cached with the rest of the common catalog.
    pub fn icu_unit(self) -> &'static MeasureUnit {
        &units()[self as usize]
    }

    /// Create a reusable ICU converter; incompatible dimensions return an error.
    pub fn converter_to<N: Convertible>(
        self,
        target: Self,
    ) -> Result<UnitsConverter<N>, UnitConversionError> {
        ConverterFactory::new()
            .converter(self.icu_unit(), target.icu_unit())
            .map_err(|_| UnitConversionError {
                from: self,
                to: target,
            })
    }

    /// Convert an `f64` quantity using ICU's factors, offsets, and reciprocal rules.
    pub fn convert(self, value: f64, target: Self) -> Result<f64, UnitConversionError> {
        Ok(self.converter_to::<f64>(target)?.convert(value))
    }
}

impl From<MeasurementUnit> for MeasureUnit {
    fn from(value: MeasurementUnit) -> Self {
        value.icu_unit().clone()
    }
}

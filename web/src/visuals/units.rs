use iso_types::{MeasurementUnit, UnitCategory, UnitUsage};

use super::Icon;

pub(super) fn category_icon(category: UnitCategory) -> Icon {
    match category {
        UnitCategory::Acceleration => Icon::Wind,
        UnitCategory::Concentr => Icon::Food,
        UnitCategory::Graphics => Icon::Camera,
        UnitCategory::Light => Icon::Eye,
        UnitCategory::Magnetic => Icon::Resistance,
        UnitCategory::Torque => Icon::Engine,
        UnitCategory::Angle => Icon::Angle,
        UnitCategory::Area => Icon::Area,
        UnitCategory::Consumption => Icon::Fuel,
        UnitCategory::Digital => Icon::Binary,
        UnitCategory::Duration => Icon::Clock,
        UnitCategory::Electric => Icon::Bolt,
        UnitCategory::Energy => Icon::Battery,
        UnitCategory::Force => Icon::Force,
        UnitCategory::Frequency => Icon::Wave,
        UnitCategory::Length => Icon::Ruler,
        UnitCategory::Mass => Icon::Weight,
        UnitCategory::Power => Icon::Plug,
        UnitCategory::Pressure => Icon::Pressure,
        UnitCategory::Speed => Icon::Speed,
        UnitCategory::Temperature => Icon::Thermometer,
        UnitCategory::Volume => Icon::Volume,
    }
}

pub(super) fn unit_icon(unit: MeasurementUnit) -> Icon {
    match unit {
        MeasurementUnit::Ampere => Icon::Current,
        MeasurementUnit::Ohm => Icon::Resistance,
        MeasurementUnit::Foodcalorie | MeasurementUnit::Kilocalorie => Icon::Food,
        MeasurementUnit::Horsepower => Icon::Engine,
        _ => category_icon(unit.category()),
    }
}

pub(super) fn usage_icon(usage: UnitUsage) -> Icon {
    match usage {
        UnitUsage::AreaFloor => Icon::House,
        UnitUsage::AreaGeograph | UnitUsage::AreaLand => Icon::Map,
        UnitUsage::ConsumptionVehicleFuel => Icon::Fuel,
        UnitUsage::DurationMedia => Icon::Play,
        UnitUsage::EnergyFood => Icon::Food,
        UnitUsage::LengthFocalLength => Icon::Camera,
        UnitUsage::LengthPerson | UnitUsage::LengthPersonHeight | UnitUsage::MassPerson => {
            Icon::Person
        }
        UnitUsage::LengthRainfall | UnitUsage::SpeedRainfall => Icon::Rain,
        UnitUsage::LengthRoad => Icon::Road,
        UnitUsage::LengthSnowfall | UnitUsage::SpeedSnowfall => Icon::Snow,
        UnitUsage::LengthVehicle | UnitUsage::VolumeVehicle => Icon::Car,
        UnitUsage::LengthVisiblty => Icon::Eye,
        UnitUsage::PowerEngine => Icon::Engine,
        UnitUsage::SpeedWind => Icon::Wind,
        UnitUsage::TemperatureWeather | UnitUsage::PressureBaromtrc => Icon::Weather,
        UnitUsage::VolumeOil => Icon::Oil,
        UnitUsage::VolumeFluid => Icon::Volume,
        UnitUsage::AreaDefault
        | UnitUsage::ConsumptionDefault
        | UnitUsage::DurationDefault
        | UnitUsage::EnergyDefault
        | UnitUsage::LengthDefault
        | UnitUsage::MassDefault
        | UnitUsage::PowerDefault
        | UnitUsage::PressureDefault
        | UnitUsage::SpeedDefault
        | UnitUsage::TemperatureDefault
        | UnitUsage::VolumeDefault => category_icon(usage.category()),
    }
}

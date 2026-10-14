use iso_types::{AddressField, AddressFieldLabel};

use super::Icon;

pub(super) fn field_icon(field: AddressField) -> Icon {
    match field {
        AddressField::StreetAddress => Icon::Road,
        AddressField::Locality => Icon::City,
        AddressField::DependentLocality => Icon::House,
        AddressField::LandmarkAffix
        | AddressField::LandmarkDescriptor
        | AddressField::LandmarkName => Icon::Location,
        AddressField::Recipient => Icon::Person,
        AddressField::Organization => Icon::Building,
        AddressField::AdministrativeArea => Icon::Map,
        AddressField::SortingCode | AddressField::PostalCode => Icon::Mail,
    }
}

pub(super) fn label_icon(label: AddressFieldLabel) -> Icon {
    match label {
        AddressFieldLabel::City | AddressFieldLabel::PostTown => Icon::City,
        AddressFieldLabel::Neighborhood
        | AddressFieldLabel::Suburb
        | AddressFieldLabel::Townland
        | AddressFieldLabel::VillageTownship => Icon::House,
        AddressFieldLabel::LandmarkAffix
        | AddressFieldLabel::LandmarkDescriptor
        | AddressFieldLabel::LandmarkName => Icon::Location,
        AddressFieldLabel::Organization => Icon::Building,
        AddressFieldLabel::Recipient => Icon::Person,
        AddressFieldLabel::Eircode
        | AddressFieldLabel::Pin
        | AddressFieldLabel::Postal
        | AddressFieldLabel::Sorting
        | AddressFieldLabel::Zip => Icon::Mail,
        AddressFieldLabel::Street => Icon::Road,
        AddressFieldLabel::Area
        | AddressFieldLabel::County
        | AddressFieldLabel::Department
        | AddressFieldLabel::District
        | AddressFieldLabel::DoSi
        | AddressFieldLabel::Emirate
        | AddressFieldLabel::Island
        | AddressFieldLabel::Oblast
        | AddressFieldLabel::Parish
        | AddressFieldLabel::Prefecture
        | AddressFieldLabel::Province
        | AddressFieldLabel::State => Icon::Map,
    }
}

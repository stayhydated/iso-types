mod data;

/// Country-level libaddressinput metadata, with its documented schema defaults resolved.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AddressMetadata {
    /// The upstream template: `%n` is a line break; other placeholders are AddressField codes.
    pub format: &'static str,
    pub latin_format: Option<&'static str>,
    /// Required components, in upstream order. The country is supplied separately.
    pub required_fields: &'static [crate::AddressField],
    /// Components in their first appearance order in the native template.
    pub fields: &'static [crate::AddressField],
    /// The upstream Java-compatible expression; choose a compatible engine before validation.
    pub postal_code_pattern: Option<&'static str>,
    pub postal_code_examples: &'static [&'static str],
    /// Labels supplied by upstream for specific languages or structured field identifiers.
    pub label_overrides: &'static [AddressLabelOverride],
    pub default_language: Option<&'static str>,
    pub languages: &'static [&'static str],
    pub(crate) labels: [crate::AddressFieldLabel; 4],
}

/// A libaddressinput label override. Structured identifiers can include fields such as `S1`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AddressLabelOverride {
    pub field: &'static str,
    pub language: Option<&'static str>,
    pub label: Option<&'static str>,
    /// An upstream message identifier when the override refers to translated UI resources.
    pub message: Option<&'static str>,
}

impl AddressMetadata {
    pub fn is_required(self, field: crate::AddressField) -> bool {
        self.required_fields.contains(&field)
    }
    pub fn uses(self, field: crate::AddressField) -> bool {
        self.fields.contains(&field)
    }
    pub fn uses_postal_code(self) -> bool {
        self.uses(crate::AddressField::PostalCode)
    }

    /// Find an upstream label for an exact language tag or a language-independent override.
    pub fn label_override(
        self,
        field: &str,
        language: Option<&str>,
    ) -> Option<&'static AddressLabelOverride> {
        self.label_overrides
            .iter()
            .find(|value| value.field == field && value.language == language)
    }

    /// A typed Fluent label selected for the country's address convention.
    pub const fn field_label(self, field: crate::AddressField) -> crate::AddressFieldLabel {
        use crate::AddressField::*;
        use crate::AddressFieldLabel as Label;
        match field {
            AdministrativeArea => self.labels[0],
            Locality => self.labels[1],
            DependentLocality => self.labels[2],
            PostalCode => self.labels[3],
            Recipient => Label::Recipient,
            Organization => Label::Organization,
            StreetAddress => Label::Street,
            SortingCode => Label::Sorting,
            LandmarkDescriptor => Label::LandmarkDescriptor,
            LandmarkAffix => Label::LandmarkAffix,
            LandmarkName => Label::LandmarkName,
        }
    }
}

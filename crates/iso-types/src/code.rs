/// An unrecognized code in a maintained catalog.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseIsoCodeError {
    /// The catalog identifier, such as `3166-1` or `units`.
    pub standard: &'static str,
    /// The rejected input, preserved for diagnostics.
    pub input: String,
}
impl std::fmt::Display for ParseIsoCodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unrecognized {} code: {}", self.standard, self.input)
    }
}
impl std::error::Error for ParseIsoCodeError {}

/// Shared code and label contract for maintained catalogs and declared subsets.
pub trait IsoCode:
    strum::VariantArray
    + strum::IntoEnumIterator
    + strum::EnumCount
    + Copy
    + Eq
    + std::str::FromStr<Err = ParseIsoCodeError>
{
    /// The ISO standard or catalog identifier.
    const STANDARD: &'static str;
    /// The authority responsible for the English labels.
    const LABEL_SOURCE: LabelSource;
    /// The canonical, case-sensitive catalog representation.
    fn code(self) -> &'static str;
    /// The authoritative English label.
    fn english_name(self) -> &'static str;
}

/// The authority that owns a catalog's English labels.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LabelSource {
    /// Names extracted from pinned ICU4X compiled providers.
    Icu4x,
    /// Names owned by the package's English Fluent resource.
    Fluent,
    /// English display names from the pinned CLDR locale data.
    Cldr,
    /// English language-group names published by the Library of Congress.
    LibraryOfCongress,
    /// English trade-unit and package names published by UNECE.
    Unece,
}

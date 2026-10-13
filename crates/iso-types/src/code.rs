/// An unrecognized code in a maintained ISO or measurement catalog.
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
    /// The ISO standard or finite catalog identifier.
    const STANDARD: &'static str;
    /// The authority responsible for the English labels.
    const LABEL_SOURCE: LabelSource;
    /// The canonical, case-sensitive ISO representation.
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
    /// English unit display names from the pinned CLDR locale data.
    Cldr,
}

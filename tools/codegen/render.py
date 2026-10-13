"""Render Rust, macro selection data, and owned Fluent resources."""

from collections import defaultdict
import json

from .catalog import Catalog
from .relations import Relations, variant_name
from .output import GENERATED
from .rust import literal, markdown


def string_table(method: str, values: list[str | None], *, optional: bool = False) -> str:
    if optional:
        value_type = "Option<&'static str>"
        entries = [f"Some({literal(value)})" if value is not None else "None" for value in values]
    else:
        value_type = "&'static str"
        entries = [literal(value) for value in values]
    table_type = value_type.replace("'static ", "")
    return f"""    pub const fn {method}(self) -> {value_type} {{
        const VALUES: &[{table_type}] = &[{', '.join(entries)}];
        VALUES[self as usize]
    }}
"""


def value_table(method: str, value_type: str, values: list[str]) -> str:
    return f"""    pub const fn {method}(self) -> {value_type} {{
        const VALUES: &[{value_type.replace("'static ", "")}] = &[{', '.join(values)}];
        VALUES[self as usize]
    }}
"""


def slice_table(method: str, type_name: str, values: list[tuple]) -> str:
    return value_table(method, f"&'static [{type_name}]", [
        "&[" + ", ".join(f"{type_name}::{entry.variant}" for entry in entries) + "]"
        for entries in values
    ])


def metadata_enum(name: str, values: dict[str, str], description: str) -> str:
    variants = "".join(f"    #[strum(to_string = {literal(code)})]\n    {variant},\n"
                       for code, variant in values.items())
    return f"""/// {description}
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd,
    strum::VariantArray, strum::EnumIter, strum::EnumCount, strum::FromRepr,
    strum::EnumString, strum::IntoStaticStr)]
#[strum(const_into_str)]
#[cfg_attr(feature = "fmt", derive(strum::Display))]
#[repr(u16)]
pub enum {name} {{
{variants}}}
"""


def catalog_rust(catalog: Catalog, currency_fractions: dict) -> str:
    spec, entries = catalog.spec, catalog.entries
    name = spec.type_name
    methods = []
    if spec.standard == "4217":
        for method, field, description in (
            ("fraction_digits", "digits", "CLDR standard formatting precision, extracted from ICU4X."),
            ("cash_fraction_digits", "cash_digits", "CLDR cash formatting precision, extracted from ICU4X."),
            ("rounding_increment", "rounding", "Formatting increment in the smallest fraction unit; `1` means ordinary digit rounding."),
            ("cash_rounding_increment", "cash_rounding", "Cash formatting increment in the smallest cash fraction unit."),
        ):
            values = [currency_fractions[entry.code][field] for entry in entries]
            if any(type(value) is not int or not 0 <= value <= 255 for value in values):
                raise ValueError(f"invalid ICU currency formatting metadata: {field}")
            methods.append(f"    /// {description}\n" + value_table(method, "u8", [str(value) for value in values]))
    fields = sorted({field for entry in entries for field in entry.metadata})
    for field in fields:
        if spec.standard == "3166-2" and field in {"parent", "type"}:
            continue
        if spec.standard == "units" and field == "category":
            continue
        method = "kind" if field == "type" else field
        methods.append(string_table(method, [entry.metadata.get(field) for entry in entries], optional=True))
    for method, field in (("english_name", "name"), ("message_id", "message_id")):
        methods.append(string_table(method, [getattr(entry, field) for entry in entries]))
    candidates = defaultdict(set)
    for entry in entries:
        for alias in entry.aliases:
            candidates[alias].add(entry.variant)
    codes = {entry.code for entry in entries}
    aliases = {alias: next(iter(variants)) for alias, variants in candidates.items()
               if alias not in codes and len(variants) == 1}
    variant_aliases = defaultdict(list)
    for alias, variant in sorted(aliases.items()):
        variant_aliases[variant].append(alias)
    variants = "".join(
        f"    /// {markdown(entry.name)} (`{entry.code}`).\n"
        f"    #[strum(to_string = {literal(entry.code)}"
        + "".join(f", serialize = {literal(alias)}" for alias in variant_aliases[entry.variant])
        + f")]\n    {entry.variant},\n"
        for entry in entries
    )
    # The subdivision catalog's PHF path is measured against the binary-search baseline.
    parser = "#[strum(use_phf)]\n" if spec.standard == "3166-2" else ""
    metadata = ""
    if spec.standard == "units":
        categories = {category: variant_name(category) for category in sorted({entry.metadata["category"] for entry in entries})}
        metadata = metadata_enum("UnitCategory", categories, "The CLDR measurement category of a common unit.") + "\n"
        methods.append(value_table("category", "UnitCategory", [
            f"UnitCategory::{categories[entry.metadata['category']]}" for entry in entries
        ]))
    description = ("Common CLDR core unit identifiers; English display names are owned by CLDR."
                   if spec.standard == "units" else
                   f"Assigned codes in ISO {spec.standard}; English labels are owned by {spec.authority}.")
    return GENERATED + metadata + f"""/// {description}
/// `from_repr()` accepts a zero-based catalog ordinal; parse ISO numeric codes as strings.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd,
    strum::VariantArray, strum::EnumIter, strum::EnumCount, strum::FromRepr,
    strum::EnumString, strum::IntoStaticStr)]
#[strum(const_into_str, parse_err_ty = crate::ParseIsoCodeError, parse_err_fn = Self::parse_error)]
{parser}\
#[repr(u16)]
pub enum {name} {{
{variants}}}

impl {name} {{
    pub const STANDARD: &'static str = {literal(spec.standard)};
    pub const LABEL_SOURCE: crate::LabelSource = crate::LabelSource::{spec.authority};
    pub const fn code(self) -> &'static str {{ self.into_str() }}
    fn parse_error(input: &str) -> crate::ParseIsoCodeError {{
        crate::ParseIsoCodeError {{ standard: Self::STANDARD, input: input.to_owned() }}
    }}
{''.join(methods)}}}

impl crate::IsoCode for {name} {{
    const STANDARD: &'static str = Self::STANDARD;
    const LABEL_SOURCE: crate::LabelSource = Self::LABEL_SOURCE;
    fn code(self) -> &'static str {{ self.code() }}
    fn english_name(self) -> &'static str {{ self.english_name() }}
}}

#[cfg(feature = "fmt")]
impl std::fmt::Display for {name} {{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {{
        f.write_str(self.english_name())
    }}
}}

#[cfg(feature = "fluent")]
impl es_fluent::FluentMessage for {name} {{
    fn to_fluent_string_with(&self, localize: &mut es_fluent::FluentMessageLookup<'_>) -> String {{
        localize(self.fluent_key(), None)
    }}
}}

#[cfg(feature = "fluent")]
impl {name} {{
    pub const fn fluent_key(self) -> es_fluent::registry::StaticFluentMessageKey {{
        es_fluent::registry::__macro::static_message_key(
            "iso-types",
            es_fluent::registry::__macro::static_domain("iso-types"),
            es_fluent::registry::__macro::static_entry_id(self.message_id()),
        )
    }}
}}
"""


def catalog_modules(catalogs: tuple[Catalog, ...]) -> str:
    return GENERATED + "".join(
        f'#[cfg(feature = "{catalog.spec.feature}")]\nmod {catalog.spec.module};\n'
        f'#[cfg(feature = "{catalog.spec.feature}")]\npub use {catalog.spec.module}::'
        + (f'{{{catalog.spec.type_name}, UnitCategory}};\n' if catalog.spec.standard == "units"
           else f'{catalog.spec.type_name};\n')
        for catalog in catalogs
    )


def macro_data(catalogs: tuple[Catalog, ...]) -> str:
    """Macros delegate labels and metadata to the public catalog enums."""
    data = {
        catalog.spec.standard: {
            "type": catalog.spec.type_name,
            "feature": catalog.spec.feature,
            "entries": [{"code": entry.code, "variant": entry.variant} for entry in catalog.entries],
        }
        for catalog in catalogs
    }
    return json.dumps(data, ensure_ascii=False, indent=2) + "\n"


def fluent_resource(catalog: Catalog) -> str:
    return f"## ISO {catalog.spec.standard}\n\n" + "".join(
        f"{entry.message_id} = {entry.name}\n" for entry in catalog.entries
    )


def fluent_registration(catalogs: tuple[Catalog, ...]) -> str:
    resources = "".join(
        f'    #[cfg(feature = "{catalog.spec.feature}")]\n'
        f'    include_str!("../../i18n/en/iso-types/{catalog.spec.standard}.ftl"),\n'
        for catalog in catalogs if catalog.spec.authority == "Fluent"
    )
    lookups = "".join(
        f'    #[cfg(feature = "{catalog.spec.feature}")]\n'
        f'    if let Some(code) = id.strip_prefix("iso-{catalog.spec.standard}-") {{\n'
        f'        return Some(code.parse::<crate::{catalog.spec.type_name}>().ok()?.english_name());\n'
        f'    }}\n'
        for catalog in catalogs if catalog.spec.authority != "Fluent"
    )
    return GENERATED + f"""pub(super) const ENGLISH_RESOURCES: &[&str] = &[
{resources}];

pub(super) fn catalog_english_name(id: &str) -> Option<&'static str> {{
{lookups}    let _ = id;
    None
}}
"""


def geography(model: Relations) -> str:
    country, subdivision, currency = "crate::Iso3166_1", "crate::Iso3166_2", "crate::Iso4217"
    enum = metadata_enum("SubdivisionKind", model.kinds, "An ISO subdivision category, preserving its source name.")
    subdivision_methods = value_table("country", country, [f"{country}::{entry.variant}" for entry in model.subdivision_countries])
    subdivision_methods += value_table("parent", f"Option<{subdivision}>", [
        f"Some({subdivision}::{entry.variant})" if entry else "None" for entry in model.parents
    ])
    subdivision_methods += value_table("kind", "SubdivisionKind", [
        f"SubdivisionKind::{model.kinds[entry.metadata['type']]}" for entry in model.subdivisions.entries
    ])
    subdivision_methods += string_table("local_code", [entry.code.split("-", 1)[1] for entry in model.subdivisions.entries])
    subdivision_methods += slice_table("children", subdivision, [model.children.get(entry.code, ()) for entry in model.subdivisions.entries])
    country_methods = slice_table("subdivisions", subdivision, [model.country_subdivisions.get(entry.code, ()) for entry in model.countries.entries])
    country_methods += slice_table("root_subdivisions", subdivision, [model.country_roots.get(entry.code, ()) for entry in model.countries.entries])
    currency_methods = slice_table("currencies", currency, [model.country_currencies[entry.code] for entry in model.countries.entries])
    inverse_methods = slice_table("countries", country, [model.currency_countries.get(entry.code, ()) for entry in model.currencies.entries])
    return GENERATED + f"""#[cfg(feature = "3166-2")]
{enum}

#[cfg(feature = "3166-2")]
impl {subdivision} {{
{subdivision_methods}}}

#[cfg(feature = "3166-2")]
impl {country} {{
{country_methods}
    /// Resolve a case-sensitive local subdivision suffix, such as `CA` within `US`.
    pub fn subdivision(self, local_code: &str) -> Option<{subdivision}> {{
        self.subdivisions().iter().copied().find(|subdivision| subdivision.local_code() == local_code)
    }}
}}

#[cfg(all(feature = "3166-1", feature = "4217"))]
impl {country} {{
    /// Active tender currencies in the pinned {model.as_of} snapshot, in canonical code order.
{currency_methods}}}

#[cfg(all(feature = "3166-1", feature = "4217"))]
impl {currency} {{
    /// Countries and territories where this currency is active tender at {model.as_of}.
{inverse_methods}}}
"""

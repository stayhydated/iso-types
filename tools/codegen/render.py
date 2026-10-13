"""Render Rust, macro selection data, and owned Fluent resources."""

from collections import defaultdict
import json

from .catalog import Catalog
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


def catalog_rust(catalog: Catalog) -> str:
    spec, entries = catalog.spec, catalog.entries
    name = spec.type_name
    variants = "".join(
        f"    /// {markdown(entry.name)} (`{entry.code}`).\n    {entry.variant},\n"
        for entry in entries
    )
    methods = []
    fields = sorted({field for entry in entries for field in entry.metadata})
    for field in fields:
        method = "kind" if field == "type" else field
        methods.append(string_table(method, [entry.metadata.get(field) for entry in entries], optional=True))
    for method, field in (("code", "code"), ("english_name", "name"), ("message_id", "message_id")):
        methods.append(string_table(method, [getattr(entry, field) for entry in entries]))
    candidates = defaultdict(set)
    for entry in entries:
        for alias in entry.aliases:
            candidates[alias].add(entry.variant)
    codes = {entry.code for entry in entries}
    aliases = {alias: next(iter(variants)) for alias, variants in candidates.items()
               if alias not in codes and len(variants) == 1}
    parse_aliases = ""
    if aliases:
        arms = "".join(f"            {literal(alias)} => return Ok(Self::{variant}),\n"
                       for alias, variant in sorted(aliases.items()))
        parse_aliases = f"        match input {{\n{arms}            _ => {{}}\n        }}\n"
    return GENERATED + f"""/// Assigned codes in ISO {spec.standard}; English labels are owned by {spec.authority}.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, strum::VariantArray)]
#[repr(u16)]
pub enum {name} {{
{variants}}}

impl {name} {{
    pub const STANDARD: &'static str = {literal(spec.standard)};
    pub const LABEL_SOURCE: crate::LabelSource = crate::LabelSource::{spec.authority};
{''.join(methods)}}}

impl std::str::FromStr for {name} {{
    type Err = crate::ParseIsoCodeError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {{
        let variants = <Self as strum::VariantArray>::VARIANTS;
        if let Ok(index) = variants.binary_search_by(|entry| entry.code().cmp(input)) {{
            return Ok(variants[index]);
        }}
{parse_aliases}        Err(crate::ParseIsoCodeError {{ standard: Self::STANDARD, input: input.to_owned() }})
    }}
}}

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
        f'#[cfg(feature = "{catalog.spec.feature}")]\npub use {catalog.spec.module}::{catalog.spec.type_name};\n'
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
        for catalog in catalogs if not catalog.spec.icu_names
    )
    lookups = "".join(
        f'    #[cfg(feature = "{catalog.spec.feature}")]\n'
        f'    if let Some(code) = id.strip_prefix("iso-{catalog.spec.standard}-") {{\n'
        f'        return Some(code.parse::<crate::{catalog.spec.type_name}>().ok()?.english_name());\n'
        f'    }}\n'
        for catalog in catalogs if catalog.spec.icu_names
    )
    return GENERATED + f"""pub(super) const ENGLISH_RESOURCES: &[&str] = &[
{resources}];

pub(super) fn icu_english_name(id: &str) -> Option<&'static str> {{
{lookups}    let _ = id;
    None
}}
"""

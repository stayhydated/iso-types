# iso-types-macros

The macro implementation behind [iso-types][facade]. Applications use that
facade's `iso_enum` and `iso_locale` re-exports; direct dependencies serve macro
implementation and expansion tooling.

Requires Rust 1.99 or newer.

## Features

| Feature | Default | Purpose |
| --- | --- | --- |
| `fluent` | No | Generate Fluent message implementations through the iso-types facade. |
| `locales` | No | Compile the configured locale macro using es-fluent-toml. |

`iso_enum` accepts `standard = "3166-1"` and an optional canonical
`codes = ["US", "CA"]` list from the maintained catalog. ICU-owned catalogs
include only current, common codes with an ICU English name. `iso_locale` reads
the invoking package's
`i18n.toml`; its optional `custom` mode assigns label ownership to that package.
Both require an empty enum without generics.

Use `standard = "4217"` for currency subsets, `"3166-2"` for delivery areas,
and `"units"` for common CLDR measurement units. `catalog_value()` exposes
currency precision, subdivision hierarchy, and measurement categories/conversion
on the facade's maintained type. Enable each corresponding catalog feature.

Use `country = "US"` with `3166-2`, or `category = "length"` with `units` and
`unit-preferences`. Filters intersect with an optional `codes` list and reject
empty results. Additional catalogs use `regions`, `time-zones`, `calling-codes`,
`address-fields`, and `address-labels`; English labels delegate to the facade.

Both macros derive Strum's `VariantArray`, `EnumIter`, `EnumCount`, `FromRepr`,
and `IntoStaticStr`. Import `iso_types::{VariantArray, IntoEnumIterator, EnumCount}`
for `VARIANTS`, `iter()`, and `COUNT`. Canonical `code()` and `into_str()` work
in const contexts. `from_repr()` accepts a zero-based declaration ordinal;
use parsing for ISO codes and locale tags. Strum paths resolve through the
facade, including when the dependency is renamed.

Configured locales use current ICU canonical tags and must have complete ICU English
name data. Both Fluent and ICU language-identifier conversions are generated.

[facade]: https://github.com/stayhydated/iso-types/tree/master/crates/iso-types

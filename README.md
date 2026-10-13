# iso-types

Typed ISO enums for Rust applications, with English labels and optional
es-fluent messages. Choose complete catalogs or declare an application-specific
subset with `#[iso_enum]`. Use `#[iso_locale]` to generate a language picker from
an es-fluent package's configured locales.

Both public packages require Rust 1.99 or newer. Fluent integration uses the
es-fluent 0.21 API.

## Crates

| Crate | Purpose |
| --- | --- |
| [`iso-types`][types] | ISO catalogs, subset and locale macros, English display, and Fluent runtime integration. |
| [`iso-types-macros`][macros] | Macro implementation, normally consumed through iso-types. |

## Example

The default features include country and currency catalogs and the subset macro:

```rust
use iso_types::{iso_enum, Iso3166_1, Iso4217};

#[iso_enum(standard = "3166-1", codes = ["US", "CA"])]
enum Market {}

assert_eq!("USA".parse::<Market>()?, Market::Us);
assert_eq!(Market::Ca.catalog_value(), Iso3166_1::Ca);
assert_eq!(Iso4217::Usd.numeric(), Some("840"));
assert_eq!(Market::Us.to_string(), "United States");
# Ok::<(), iso_types::ParseIsoCodeError>(())
```

The maintained catalogs cover ISO 639 sets 1, 2, 3, and 5; ISO 3166 country,
subdivision, and former-country codes; ISO 4217 currencies; ISO 15924 scripts;
ISO 5218 sex codes; and ISO 8601 months and weekdays. Catalogs preserve canonical
code spelling and applicable alternate representations.

[types]: crates/iso-types/README.md
[macros]: crates/iso-types-macros/README.md

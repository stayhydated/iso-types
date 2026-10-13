# iso-types

[![CI][ci-image]][ci]

Typed ISO enums for Rust applications, with ICU4X English names and optional
es-fluent messages. Choose maintained catalogs or declare an application-specific
subset with `#[iso_enum]`. Use `#[iso_locale]` to generate a language picker from
an es-fluent package's configured locales.

Both public packages require Rust 1.99 or newer. Fluent integration uses the
es-fluent 0.21 API.

ICU4X owns language, country, script, currency, month, weekday, and composite
locale names. ICU-owned catalogs include assigned codes with an ICU4X English
name. Every included variant has English `Display`; codes without ICU names are
excluded. Current subdivisions and sex codes use authoritative English Fluent
catalogs.

The data scope is current and common: living languages with CLDR modern locale
coverage, recommended modern scripts, current countries and subdivisions, and
currencies in active tender circulation. Historical, deprecated, reserved, and
uncommon entries are excluded from catalogs and locale lookup.

## Crates

| Crate | Purpose |
| --- | --- |
| [`iso-types`][types] | ISO catalogs, subset and locale macros, English display, and Fluent runtime integration. |
| [`iso-types-macros`][macros] | Macro implementation, normally consumed through iso-types. |

## Example

The default features include country and currency catalogs and the subset macro:

```rust
use iso_types::{iso_enum, Iso3166_1, Iso4217, VariantArray as _};

#[iso_enum(standard = "3166-1", codes = ["US", "CA"])]
enum Market {}

assert_eq!("USA".parse::<Market>()?, Market::Us);
assert_eq!(Market::Ca.catalog_value(), Iso3166_1::Ca);
assert_eq!(Market::VARIANTS, &[Market::Us, Market::Ca]);
assert_eq!(Iso4217::Usd.numeric(), Some("840"));
assert_eq!(Market::Us.to_string(), "United States");
# Ok::<(), iso_types::ParseIsoCodeError>(())
```

The maintained catalogs cover ISO 639 sets 1, 2, and 3; ISO 3166 country and
subdivision codes; ISO 4217 currencies; ISO 15924 scripts;
ISO 5218 sex codes; and ISO 8601 months and weekdays. Catalogs preserve canonical
code spelling and applicable alternate representations.

[types]: crates/iso-types/README.md
[macros]: crates/iso-types-macros/README.md
[ci-image]: https://github.com/stayhydated/iso-types/actions/workflows/ci.yml/badge.svg?branch=master
[ci]: https://github.com/stayhydated/iso-types/actions/workflows/ci.yml

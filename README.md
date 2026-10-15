# iso-types

[![CI][ci-image]][ci]
[![Book][book-image]][book]

Typed ISO, geographic, and measurement catalogs for Rust applications, with authoritative
English names and optional es-fluent messages. Choose maintained catalogs or
declare an application-specific subset with `#[iso_enum]`. Use `#[iso_locale]`
to generate a language picker from
an es-fluent package's configured locales.

The public packages require Rust 1.99 or newer. Fluent integration uses the
es-fluent 0.21 API.

[Interactive Dioxus examples][examples] cover every catalog and regional data
area, with reactive English/French messages through es-fluent-manager-dioxus.

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
| [`iso-types-locations`][locations] | Separately packaged UN/LOCODE directory with typed countries and preserved alternative listings. |
| [`iso-types-macros`][macros] | Macro implementation, normally consumed through iso-types. |

## Example

The default features include country and currency catalogs and the subset macro:

```rust
use iso_types::{iso_enum, EnumCount as _, Iso3166_1, Iso4217, VariantArray as _};

#[iso_enum(standard = "3166-1", codes = ["US", "CA"])]
enum Market {}

assert_eq!("USA".parse::<Market>()?, Market::Us);
assert_eq!(Market::Ca.catalog_value(), Iso3166_1::Ca);
assert_eq!(Market::VARIANTS, &[Market::Us, Market::Ca]);
assert_eq!(Market::COUNT, 2);
assert_eq!(Iso4217::Usd.numeric(), Some("840"));
assert_eq!(Iso3166_1::Us.currencies(), &[Iso4217::Usd]);
assert_eq!(Market::Us.to_string(), "United States");
# Ok::<(), iso_types::ParseIsoCodeError>(())
```

The maintained catalogs cover ISO 639 sets 1, 2, 3, and 5; ISO 3166 country and
subdivision codes; ISO 4217 currencies; ISO 15924 scripts;
ISO 5218 sex codes; and ISO 8601 months and weekdays. Catalogs preserve canonical
code spelling and applicable alternate representations.

Enable `3166-2` for typed country/subdivision navigation, including states,
provinces, territories, parents, and children. Country/currency relationships
support shared tender and countries using several currencies. Currency metadata
includes ICU4X formatting precision and cash rounding.

Enable `units` for 148 common CLDR measurement identifiers across 22 categories.
`unit-conversion` delegates metric/customary conversions, temperature offsets,
and reciprocal fuel economy to ICU4X. CLDR owns the unit display names.

Optional country metadata covers time zones, regional defaults, language/script
relationships, UN M49 regions, telephone calling codes, and postal address schemas.
Jiff supplies bundled IANA transition rules; ICU4X supplies time-zone mappings and
localized names. CLDR unit preferences retain usage-specific choices, thresholds,
mixed units, and formatting skeletons. Phone parsing integrates with `phonenumber`.

Additional features provide CLDR numbering systems and locale defaults, ISO 216
paper dimensions, language group hierarchies and macrolanguage membership,
UNECE trade units and packaging, and registered IBAN formats with checksum
validation. ISO 10383 support imports an application's own MIC CSV.
`iso-types-locations` supplies the larger UN/LOCODE directory separately.

Use `#[iso_enum(standard = "3166-2", country = "US")]` to select a country's
subdivisions, or `#[iso_enum(standard = "units", category = "length")]` to select
a measurement category. The facade's feature table describes each optional area.

[types]: crates/iso-types/README.md
[locations]: crates/iso-types-locations/README.md
[macros]: crates/iso-types-macros/README.md
[ci-image]: https://github.com/stayhydated/iso-types/actions/workflows/ci.yml/badge.svg?branch=master
[ci]: https://github.com/stayhydated/iso-types/actions/workflows/ci.yml
[book-image]: https://img.shields.io/badge/book-guide-blue
[book]: https://stayhydated.github.io/iso-types/book/
[examples]: https://stayhydated.github.io/iso-types/examples/

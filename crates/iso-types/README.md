# iso-types

Use `iso-types` for typed ISO code enums, application-specific subsets,
and configured locale enums. Every catalog supplies Strum's `VariantArray`,
canonical `code()`,
`english_name()`, parsing, and applicable metadata. ICU4X owns English names for
languages, countries, scripts, currencies, months, weekdays, and locales.
Current subdivisions and sex codes use English FTL. Fluent output and English
display share each catalog's authority.

Requires Rust 1.99 or newer. Fluent integration uses es-fluent's 0.21 API.
The ICU integration pins ICU4X 2.3; its locale-name APIs are upstream unstable.

## Features

| Feature | Default | Purpose |
| --- | --- | --- |
| `fmt` | Yes | English `Display` for every catalog enum. |
| `macros` | Yes | Re-export `iso_enum`. |
| `icu` | Yes | ICU locale/calendar re-exports and typed ISO conversions. |
| `locales` | No | ICU locale parsing and naming; re-export `iso_locale` and Fluent-facing `LanguageIdentifier`/`langid`. |
| `fluent` | No | `FluentMessage` and English utility registration using ICU names or owned FTL. |
| `full` | No | Enable every catalog below. |
| `639-1` | Yes | `Iso6391`: current languages with two-letter identifiers. |
| `639-2` | No | `Iso6392`: current languages with three-letter identifiers. |
| `639-3` | No | `Iso6393`: current living languages, scope, and kind. |
| `3166-1` | Yes | `Iso3166_1`: countries with alpha-3 and numeric representations. |
| `3166-2` | No | `Iso3166_2`: subdivisions and parent codes; enable `3166-1`. |
| `4217` | Yes | `Iso4217`: currencies in active tender circulation and numeric representations. |
| `15924` | Yes | `Iso15924`: recommended modern scripts and numeric representations. |
| `5218` | Yes | `Iso5218`: human sex interchange codes. |
| `8601` | Yes | `Iso8601Weekday` and `Iso8601Month`: numbered weekdays and months. |

Parsing is case-sensitive for ISO catalogs and accepts unique alternate codes,
including numeric and current alpha-code representations. ICU conversions
validate the current catalog when converting back into an ISO enum. Configured
locale parsing uses ICU4X normalization and distinguishes malformed, excluded,
and valid but unselected locales.

Language catalogs contain living languages with CLDR modern locale coverage and
a recommended default script. Script catalogs use CLDR's recommended regular
scripts. Countries and subdivisions use current regular codes; currencies use
the active tender interval and the current ISO 4217 list. Selection sources are
pinned to a dated snapshot and checked during generation.

`english_name()` returns `&'static str`. ICU-owned catalogs include only codes
with an authoritative English name. Excluded codes return `ParseIsoCodeError`
and remain absent from Fluent lookup. `Display` and declared subsets always have
an English label.

## Example

With default features:

```rust
# #[cfg(all(feature = "macros", feature = "3166-1", feature = "4217"))] {
use iso_types::{iso_enum, Iso3166_1, Iso4217, VariantArray as _};

#[iso_enum(standard = "3166-1", codes = ["US", "CA"])]
pub enum Markets {}

assert_eq!("USA".parse::<Markets>()?, Markets::Us);
assert_eq!(Markets::Ca.catalog_value(), Iso3166_1::Ca);
assert_eq!(Markets::VARIANTS, &[Markets::Us, Markets::Ca]);
assert_eq!(Iso4217::Usd.numeric(), Some("840"));
assert_eq!(Markets::Us.to_string(), "United States");
# }
# Ok::<(), iso_types::ParseIsoCodeError>(())
```

Enable the corresponding catalog feature before declaring a subset. Omit
`codes` to generate all catalog variants. Macros add `Clone`, `Copy`, `Debug`,
`Eq`, `Hash`, `PartialEq`, and Strum's `VariantArray`; additional derives remain
caller-owned. Import `iso_types::VariantArray` to access `Enum::VARIANTS` as a
constant slice. Catalogs use canonical code order; subsets use declaration order.

With `locales` and `fluent`, use `#[iso_locale]` on an empty enum in a package
with `i18n.toml` and canonical locale directories. The fallback locale becomes
`Default`. `Languages::VARIANTS` lists supported locales; variants convert into
`LanguageIdentifier` for manager selection. `Display` and built-in Fluent
labels remain English, including script and region qualifiers.

Use `#[iso_locale(custom)]` to register exact locale-tag message keys with the
application's FTL inventory. The application then owns those Fluent labels.
Catalog and locale macros support renamed dependencies.

Configured locale directories and the fallback identifier must use ICU canonical
tags. The macro checks current data scope and English name availability at
compile time. Locale enums convert into both Fluent and ICU language identifiers;
parsing normalizes current ISO forms such as `eng-US` to `en-US`.
`locale_english_name()` returns a `Result`
when naming arbitrary Fluent identifiers.

Locale identifiers use current languages, scripts, and regions without variants.
Deprecated aliases, bibliographic language forms, historical orthographies, and
excluded components return `LocaleConversionError::OutOfScope`.

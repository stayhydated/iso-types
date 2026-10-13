# iso-types

Use `iso-types` for typed ISO and measurement catalogs, application-specific
subsets, and configured locale enums. Every catalog supplies constant enumeration,
iteration, variant counts, canonical `code()`,
`english_name()`, parsing, and applicable metadata. ICU4X owns English names for
languages, countries, scripts, currencies, months, weekdays, and locales.
Current subdivisions and sex codes use English FTL. Fluent output and English
display share each catalog's authority. Common measurement units use CLDR
identifiers and English display names, with optional ICU4X conversions.

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
| `3166-2` | No | `Iso3166_2` and `SubdivisionKind`: typed hierarchy and country navigation; enable `3166-1`. |
| `4217` | Yes | `Iso4217`: active tender currencies, numeric representations, and ICU formatting precision. |
| `15924` | Yes | `Iso15924`: recommended modern scripts and numeric representations. |
| `5218` | Yes | `Iso5218`: human sex interchange codes. |
| `8601` | Yes | `Iso8601Weekday` and `Iso8601Month`: numbered weekdays and months. |
| `units` | No | `MeasurementUnit` and `UnitCategory`: 87 common CLDR measurement identifiers and English names; included by `full`. |
| `unit-conversion` | No | Enable `units` and ICU4X unit conversion, including temperature offsets and reciprocal fuel economy. |

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
use iso_types::{
    EnumCount as _, IntoEnumIterator as _, Iso3166_1, Iso4217, VariantArray as _,
    iso_enum,
};

#[iso_enum(standard = "3166-1", codes = ["US", "CA"])]
pub enum Markets {}

assert_eq!("USA".parse::<Markets>()?, Markets::Us);
assert_eq!(Markets::Ca.catalog_value(), Iso3166_1::Ca);
assert_eq!(Markets::VARIANTS, &[Markets::Us, Markets::Ca]);
assert_eq!(Markets::COUNT, 2);
assert_eq!(Markets::iter().collect::<Vec<_>>(), Markets::VARIANTS);
assert_eq!(Iso4217::Usd.numeric(), Some("840"));
assert_eq!(Iso3166_1::Us.currencies(), &[Iso4217::Usd]);
assert_eq!(Iso4217::Jpy.fraction_digits(), 0);
assert_eq!(Markets::Us.to_string(), "United States");
# }
# Ok::<(), iso_types::ParseIsoCodeError>(())
```

With `3166-2`, navigate states, provinces, territories, districts, and other ISO
subdivision categories:

```rust
# #[cfg(feature = "3166-2")] {
use iso_types::{Iso3166_1, Iso3166_2, SubdivisionKind};

assert_eq!(Iso3166_1::Us.subdivision("CA"), Some(Iso3166_2::UsCa));
assert_eq!(Iso3166_2::UsCa.country(), Iso3166_1::Us);
assert_eq!(Iso3166_2::UsCa.kind(), SubdivisionKind::State);
assert_eq!(Iso3166_2::CaQc.kind(), SubdivisionKind::Province);
assert_eq!(Iso3166_2::GbLnd.parent(), Some(Iso3166_2::GbEng));
assert!(Iso3166_2::GbEng.children().contains(&Iso3166_2::GbLnd));
# }
```

`subdivisions()` lists all current subdivisions; `root_subdivisions()` starts
the hierarchy. `parent()` and `children()` navigate direct relationships.
`local_code()` returns the country-relative suffix accepted by `subdivision()`.
Lists follow canonical code order. Subset enums expose the same metadata through
`catalog_value()`.

Country `currencies()` and currency `countries()` are available when `3166-1`
and `4217` are enabled. These dated active-tender relationships allow multiple
currencies per country and shared currencies across countries. Countries without
active tender data have an empty slice. Currency `fraction_digits()`,
`cash_fraction_digits()`, `rounding_increment()`, and `cash_rounding_increment()`
use ICU4X/CLDR formatting rules. An increment of `5` with two cash fraction digits
represents `0.05`.

With `units` and `unit-conversion`:

```rust
# #[cfg(feature = "unit-conversion")] {
use iso_types::{MeasurementUnit, UnitCategory};

assert_eq!(MeasurementUnit::Meter.category(), UnitCategory::Length);
assert_eq!(MeasurementUnit::Meter.code(), "meter");
let centimeters = MeasurementUnit::Meter.convert(1.0, MeasurementUnit::Centimeter)?;
assert_eq!(centimeters, 100.0);
let fahrenheit = MeasurementUnit::Celsius.convert(0.0, MeasurementUnit::Fahrenheit)?;
assert_eq!(fahrenheit, 32.0);
# }
# Ok::<(), Box<dyn std::error::Error>>(())
```

The unit catalog covers length, area, volume, mass, duration, temperature, speed,
fuel economy, digital quantities, energy, power, pressure, frequency, angle,
electric quantities, and force. Parse canonical CLDR core IDs such as `meter`
or `gallon-imperial`. Unit symbols belong to localized formatting. English
`Display` uses CLDR display names, such as `meters`; `iso_enum(standard = "units",
codes = ["meter", "foot"])` declares a subset.

`converter_to::<f64>()` creates a reusable ICU converter. `icu_unit()` exposes
the cached parsed ICU unit, and values convert into ICU's `MeasureUnit`.
Incompatible conversions return `UnitConversionError` with the source and target.
Unit categories group the catalog; compatibility follows ICU's physical
dimensions. ICU4X's unit APIs use the pinned experimental 0.6 release.

Enable the corresponding catalog feature before declaring a subset. Omit
`codes` to generate all catalog variants. Macros add `Clone`, `Copy`, `Debug`,
`Eq`, `Hash`, `PartialEq`, and Strum's `VariantArray`, `EnumIter`, `EnumCount`,
`FromRepr`, and `IntoStaticStr`; additional derives remain caller-owned.
Import `iso_types::{VariantArray, IntoEnumIterator, EnumCount}` for `VARIANTS`,
`iter()`, and `COUNT`. Catalogs use canonical code order; subsets use declaration
order. `from_repr()` accepts a zero-based ordinal in that order: use parsing for
ISO numeric representations, and use canonical codes for persistence.

`code()` and Strum's `into_str()` return the canonical code in const contexts.
Owned and borrowed enum values also convert into `&'static str`. Maintained
catalogs derive `EnumString` for both `FromStr` and `TryFrom<&str>` with the same
case-sensitive canonical codes, unique aliases, and `ParseIsoCodeError`.

With `locales` and `fluent`, use `#[iso_locale]` on an empty enum in a package
with `i18n.toml` and canonical locale directories. The fallback locale becomes
`Default`. `Languages::VARIANTS` and `Languages::iter()` list supported locales;
`Languages::COUNT` gives their count. Variants convert into
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

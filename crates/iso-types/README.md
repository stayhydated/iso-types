# iso-types

Use `iso-types` for typed ISO, geographic, and measurement catalogs, application-specific
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
| `full` | No | Enable all catalogs and regional relationships, including Jiff zone rules and local MIC imports; integration adapters remain separate. |
| `639-1` | Yes | `Iso6391`: current languages with two-letter identifiers. |
| `639-2` | No | `Iso6392`: current languages with three-letter identifiers. |
| `639-3` | No | `Iso6393`: current living languages, scope, and kind. |
| `3166-1` | Yes | `Iso3166_1`: countries with alpha-3 and numeric representations. |
| `3166-2` | No | `Iso3166_2` and `SubdivisionKind`: typed hierarchy and country navigation; enable `3166-1`. |
| `4217` | Yes | `Iso4217`: active tender currencies, numeric representations, and ICU formatting precision. |
| `15924` | Yes | `Iso15924`: recommended modern scripts and numeric representations. |
| `5218` | Yes | `Iso5218`: human sex interchange codes. |
| `8601` | Yes | `Iso8601Weekday` and `Iso8601Month`: numbered weekdays and months. |
| `units` | No | `MeasurementUnit` and `UnitCategory`: 148 common CLDR measurement identifiers and English names. |
| `unit-conversion` | No | Enable `units` and ICU4X unit conversion, including temperature offsets and reciprocal fuel economy. |
| `regions` | No | `GeographicRegion`: 30 UN M49 regions, hierarchy, and country associations; enable `3166-1`. |
| `regional-preferences` | No | Week, calendar, hour-cycle, measurement-system, and paper-size defaults; enable `3166-1`, `8601`, and `icu`. |
| `unit-preferences` | No | `UnitUsage`: 35 CLDR usage categories with ranked and mixed-unit choices; enable `units` and `3166-1`. |
| `language-relations` | No | Language/country/script associations, population metadata, macrolanguage membership, and ICU directionality; enable `639-1`, `3166-1`, `15924`, and `icu`. |
| `time-zones` | No | `TimeZoneId`: 419 IANA location zones and UTC, country associations, and Jiff's bundled rules; enable `3166-1`. |
| `time-zone-localization` | No | ICU time-zone conversions and localized location names; enable `time-zones` and `icu`. |
| `calling-codes` | No | `CallingCode`: 215 telephone prefixes, country associations, and global service prefixes; enable `3166-1`. |
| `phone-numbers` | No | Re-export `phonenumber` and convert country identifiers for parsing and validation; enable `calling-codes`. |
| `addresses` | No | Address templates, required fields, postal data, typed Fluent labels, and language overrides; enable `3166-1`. |
| `numbering-systems` | No | `NumberingSystem` and inherited CLDR locale numbering preferences; enable `locales`. |
| `216` | No | `Iso216`: ISO A0–A10 and B0–B10 paper dimensions in millimeters. |
| `639-5` | No | `Iso6395`: 115 language groups and their published coding hierarchy. |
| `13616` | No | Registered `IbanFormat` metadata and validated `Iban`; enable `3166-1`. |
| `10383` | No | `Iso10383` identifiers and a date-selected `MarketRegistry` imported from local CSV; enable `3166-1`. |
| `trade-units` | No | `TradeUnit`: 2,161 UNECE Rec 20 codes, including Rec 21 package counts. |
| `packaging` | No | `PackagingType`: 405 current UNECE Rec 21 package codes. |


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
electric quantities, force, acceleration, concentration, graphics, light, magnetic quantities,
and torque. Parse canonical CLDR core IDs such as `meter`
or `gallon-imperial`. Unit symbols belong to localized formatting. English
`Display` uses CLDR display names, such as `meters`; `iso_enum(standard = "units",
codes = ["meter", "foot"])` declares a subset.

`converter_to::<f64>()` creates a reusable ICU converter. `icu_unit()` exposes
the cached parsed ICU unit, and values convert into ICU's `MeasureUnit`.
Incompatible conversions return `UnitConversionError` with the source and target.
Unit categories group the catalog; compatibility follows ICU's physical
dimensions. ICU4X's unit APIs use the pinned experimental 0.6 release.

With `full`, query regional data using the same country identifiers:

```rust
# #[cfg(feature = "full")] {
use iso_types::{AddressField, AddressFieldLabel, CallingCode, GeographicRegion,
    Iso3166_1, Iso6391, Iso15924, Iso8601Weekday, TimeZoneId, UnitUsage};

assert_eq!(Iso3166_1::Us.regional_preferences().week.first_weekday, Iso8601Weekday::Sunday);
assert_eq!(Iso3166_1::Us.geographic_region(), Some(GeographicRegion::NorthernAmerica));
assert!(Iso3166_1::Ca.languages().contains(&Iso6391::Fr));
assert!(Iso6391::Sr.scripts().contains(&Iso15924::Latn));
assert!(CallingCode::C1.countries().contains(&Iso3166_1::Ca));
let address = Iso3166_1::Us.address_metadata().expect("pinned US address schema");
assert_eq!(address.field_label(AddressField::AdministrativeArea), AddressFieldLabel::State);
let weather = Iso3166_1::Us.unit_preferences(UnitUsage::TemperatureWeather);
assert_eq!(weather[0].units[0].code(), "fahrenheit");
let instant = "2026-07-15T12:00Z".parse()?;
assert_eq!(TimeZoneId::AmericaNewYork.at(instant)?.offset().seconds(), -4 * 3600);
# }
# Ok::<(), Box<dyn std::error::Error>>(())
```

`TimeZoneId::time_zone()` exposes Jiff's transition and ambiguity-resolution APIs.
Use `to_ambiguous_zoned()` to resolve DST gaps and overlaps explicitly.
`time-zone-localization` adds ICU identity conversions and
`localized_name(&locale)`. English catalog names use ICU location formatting.

`regional_preferences()` returns CLDR country defaults using ICU calendar,
hour-cycle, and measurement-system types. Explicit user preferences take precedence.
`geographic_regions()` returns the UN M49 chain from World to the most specific
recorded region; `geographic_region()` returns an `Option`. Regions supply
`parent()`, `children()`, and `countries()`.

Language `scripts()`/`countries()` and script `languages()` navigate associations
within the common scope. Country `language_usage()` preserves estimated functional
population, including second-language users, and CLDR's practical government-use
status. Features `639-2`/`639-3` add country `languages_6392()`/`languages_6393()`
and reverse lookups. Script `direction()` delegates to ICU4X.

`unit_preferences(usage)` returns ranked choices with ordered mixed-unit
components, thresholds in the largest component, and formatting skeletons. The
final choice handles smaller quantities. `PreferredUnit::catalog_unit()` identifies
named catalog units; `icu_unit()` parses every component with `unit-conversion`.
Apply the skeleton's rounding when choosing units for display.

Country `calling_codes()` and prefix `countries()` preserve shared prefixes and
global services. With `phone-numbers`, convert a country to
`phonenumber::country::Id` using `TryFrom`, then use `parse()` and `is_valid()`.

`address_metadata()` provides native/Latin templates, required fields, and postal
data. `fields` preserves native field order; `uses_postal_code()` reports usage.
`field_label()` returns a typed Fluent label such as State, Province, or Eircode.
`label_override(field, language)` retrieves exact upstream language overrides and
structured field labels. Postal patterns use Java-compatible regular expressions.

Enable the corresponding catalog feature before declaring a subset. Omit
`codes` to generate all catalog variants. `country = "US"` filters `3166-2`;
`category = "length"` filters `units` and `unit-preferences`. Filters intersect
with explicit `codes`; an empty result is an error. Macros add `Clone`, `Copy`,
`Debug`,
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

With `numbering-systems`, `216`, `639-5`, `trade-units`, `packaging`, and `units`:

```rust
# #[cfg(all(feature = "numbering-systems", feature = "216", feature = "639-5", feature = "trade-units", feature = "packaging", feature = "units"))] {
use iso_types::{numbering_preferences, Iso216, Iso6395, MeasurementUnit, NumberingSystem, PackagingType, TradeUnit};

assert_eq!(numbering_preferences("ar-MA")?.default, NumberingSystem::Latn);
assert_eq!(Iso216::A4.width_mm(), 210);
assert_eq!(Iso216::A4.height_mm(), 297);
assert_eq!(Iso6395::Gmw.parent(), Some(Iso6395::Gem));
assert_eq!(TradeUnit::Kgm.catalog_unit(), Some(MeasurementUnit::Kilogram));
assert_eq!(PackagingType::Bx.trade_unit(), TradeUnit::Xbx);
# }
# Ok::<(), Box<dyn std::error::Error>>(())
```

Numbering metadata distinguishes decimal digits from algorithmic CLDR rules.
`numbering_preferences()` accepts a language identifier; apply explicit user
numbering choices separately. Language `macrolanguage()` and `members()` expose
IANA membership within each selected language catalog under `language-relations`.
ISO 639-5 `parent()`, `children()`, and `ancestors()` describe the published group
hierarchy. Individual language membership is a separate relationship.

Trade records preserve source descriptions, symbols, and conversion-factor text.
With `units`, explicit `catalog_unit()`/`trade_codes()` links connect equivalent
CLDR units; ICU4X performs conversions. Package counts link through
`packaging_type()`/`trade_unit()` when both trade features are enabled.

With `13616`, parse an electronic or space-grouped IBAN:

```rust
# #[cfg(feature = "13616")] {
use iso_types::{Iban, Iso3166_1};

let account: Iban = "GB82 WEST 1234 5698 7654 32".parse()?;
assert_eq!(account.country(), Iso3166_1::Gb);
assert_eq!(account.bank_identifier(), Some("WEST"));
assert_eq!(account.as_str(), "GB82WEST12345698765432");
# }
# Ok::<(), Box<dyn std::error::Error>>(())
```

`IbanFormat::ALL` contains 88 registered country-prefix formats from release 103.
Country `iban_format()` includes explicit territory associations. Parsing checks
registered length, BBAN character classes, check digits, and MOD 97. Account
existence and domestic validation belong to the bank.

With `10383`, import a MIC CSV that the application is authorized to use:

```rust,no_run
# #[cfg(feature = "10383")] {
use iso_types::{Iso10383, MarketRegistry};

let file = std::fs::File::open("registry.csv")?;
let registry = MarketRegistry::from_csv(file, "2026-10-15".parse()?)?;
let code: Iso10383 = "XNAS".parse()?;
if let Some(market) = registry.get(code) {
    println!("{}: {}", market.code, market.name());
}
# }
# Ok::<(), Box<dyn std::error::Error>>(())
```

MIC parsing validates four uppercase ASCII letters/digits. The local registry
resolves membership and operating/segment relationships at its selected date.
Descriptions and reported status retain the supplied publication's values.
Use [`iso-types-locations`](https://github.com/stayhydated/iso-types/tree/master/crates/iso-types-locations)
for the separately packaged UN/LOCODE directory.

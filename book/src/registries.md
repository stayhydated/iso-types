# Registry data

Choose the feature for the data you need. Smaller catalogs live in `iso-types`;
the larger transport location directory lives in `iso-types-locations`. MIC
lookup uses a CSV supplied by your application. The [registry example](../examples/registries/)
demonstrates IBAN checks, location alternatives, and local MIC imports.

## Select a catalog

| Feature | Type or function | Use |
| --- | --- | --- |
| `units` | `MeasurementUnit` | 148 common CLDR identifiers across 22 categories. |
| `numbering-systems` | `NumberingSystem`, `numbering_preferences()` | Decimal digits, algorithmic rules, and inherited locale defaults. |
| `216` | `Iso216` | ISO A0–A10 and B0–B10 portrait dimensions in millimeters. |
| `639-5` | `Iso6395` | Published language group hierarchy. |
| `language-relations` | Language `macrolanguage()` and `members()` | IANA membership within the selected language catalog. |
| `13616` | `Iban`, `IbanFormat` | Registered national formats and MOD 97 validation. |
| `10383` | `Iso10383`, `MarketRegistry` | Identifiers and local, date-selected market lookup. |
| `trade-units` | `TradeUnit` | UNECE Rec 20 codes, including X-prefixed package counts. |
| `packaging` | `PackagingType` | UNECE Rec 21 packaging identifiers. |

The five additional closed catalogs accept the same `#[iso_enum]` syntax as
countries: use standards `numbering-systems`, `216`, `639-5`, `trade-units`, and
`packaging`. Enable the corresponding feature first. `catalog_value()` exposes
each maintained type's metadata and relationships.

## Resolve numbering and paper preferences

With `numbering-systems`, call `numbering_preferences("ar-MA")` to get the
inherited CLDR default, native, traditional, and finance systems. The default for
Morocco's Arabic locale is `latn`, while its native system is `arab`.
`NumberingSystem::digits()` supplies ten characters in value order for numeric
systems; `rules()` identifies the CLDR RBNF ruleset for algorithmic systems.
These identifiers describe data for a formatter to consume.

The input is a language identifier such as `zh-Hant-TW`. Apply an explicit user
numbering choice separately. Malformed identifiers and components outside the
current/common locale scope return `LocaleConversionError`.

With `216`, `Iso216::A4` has `width_mm() == 210` and `height_mm() == 297`.
`pwg_name()` returns its dimension-bearing IANA IPP keyword. With
`regional-preferences`, `PaperSize::iso216()` connects a regional A4 default to
the catalog; US Letter remains its own regional paper choice.

## Navigate languages

Enable `language-relations` and the language catalogs you use. For example,
`Iso6393::Cmn.macrolanguage()` returns `Some(Iso6393::Zho)`, while the individual
identifier remains `cmn`. `members()` is the reverse relationship. A slice
contains members within the selected catalog's current/common scope.

With `639-5`, `Iso6395::Gmw.parent()` is `Some(Iso6395::Gem)`, and its ancestors
are `Ine` then `Gem`. Group hierarchy follows the Library of Congress coding
table. This hierarchy and individual language macrolanguage membership answer
different questions; use the appropriate relationship for your picker.

## Validate an IBAN

Enable `13616` and parse into `Iban`:

```rust
use iso_types::{Iban, Iso3166_1};

let account: Iban = "GB82 WEST 1234 5698 7654 32".parse()?;
assert_eq!(account.as_str(), "GB82WEST12345698765432");
assert_eq!(account.country(), Iso3166_1::Gb);
assert_eq!(account.bank_identifier(), Some("WEST"));
assert_eq!(account.branch_identifier(), Some("123456"));
# Ok::<(), Box<dyn std::error::Error>>(())
```

Parsing accepts lowercase ASCII and ordinary spaces, then stores uppercase
electronic form. It checks the registered prefix, national length, BBAN character
classes, check digit range, and MOD 97. `ParseIbanError` distinguishes these
failures. Account existence and domestic check algorithms require bank services.

`IbanFormat::ALL` exposes 88 selected formats from Swift registry release 103,
including structure, effective date, examples, and bank/branch positions.
Country `iban_format()` resolves explicit territory associations: Guernsey uses
the GB format, for example. The account itself must use the registered GB prefix.

## Import a MIC registry

Enable `10383` and supply an authorized UTF-8 CSV in the official column layout.
The runnable `market_registry` example takes a local path and effective date:

```sh
cargo run -p iso-types --example market_registry --features 10383 -- registry.csv 2026-10-15
```

In an application, call `MarketRegistry::from_csv(reader, date)`, then use
`get(code)`, `for_country(country)`, or `segments(operating_mic)`.
`Iso10383` validates four uppercase ASCII letters/digits; registry membership
comes from the supplied CSV. The importer preserves all source columns,
quoted names, and embedded line breaks.

Creation and expiry dates select effective identities. Names and reported status
retain the supplied publication's values, so importing a later publication with
an earlier date does not reconstruct historical descriptions. Operating and
segment links must form a closed graph at the selected date. Missing columns,
duplicates, malformed records, and unavailable operating MICs return
`MarketRegistryError`.

The browser example processes pasted CSV locally. The application supplies the
snapshot and its authorization to use it.

## Look up transport locations

Depend on `iso-types-locations` to query release 2025-1 of UN/LOCODE:

```rust
use iso_types_locations::UnLocode;

let location: UnLocode = "FIHEL".parse()?;
for listing in location.records() {
    println!("{}: {} — {}", location, listing.name, listing.function_code);
}
# Ok::<(), iso_types::ParseIsoCodeError>(())
```

There are 116,021 selected identities and 116,167 listings. `records()` returns
every published alternative, preserving differing names, functions, status,
subdivisions, and coordinates. `iter()` enumerates all codes; `for_country()`
restricts them to a typed country.

`functions()` decodes the source function mask. `coordinates()` parses whole
degree/minute coordinates when valid, and `coordinate_text` preserves original
source text. With the package's `subdivisions` feature, `subdivision()` resolves
current ISO 3166-2 identities where available. Unknown codes return
`ParseIsoCodeError`. Store canonical codes for persistence.

## Connect trade and measurement data

Enable `trade-units` for 2,161 selected trade codes and `packaging` for 405 package
types. `PackagingType::Bx` is Box; its `trade_unit()` is `TradeUnit::Xbx` when
both features are enabled. Rec 20's X prefix represents counts of Rec 21 packages.
`packaging_type()` provides the reverse link.

With `units`, `TradeUnit::Kgm.catalog_unit()` is
`Some(MeasurementUnit::Kilogram)`. `MeasurementUnit::trade_codes()` returns the
explicit reverse crosswalk. A missing link means the project has no explicit
equivalence for that code. Source conversion factors remain descriptive text;
enable `unit-conversion` to perform conversions through ICU4X.

## Data sources

The repository pins inputs, SHA-256 checksums, releases, retrieval dates, and
selection dates in `data/registries/provenance.json`. Generated packages contain
their selected tables. `NOTICE` attributes the authorities and describes the
derived selections. Numbering data comes from CLDR; language group names from
the Library of Congress; trade and location records from UNECE; registered IBAN
format information from Swift. The UNECE derived selections carry CC BY 4.0
attribution. MIC snapshots belong to the importing application.

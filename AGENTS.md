# Working in iso-types

Use the fff MCP tools for all file search operations.

Start with `just --list`; `just verify` runs generation checks, codegen tests,
Rust formatting, the feature test matrix, Clippy, and documentation. Codegen uses
Python 3.10 or newer and the pinned Rust toolchain.

## Ownership

| Surface | Owner |
| --- | --- |
| Public ISO contract and errors | `crates/iso-types/src/code.rs`; exports in `src/lib.rs`. |
| Catalog enums and tables | Generated `crates/iso-types/src/catalogs.rs` and `src/catalogs/`. |
| ICU conversions | `crates/iso-types/src/icu.rs`. |
| Country, subdivision, and currency relationships | Generated `crates/iso-types/src/geography.rs`; model validation in `tools/codegen/relations.py`. |
| Common measurement units | Generated `src/catalogs/measurement_unit.rs`; selection in `tools/codegen/units.py`; ICU conversions in `src/measurement.rs`. |
| Locale parsing and English names | `crates/iso-types/src/locale.rs`. |
| English Fluent utility | `crates/iso-types/src/i18n.rs`; owned resources under `i18n/en/iso-types/`. |
| Macro input conventions | `crates/iso-types-macros/src/enum_input.rs`. |
| Catalog subset expansion | `crates/iso-types-macros/src/catalog/`: cached data, validated model, and token emission. |
| Configured locale expansion | `crates/iso-types-macros/src/locale/`: validated model and token emission. |
| Real consumer tests | `tests/locale-consumer`: renamed dependency, custom inventory, conversions, and macro hygiene. |
| ICU snapshot extraction | `tools/icu-data`: pinned compiled providers. |
| Data selection and generation | `tools/codegen`: catalogs, current policy, Rust rendering, and output verification. |

Paths in a crate's row are relative to that crate where the prefix is omitted.
Generated `src/locale/scope.rs` files carry the same selection policy into both
public packages. The macro crate's `src/catalog/data.json` is a generated selection
index; labels and metadata remain owned by the facade's catalog tables.
`src/i18n/resources.rs` is generated registration glue.

Catalogs and macro-generated enums derive Strum's `VariantArray`, `EnumIter`,
`EnumCount`, `FromRepr`, and `IntoStaticStr`. The facade re-exports Strum and
the enumeration traits; use `VARIANTS`, `iter()`, and `COUNT`. Canonical codes
use `const_into_str`; `from_repr()` follows zero-based declaration ordinals.
Macro derives resolve their Strum path through the caller's facade dependency
name. Maintained catalogs derive `EnumString` with explicit canonical codes,
unique aliases, and `ParseIsoCodeError`. The `3166-2` feature enables Strum PHF
parsing after benchmarking it against binary search.

## Data and labels

Pinned ISO snapshots and their checksums live in `data/iso`. CLDR/SIX selection
metadata, checksums, retrieval dates, and the snapshot's `as_of` date live in
`data/current`. `data/icu/english.json` records names extracted from the locked
ICU4X providers, including currency formatting precision and cash rounding.
Complete upstream inputs stay at repository scope; published
crates contain their generated tables and selection index.

ICU4X owns English names for languages, countries, scripts, currencies, months,
and weekdays. Exclude codes whose ICU English name is absent; do not substitute
ISO labels or codes. Current subdivisions and sex-code labels are owned by the
single-line literal English FTL resources. Duplicate FTL keys and dynamic values
are errors. Fluent lookup and `Display` share the same label authority.
Measurement-unit display names come directly from pinned CLDR English data.
`units.py` selects common regular CLDR core identifiers and requires every name.
The `units` feature supplies the catalog; `unit-conversion` delegates conversion
factors, temperature offsets, and reciprocal rules to ICU4X. Unit categories use
CLDR's grouping; ICU determines conversion compatibility.

`tools/codegen/policy.py` owns the current/common selection rules. Living
languages require CLDR modern locale coverage and a recommended default script.
Scripts require CLDR regular validity and recommended identifier usage.
Countries and subdivisions require current regular codes; currencies require
active tender circulation and presence in SIX's current ISO 4217 list. Locale
APIs and macros check components before and after ICU normalization and reject
variants and excluded representations.

`relations.py` validates country ownership, closed parent references, and acyclic
subdivision hierarchies before rendering. Country/subdivision navigation and
country/currency slices are const; relationships preserve canonical code order.
Currency associations are active tender at the dated snapshot and permit several
currencies per country. `SubdivisionKind` preserves ISO source category names.

## Regeneration and validation

Edit authoritative inputs or generator code first, then run `just generate`.
The generator extracts ICU names, validates catalogs, and formats every output
before replacing changed files. `just generate-check` checks all generated
outputs without modifying them and reports every differing path. Root `NOTICE`
and `LICENSE-*` files are the source for the copies shipped in each public crate.

Use `python3 -m tools.codegen --import-data /path/to/iso-codes` to snapshot a new
ISO checkout and regenerate. Refresh CLDR/SIX metadata and its provenance when
updating the dated selection policy. Review selected-code changes with the
catalog counts and scope tests.

Use the narrowest relevant validation while editing:

- `just codegen-test` for selection, ranges, literals, and output plans.
- `cargo test -p iso-types-macros --all-features --locked` for macro models
  and diagnostics.
- `cargo test -p iso-types-locale-consumer --locked` for real macro/runtime integration.
- `just test` for default, minimal, and full Fluent configurations.
- `just verify` before committing cross-cutting changes.
- `just package` for a public-package dry run; it verifies archives without uploading.

# Working in iso-types

Use the fff MCP tools for all file search operations.

`crates/iso-types` owns ISO enums and runtime integration.
`crates/iso-types-macros` owns catalog subset and configured locale expansion.
`crates/iso-types-macros/data` contains pinned upstream source snapshots.

ICU4X owns English names for languages, countries, scripts, currencies, months,
and weekdays. `tools/icu-data` extracts pinned compiled provider data into
`crates/iso-types-macros/data/icu_english.json`. Exclude codes whose ICU English
name is absent; do not substitute ISO names or codes. English FTL under
`crates/iso-types/i18n/en/iso-types` owns current subdivision and sex-code labels,
following koruma-collection's FTL-to-Display
convention.

`scripts/current_data.py` owns the current/common selection policy. Living
languages require CLDR modern locale coverage and a recommended default script.
Scripts require CLDR regular status and recommended identifier usage. Countries
and subdivisions require regular current validity; currencies require active
tender circulation and presence in SIX's current ISO 4217 list. Locale APIs and
macros share generated policy tables and reject excluded components and variants
before ICU normalization. Bibliographic aliases are excluded.

Selection sources are pinned under `crates/iso-types-macros/data/current` with
checksums and an `as_of` date. Refresh and review those inputs when updating the
data; do not hand-edit generated policy tables. Published macro packages contain
the selected catalog and policy tables; complete source snapshots stay in the
repository for reproducibility.

Run `python3 scripts/generate.py` after changing source data or an owned FTL label.
It runs the ICU extractor and regenerates Rust tables, macro records, and resource
registration. `python3 scripts/generate.py --check` checks the ICU snapshot against
the locked providers before verifying generated output. Generated Rust files
identify their source; change the generator and authoritative data first.

Validate default, minimal, and full Fluent configurations:

- `cargo test --workspace --locked`
- `cargo test -p iso-types --no-default-features --locked`
- `cargo test --workspace --all-features --locked`
- `cargo clippy --workspace --all-features --all-targets --locked -- -D warnings`

# Working in iso-types

Use the fff MCP tools for all file search operations.

`crates/iso-types` owns ISO enums and runtime integration.
`crates/iso-types-macros` owns catalog subset and configured locale expansion.
`crates/iso-types-macros/data` contains pinned upstream source snapshots.

English FTL under `crates/iso-types/i18n/en/iso-types` owns display labels, following
koruma-collection's FTL-to-Display convention. Run `python3 scripts/generate.py`
after editing a label. Check reproducibility with
`python3 scripts/generate.py --check`. Generated Rust files identify their source;
change the generator and source data before regenerating them.

Validate default, minimal, and full Fluent configurations:

- `cargo test --workspace --locked`
- `cargo test -p iso-types --no-default-features --locked`
- `cargo test --workspace --all-features --locked`
- `cargo clippy --workspace --all-features --all-targets --locked -- -D warnings`

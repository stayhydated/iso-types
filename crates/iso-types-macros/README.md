# iso-types-macros

The macro implementation behind [iso-types][facade]. Applications use that
facade's `iso_enum` and `iso_locale` re-exports; direct dependencies serve macro
implementation and expansion tooling.

Requires Rust 1.99 or newer.

## Features

| Feature | Default | Purpose |
| --- | --- | --- |
| `fluent` | No | Generate Fluent message implementations through the iso-types facade. |
| `locales` | No | Compile the configured locale macro using es-fluent-toml. |

`iso_enum` accepts `standard = "3166-1"` and an optional canonical
`codes = ["US", "CA"]` list. `iso_locale` reads the invoking package's
`i18n.toml`; its optional `custom` mode assigns label ownership to that package.
Both require an empty enum without generics.

[facade]: https://github.com/stayhydated/iso-types/tree/master/crates/iso-types

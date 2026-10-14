# Using the examples

The [example gallery](../examples/) runs iso-types in Dioxus 0.7. Each page
uses typed `es-fluent` messages and `es-fluent-manager-dioxus` for reactive
English and French interface text. Open **Rust source** on a page to read the
component that produces its controls and results.

Select **Interface language** and choose French. The interface changes without
resetting your inputs. Catalog names retain their authoritative English labels;
the time-zone example also demonstrates ICU's localized location names.

To run the gallery from a checkout, use Rust 1.99, Dioxus CLI 0.7, `just`, and Bun
for static preview. The WASM target must be installed for the Rust toolchain.

```sh
just web-build
cargo xtask preview web
```

Open the preview URL ending in `/iso-types/`, then choose **Demos**. For live
development, `just web` starts Dioxus after assembling the static documentation.
`just web-i18n` checks the web package's message registration and both languages.

The application registers its package's assets with
`es_fluent_manager_dioxus::define_i18n_module!()` in a library module. Its root
`DioxusAssetI18nProvider` persists across routes; components obtain the active
context with `use_i18n()` and render typed values with `localize_message`.
`#[iso_locale]` creates the language choices from the configured locale folders.

The source is divided into one component module per data area in
[`web/src/examples`](https://github.com/stayhydated/iso-types/tree/master/web/src/examples).
Shared controls use `SearchableSelect` from the pinned stayhydated components.
Each example enables the library features it demonstrates through the web
package's manifest.

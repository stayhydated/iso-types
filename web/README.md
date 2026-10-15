# iso-types examples

Interactive examples for every iso-types catalog and regional data area, built
with Dioxus and reactive English/French messages from es-fluent-manager-dioxus.
Requires Rust 1.99 or newer and Dioxus 0.7.

The [gallery](https://stayhydated.github.io/iso-types/examples/) covers country
and subdivision choices, address labels, currencies, languages and scripts,
unit conversion and preferences, regional defaults, UN regions, IANA time zones,
phone numbers, IBAN checks, transport locations, local MIC imports, and all 22
catalogs. Each page exposes its complete Rust source.

Catalog data retains the library's authoritative English names. App messages
switch languages through the Dioxus provider, and ICU supplies localized
time-zone location names.

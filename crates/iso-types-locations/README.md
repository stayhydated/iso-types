# iso-types-locations

Use `iso-types-locations` for the pinned UN/LOCODE directory, with typed ISO
countries, transport functions, status codes, coordinates, and original source
metadata. Depend on this package directly when your application needs location
lookup. It shares country types with `iso-types` and preserves every alternative
listing for each location identity.

Requires Rust 1.99 or newer.

## Features

No features are enabled by default.

| Feature | Default | Purpose |
| --- | --- | --- |
| `subdivisions` | No | Resolve published subdivision suffixes into `iso_types::Iso3166_2` where a current mapping exists. |

## Example

```rust
use iso_types::Iso3166_1;
use iso_types_locations::UnLocode;

let helsinki: UnLocode = "FIHEL".parse()?;
assert_eq!(helsinki.country(), Iso3166_1::Fi);
for listing in helsinki.records() {
    println!("{}: {} ({})", helsinki, listing.name, listing.function_code);
}
assert!(UnLocode::for_country(Iso3166_1::Fi).any(|place| place == helsinki));
# Ok::<(), iso_types::ParseIsoCodeError>(())
```

Release 2025-1 supplies 116,021 selected identities and 116,167 source listings.
`iter()` and `for_country()` enumerate canonical codes. `records()` preserves
alternative names and differences in functions, status, subdivisions, and
coordinates. Source names retain their published spelling. `coordinates()`
returns signed whole-minute coordinates when the source value can be parsed;
`coordinate_text` preserves the original text.

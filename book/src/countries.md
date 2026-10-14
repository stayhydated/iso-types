# Country and address choices

Use `Iso3166_1::subdivisions()` to populate a country-dependent picker. For
Canada, the returned entries include provinces and territories; for the United
States, they include states and DC. `kind()` describes each individual entry.

The geography example keeps the country and optional subdivision together. On a
country change it retains an existing valid choice or selects the new country's
first root subdivision. Countries with no subdivisions show an empty state.
Selecting a subdivision also shows its parent and children where ISO supplies
those relationships. Root subdivisions follow ISO's hierarchy.

For an address form, ask `country.address_metadata()` for the country's schema.
`field_label(AddressField::AdministrativeArea)` returns a typed label such as
`State` or `Province`; `fields` gives the source order and `is_required` supplies
the required status. Postal fields and labels also vary: Ireland uses Eircode,
India uses PIN, and Hong Kong omits a postal-code field.

The address preview rebuilds its fields when the country changes. It exposes
the original template and postal examples alongside the form. Source templates
and Java-compatible postal expressions need an appropriate formatter or
validation engine before being used for address submission.

Country-to-currency and country-to-time-zone APIs return collections. Preserve
multiple choices instead of assuming each country has one currency or zone.
The currency page shows current tender and lets you inspect precision and
cash-rounding metadata. The time-zone page selects a country-associated IANA
zone and uses Jiff to convert an explicit instant, including daylight saving.

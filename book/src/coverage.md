# Catalog and data coverage

The gallery's ten pages cover every maintained catalog and relationship area.
The catalog browser uses Strum iteration and the shared `IsoCode` contract for
all 17 catalogs. Enter a code or supported alias to see its canonical spelling,
or filter the complete list by code or English name.

| Page | Catalogs and APIs |
| --- | --- |
| Countries & subdivisions | ISO 3166-1 and 3166-2; subdivision kinds, parents, and children. |
| Address fields & labels | Address fields and labels; country schemas, required fields, templates, and postal examples. |
| Currencies | ISO 4217; country tender relationships, formatting precision, and cash rounding. |
| Languages & scripts | ISO 639-1 and 15924; countries, scripts, directionality, and CLDR population estimates/status. |
| Measurement units | Measurement units and unit usages; ICU conversion and ranked CLDR preferences with thresholds and skeletons. |
| Regional preferences | ISO weekdays; week conventions, clock cycles, calendars, measurement systems, and paper. |
| Geographic regions | UN M49 hierarchy, region countries, and country ancestry. |
| Time zones | IANA identities; country associations, Jiff transitions, ICU mappings, and localized names. |
| Telephone data | Calling codes, shared/global code relationships, and phone parsing/validation. |
| All catalogs & subsets | All of the above plus ISO 639-2, ISO 639-3, ISO 5218 sex codes, and ISO months; typed subset macros. |

The subset examples select Canada/US markets, US subdivisions, and length units
with `#[iso_enum]`. Their generated enums use `VARIANTS`, `iter()`, and `COUNT`
from Strum, and `catalog_value()` reconnects each subset to the maintained data.

Data follows the repository's pinned current/common scope. Unit preferences are
ranked metadata, including mixed components and formatting rules. The conversion
example delegates numerical conversion to ICU4X and reports incompatible units.
Regional defaults can be overridden by application users. CLDR language
population figures include second-language use.

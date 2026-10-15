## Example pages


## Interface


## CatalogCount

catalog_count = { $visible } of { $total } entries

## Page

page-Addresses = Address fields & labels
page-Catalogs = All catalogs & subsets
page-Currencies = Currencies
page-Geography = Countries & subdivisions
page-Languages = Languages & scripts
page-Phone = Telephone data
page-Preferences = Regional preferences
page-Regions = Geographic regions
page-TimeZones = Time zones
page-Units = Measurement units

## QuantityResult

quantity_result = { $amount } { $unit }

## Ui

ui-AddressNote = Try US, Canada, Ireland, India, and Hong Kong. This preview follows the source field order.
ui-AddressesIntro = The country selects the address fields, their required status, and labels such as State, Province, ZIP, or Eircode.
ui-AllCurrencies = Showing all active currencies. Choose any currency independently of a country.
ui-Calendars = Preferred calendars
ui-CallingCode = Calling code
ui-CallingCodes = Country calling codes
ui-CashDigits = Cash fraction digits
ui-CashRounding = Cash rounding increment
ui-Catalog = Catalog
ui-CatalogsIntro = Browse every maintained catalog and parse a canonical code or supported alias. Subsets use the same Strum iteration and parsing APIs.
ui-Children = Children
ui-Code = Code
ui-Components = Unit components
ui-Countries = Countries
ui-Country = Country
ui-CurrenciesIntro = Explore active tender by country, currency precision, cash rounding, and other countries using the same currency.
ui-Currency = Currency
ui-CurrencyCountries = Countries using this currency
ui-CurrencyNote = A country can use several currencies. Precision is formatting metadata; rounding increments are measured in the smallest displayed unit.
ui-DetectedCountry = Number region
ui-Digits = Fraction digits
ui-Direction = Text direction
ui-EnglishLabels = Catalog names follow their authoritative English sources. Interface messages switch languages.
ui-EnglishName = English name
ui-Examples = Explore the catalogs
ui-Filter = Filter entries
ui-FirstWeekday = First weekday
ui-FromUnit = From unit
ui-GeographyIntro = Choose a country, then one of its subdivisions. Canada includes provinces and territories; the US includes states and DC.
ui-GlobalCode = Global service code
ui-HourCycle = Hour cycle
ui-IncompatibleUnits = These units cannot be converted.
ui-Instant = UTC instant (RFC 3339)
ui-Intro = Real data, typed choices. Change a value and see the relationships update.
ui-InvalidInstant = Enter a valid timestamp with an offset, such as 2026-07-01T12:00:00Z.
ui-InvalidNumber = Enter a finite number.
ui-InvalidPhone = The number is invalid or cannot be parsed for this region.
ui-Kind = Subdivision category
ui-Language = Interface language
ui-LanguageCode = Language
ui-LanguageNote = CLDR estimates include second-language use. The catalog browser also covers ISO 639-2 and 639-3.
ui-LanguagesIntro = Explore country language estimates, language-to-script relationships, and ICU text direction.
ui-LeftToRight = Left to right
ui-Level = Hierarchy level
ui-MeasurementSystem = Measurement system
ui-MinWeekDays = Minimum days in week one
ui-NoData = No entries for this selection.
ui-NoMatches = No matches
ui-None = None
ui-Optional = Optional
ui-Paper = Paper size
ui-Parent = Parent
ui-ParseInput = Code or alias to parse
ui-PhoneIntro = Country calling codes can be shared. Parse a sample phone number using the selected country as its default region.
ui-PhoneNumber = Phone number
ui-Population = Population estimate (%)
ui-PostalExamples = Postal code examples
ui-PreferencesIntro = Inspect regional defaults for weeks, clocks, calendars, measurement, and paper. User preferences can override these defaults.
ui-Quantity = Quantity
ui-Region = Geographic region
ui-RegionsIntro = Navigate the UN M49 hierarchy and inspect each region’s countries.
ui-Required = Required
ui-RestrictToCountry = Limit currencies to a country
ui-Result = Result
ui-RightToLeft = Right to left
ui-Script = Script
ui-Search = Search options…
ui-SharedCountries = Countries sharing this code
ui-Skeleton = Number formatting skeleton
ui-Source = Rust source
ui-Status = Source status
ui-Subdivision = State / province / territory
ui-Subset = Application subset
ui-TemperatureSystem = Temperature system
ui-Template = Address template
ui-Threshold = Lower bound in the first unit
ui-TimeZone = Time zone
ui-TimeZonesIntro = Select a country and an IANA time zone. Jiff converts your UTC instant using the bundled transition rules; ICU localizes the zone name.
ui-ToUnit = To unit
ui-UnitNote = Preferences are ranked source data. Thresholds use the first component; mixed units and rounding rules need a formatter.
ui-UnitsIntro = Convert quantities through ICU4X, then inspect the ranked CLDR unit choices for a country and usage.
ui-UnknownCode = Unrecognized code in this catalog.
ui-Usage = Measurement usage
ui-Weekend = Weekend
ui-ZoneName = Localized time-zone name
ui-ZoneUnavailable = Time-zone data could not be loaded.
ui-ZonedTime = Local date and time

page-Registries = Accounts, locations & markets
ui-Macrolanguage = Macrolanguage
ui-Members = Member languages
ui-LanguageGroup = Language group (ISO 639-5)
ui-Locale = Locale identifier
ui-DefaultNumbering = Default numbering
ui-NativeNumbering = Native numbering
ui-TraditionalNumbering = Traditional numbering
ui-FinanceNumbering = Finance numbering
ui-InvalidLocale = Enter a supported language identifier, such as ar-MA.
ui-TradeCodes = Equivalent trade unit codes
ui-RegistriesIntro = Validate an IBAN, inspect published transport locations, and look up markets in your own MIC registry.
ui-Iban = IBAN
ui-BankIdentifier = Bank identifier
ui-BranchIdentifier = Branch identifier
ui-InvalidIban = The IBAN format or checksum is invalid.
ui-IbanNote = Validation checks the registered country format and MOD 97 checksum. Account existence and domestic checks require the bank.
ui-LocationCode = Location code
ui-LocationName = Published name
ui-Functions = Function mask
ui-Coordinates = Coordinates
ui-LocationNote = Every published listing is shown, including alternative names and differing metadata. Coordinates retain whole-minute source precision.
ui-MicRegistry = Market registry (MIC)
ui-MicNote = Paste a MIC CSV you are authorized to use. It is processed locally in this browser. The selected date filters creation and expiry; names and status retain the supplied publication’s values.
ui-MicCsv = MIC registry CSV
ui-AsOf = Effective date
ui-InvalidDate = Enter a valid date.
ui-MarketName = Market name
ui-OperatingMic = Operating MIC

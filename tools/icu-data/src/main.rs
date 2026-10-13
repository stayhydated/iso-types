//! Extract English names and currency formatting metadata from pinned ICU4X providers.
use icu_datetime::provider::names::{
    DatetimeNamesMonthGregorianV1, DatetimeNamesWeekdayV1, MonthNames,
};
use icu_experimental::dimension::provider::currency::extended::CurrencyExtendedDataV1;
use icu_experimental::dimension::provider::currency::fractions::{CurrencyFractionsV1, Rounding};
use icu_locale::{
    Locale, LocaleCanonicalizer, data_locale, locale,
    names::{
        LanguageIdentifierDisplayName, LanguageIdentifierDisplayNameOptions, RegionDisplayName,
        ScriptDisplayName,
    },
};
use icu_plurals::PluralRules;
use icu_provider::prelude::*;
use serde_json::{Value, json};
use std::{collections::BTreeMap, error::Error, path::Path};
use writeable::{TryWriteable, Writeable};

fn request<'a>(attributes: &'a str, display_locale: &'a DataLocale) -> DataRequest<'a> {
    let mut metadata = DataRequestMetadata::default();
    metadata.silent = true;
    DataRequest {
        id: DataIdentifierBorrowed::for_marker_attributes_and_locale(
            DataMarkerAttributes::from_str_or_panic(attributes),
            display_locale,
        ),
        metadata,
    }
}

fn name_or_missing<T: std::fmt::Display>(
    result: Result<T, DataError>,
) -> Result<Option<String>, DataError> {
    match result {
        Ok(name) => Ok(Some(name.to_string())),
        Err(error) if error.kind == DataErrorKind::IdentifierNotFound => Ok(None),
        Err(error) => Err(error),
    }
}

fn rounding_increment(rounding: Rounding) -> Result<u8, Box<dyn Error>> {
    match rounding {
        Rounding::R1 => Ok(1),
        Rounding::R5 => Ok(5),
        Rounding::R20 => Ok(20),
        Rounding::R50 => Ok(50),
        _ => Err("unsupported ICU currency rounding increment".into()),
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    match arguments.as_slice() {
        [argument] if argument == "--stdout" => {}
        [argument] if argument == "--help" || argument == "-h" => {
            println!(
                "Extract pinned ICU4X data for tools.codegen; reads supplemental name requests as JSON on stdin.\nUsage: iso-types-icu-data --stdout"
            );
            return Ok(());
        }
        _ => return Err("run python3 -m tools.codegen to extract and verify data".into()),
    };
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../");
    let source = root.join("data/iso");
    let english = data_locale!("en");
    let requests: Value = serde_json::from_reader(std::io::stdin().lock())?;
    let mut region_names = BTreeMap::new();
    for code in requests["regions"]
        .as_array()
        .ok_or("missing region name requests")?
    {
        let code = code.as_str().ok_or("invalid region code")?;
        region_names.insert(
            code,
            name_or_missing(RegionDisplayName::try_new_light(
                locale!("en").into(),
                code.parse()?,
            ))?,
        );
    }
    let zone_formatter = icu_datetime::NoCalendarFormatter::try_new(
        locale!("en").into(),
        icu_datetime::fieldsets::zone::Location,
    )?;
    let mut zone_names = BTreeMap::new();
    for code in requests["time_zones"]
        .as_array()
        .ok_or("missing time zone name requests")?
    {
        let code = code.as_str().ok_or("invalid time zone code")?;
        let zone = icu_time::TimeZone::from_iana_id(code);
        if zone == icu_time::TimeZone::UNKNOWN {
            return Err(format!("ICU time zone mapping is unavailable: {code}").into());
        }
        let formatted = zone_formatter
            .format(&zone.without_offset())
            .write_to_string()
            .into_owned();
        zone_names.insert(code, json!({"name": formatted, "bcp47": zone.as_str()}));
    }
    let canonicalizer = LocaleCanonicalizer::new_extended();
    let currency_plurals = PluralRules::try_new_cardinal(locale!("en").into())?;
    let fractions = DataProvider::<CurrencyFractionsV1>::load(
        &icu_experimental::provider::Baked,
        DataRequest::default(),
    )?
    .payload;
    let mut currency_fractions = BTreeMap::new();
    let mut catalogs = BTreeMap::<&str, BTreeMap<String, Option<String>>>::new();
    for (standard, file, field) in [
        ("639-1", "639-2", "alpha_2"),
        ("639-2", "639-2", "alpha_3"),
        ("639-3", "639-3", "alpha_3"),
        ("3166-1", "3166-1", "alpha_2"),
        ("15924", "15924", "alpha_4"),
        ("4217", "4217", "alpha_3"),
    ] {
        let records: Value =
            serde_json::from_slice(&std::fs::read(source.join(format!("iso_{file}.json")))?)?;
        let mut names = BTreeMap::new();
        for record in records[file].as_array().ok_or("invalid ISO source array")? {
            let Some(code) = record[field].as_str() else {
                continue;
            };
            if standard == "4217" {
                let currency =
                    icu_experimental::dimension::currency::CurrencyType::try_from_str(code)?;
                let info = fractions.get().resolve(currency);
                currency_fractions.insert(code.to_owned(), json!({
                    "digits": info.digits,
                    "cash_digits": info.cash_digits.unwrap_or(info.digits),
                    "rounding": rounding_increment(info.rounding)?,
                    "cash_rounding": rounding_increment(info.cash_rounding.unwrap_or(info.rounding))?,
                }));
            }
            let name = match standard {
                "639-1" | "639-2" | "639-3" => match code.parse::<Locale>() {
                    Ok(mut target) => {
                        canonicalizer.canonicalize(&mut target);
                        LanguageIdentifierDisplayName::try_new_heavy(
                            locale!("en").into(),
                            target.id,
                            LanguageIdentifierDisplayNameOptions::default(),
                        )?
                        .as_borrowed()
                        .try_write_to_string()
                        .ok()
                        .map(|s| s.into_owned())
                    }
                    Err(_) if code == "qaa-qtz" => None,
                    Err(e) => return Err(e.into()),
                },
                "3166-1" => name_or_missing(RegionDisplayName::try_new_light(
                    locale!("en").into(),
                    code.parse()?,
                ))?,
                "15924" => name_or_missing(ScriptDisplayName::try_new_heavy(
                    locale!("en").into(),
                    code.parse()?,
                ))?,
                "4217" => DataProvider::<CurrencyExtendedDataV1>::load(
                    &icu_experimental::provider::Baked,
                    request(code, &english),
                )
                .allow_identifier_not_found()?
                .map(|r| {
                    r.payload
                        .get()
                        .get(1_u64.into(), &currency_plurals)
                        .to_owned()
                }),
                _ => unreachable!(),
            };
            names.insert(code.to_owned(), name);
        }
        catalogs.insert(standard, names);
    }
    let months = DataProvider::<DatetimeNamesMonthGregorianV1>::load(
        &icu_datetime::provider::Baked,
        request("5s", &english),
    )?
    .payload;
    let MonthNames::Linear(months) = months.get() else {
        return Err("Gregorian months must be linear".into());
    };
    catalogs.insert(
        "8601-month",
        (1..=12)
            .map(|month| {
                Ok((
                    format!("{month:02}"),
                    Some(
                        months
                            .get(month - 1)
                            .ok_or("missing month name")?
                            .to_owned(),
                    ),
                ))
            })
            .collect::<Result<_, Box<dyn Error>>>()?,
    );
    let weekdays = DataProvider::<DatetimeNamesWeekdayV1>::load(
        &icu_datetime::provider::Baked,
        request("5s", &english),
    )?
    .payload;
    catalogs.insert(
        "8601-weekday",
        (1..=7)
            .map(|day| {
                Ok((
                    day.to_string(),
                    Some(
                        weekdays
                            .get()
                            .names
                            .get(day % 7)
                            .ok_or("missing weekday name")?
                            .to_owned(),
                    ),
                ))
            })
            .collect::<Result<_, Box<dyn Error>>>()?,
    );
    let result = json!({
        "source": "https://github.com/unicode-org/icu4x",
        "display_locale": "en",
        "versions": { "icu_locale": "2.3.1", "icu_datetime": "2.3.0", "icu_experimental": "0.6.0", "icu_time": "2.3.0" },
        "catalogs": catalogs,
        "currency_fractions": currency_fractions,
        "regions": region_names,
        "time_zones": zone_names,
    });
    let output = serde_json::to_string_pretty(&result)? + "\n";
    print!("{output}");
    Ok(())
}

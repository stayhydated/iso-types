//! Extract English names from pinned ICU4X compiled providers, without name substitution.
use icu_datetime::provider::names::{
    DatetimeNamesMonthGregorianV1, DatetimeNamesWeekdayV1, MonthNames,
};
use icu_experimental::dimension::provider::currency::extended::CurrencyExtendedDataV1;
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
use writeable::TryWriteable;

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

enum Output {
    Snapshot,
    Check,
    Stdout,
}

fn main() -> Result<(), Box<dyn Error>> {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    let mode = match arguments.as_slice() {
        [] => Output::Snapshot,
        [argument] if argument == "--check" => Output::Check,
        [argument] if argument == "--stdout" => Output::Stdout,
        [argument] if argument == "--help" || argument == "-h" => {
            println!(
                "Extract pinned ICU4X English names\nUsage: iso-types-icu-data [--check | --stdout]"
            );
            return Ok(());
        }
        _ => return Err("expected --check, --stdout, or no arguments; use --help".into()),
    };
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../");
    let source = root.join("data/iso");
    let english = data_locale!("en");
    let canonicalizer = LocaleCanonicalizer::new_extended();
    let currency_plurals = PluralRules::try_new_cardinal(locale!("en").into())?;
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
        "versions": { "icu_locale": "2.3.1", "icu_datetime": "2.3.0", "icu_experimental": "0.6.0" },
        "catalogs": catalogs,
    });
    let output = serde_json::to_string_pretty(&result)? + "\n";
    let path = root.join("data/icu/english.json");
    match mode {
        Output::Stdout => print!("{output}"),
        Output::Check => {
            if std::fs::read_to_string(&path)? != output {
                return Err("ICU English snapshot differs; run python3 -m tools.codegen".into());
            }
        }
        Output::Snapshot => std::fs::write(path, output)?,
    }
    Ok(())
}

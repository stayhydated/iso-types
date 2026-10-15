//! ISO 10383 identifiers and locally supplied MIC registry snapshots.

use std::{collections::BTreeMap, io::Read, str::FromStr};

use crate::{Iso3166_1, ParseIsoCodeError};

/// Four uppercase ASCII letters/digits identifying a market or reporting facility.
/// Registry membership is resolved through `MarketRegistry`.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Iso10383([u8; 4]);

impl Iso10383 {
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.0).expect("MIC is validated ASCII")
    }
}

impl FromStr for Iso10383 {
    type Err = ParseIsoCodeError;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let bytes: [u8; 4] = input.as_bytes().try_into().map_err(|_| mic_error(input))?;
        if !bytes
            .iter()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
        {
            return Err(mic_error(input));
        }
        Ok(Self(bytes))
    }
}

fn mic_error(input: &str) -> ParseIsoCodeError {
    ParseIsoCodeError {
        standard: "10383",
        input: input.to_owned(),
    }
}

impl TryFrom<&str> for Iso10383 {
    type Error = ParseIsoCodeError;
    fn try_from(input: &str) -> Result<Self, Self::Error> {
        input.parse()
    }
}
impl AsRef<str> for Iso10383 {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for Iso10383 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Gregorian date used by a MIC registry and its effective-date selection.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MicDate {
    year: u16,
    month: u8,
    day: u8,
}

impl FromStr for MicDate {
    type Err = ParseIsoCodeError;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let error = || ParseIsoCodeError {
            standard: "mic-date",
            input: input.to_owned(),
        };
        let digits = match *input.as_bytes() {
            [a, b, c, d, b'-', e, f, b'-', g, h] => [a, b, c, d, e, f, g, h],
            [a, b, c, d, e, f, g, h] => [a, b, c, d, e, f, g, h],
            _ => return Err(error()),
        };
        if !digits.iter().all(u8::is_ascii_digit) {
            return Err(error());
        }
        let value = |bytes: &[u8]| {
            bytes
                .iter()
                .fold(0_u16, |n, b| n * 10 + u16::from(b - b'0'))
        };
        let year = value(&digits[..4]);
        let month = value(&digits[4..6]) as u8;
        let day = value(&digits[6..]) as u8;
        let leap =
            year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
        let days = match month {
            2 => {
                if leap {
                    29
                } else {
                    28
                }
            }
            4 | 6 | 9 | 11 => 30,
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            _ => return Err(error()),
        };
        if year == 0 || day == 0 || day > days {
            return Err(error());
        }
        Ok(Self { year, month, day })
    }
}

impl std::fmt::Display for MicDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MicKind {
    Operating,
    Segment,
}

/// Status reported by the publisher, including changes with future effective dates.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MicStatus {
    Active,
    Expired,
    Updated,
}

/// A locally imported record; source columns remain available without normalization.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MarketRecord {
    pub code: Iso10383,
    pub operating_mic: Iso10383,
    pub country: Iso3166_1,
    pub kind: MicKind,
    pub reported_status: MicStatus,
    pub creation_date: MicDate,
    pub last_update_date: Option<MicDate>,
    pub last_validation_date: Option<MicDate>,
    pub expiry_date: Option<MicDate>,
    fields: BTreeMap<String, String>,
}

impl MarketRecord {
    pub fn name(&self) -> &str {
        &self.fields["MARKET NAME-INSTITUTION DESCRIPTION"]
    }
    pub fn field(&self, column: &str) -> Option<&str> {
        self.fields.get(column).map(String::as_str)
    }
    pub fn is_active_on(&self, date: MicDate) -> bool {
        self.creation_date <= date && self.expiry_date.is_none_or(|expiry| date < expiry)
    }
}

/// A date-selected, closed operating/segment graph from an application-supplied CSV.
#[derive(Debug)]
pub struct MarketRegistry {
    as_of: MicDate,
    records: BTreeMap<Iso10383, MarketRecord>,
}

impl MarketRegistry {
    /// Import the official UTF-8 CSV layout, preserving quoted fields and line breaks.
    /// Identity membership uses creation/expiry dates; descriptions and reported
    /// status retain the supplied publication's values. Countries follow the facade's scope.
    pub fn from_csv(reader: impl Read, as_of: MicDate) -> Result<Self, MarketRegistryError> {
        const REQUIRED: &[&str] = &[
            "MIC",
            "OPERATING MIC",
            "OPRT/SGMT",
            "MARKET NAME-INSTITUTION DESCRIPTION",
            "ISO COUNTRY CODE (ISO 3166)",
            "STATUS",
            "CREATION DATE",
            "LAST UPDATE DATE",
            "LAST VALIDATION DATE",
            "EXPIRY DATE",
        ];
        let mut csv = csv::Reader::from_reader(reader);
        let headers = csv.headers().map_err(MarketRegistryError::Csv)?.clone();
        let mut columns = BTreeMap::new();
        for (index, header) in headers.iter().enumerate() {
            if columns.insert(header, index).is_some() {
                return Err(MarketRegistryError::DuplicateColumn(header.to_owned()));
            }
        }
        for &column in REQUIRED {
            if !columns.contains_key(column) {
                return Err(MarketRegistryError::MissingColumn(column));
            }
        }
        let mut records = BTreeMap::new();
        let mut seen = std::collections::BTreeSet::new();
        for (index, row) in csv.records().enumerate() {
            let row = row.map_err(MarketRegistryError::Csv)?;
            let get = |column| &row[columns[column]];
            let invalid = |field: &'static str| MarketRegistryError::InvalidRecord {
                row: index + 2,
                field,
                value: get(field).to_owned(),
            };
            let code = get("MIC").parse().map_err(|_| invalid("MIC"))?;
            if !seen.insert(code) {
                return Err(MarketRegistryError::DuplicateCode(code));
            }
            let operating_mic = get("OPERATING MIC")
                .parse()
                .map_err(|_| invalid("OPERATING MIC"))?;
            let kind = match get("OPRT/SGMT") {
                "OPRT" => MicKind::Operating,
                "SGMT" => MicKind::Segment,
                _ => return Err(invalid("OPRT/SGMT")),
            };
            let reported_status = match get("STATUS") {
                "ACTIVE" => MicStatus::Active,
                "EXPIRED" => MicStatus::Expired,
                "UPDATED" => MicStatus::Updated,
                _ => return Err(invalid("STATUS")),
            };
            let creation_date = get("CREATION DATE")
                .parse()
                .map_err(|_| invalid("CREATION DATE"))?;
            let date = |field| {
                if get(field).is_empty() {
                    Ok(None)
                } else {
                    get(field).parse().map(Some).map_err(|_| invalid(field))
                }
            };
            let expiry_date = date("EXPIRY DATE")?;
            let last_update_date = date("LAST UPDATE DATE")?;
            let last_validation_date = date("LAST VALIDATION DATE")?;
            if get("MARKET NAME-INSTITUTION DESCRIPTION").is_empty() {
                return Err(invalid("MARKET NAME-INSTITUTION DESCRIPTION"));
            }
            if reported_status == MicStatus::Expired && expiry_date.is_none() {
                return Err(invalid("EXPIRY DATE"));
            }
            let country_code = get("ISO COUNTRY CODE (ISO 3166)");
            if country_code.len() != 2 || !country_code.bytes().all(|b| b.is_ascii_uppercase()) {
                return Err(invalid("ISO COUNTRY CODE (ISO 3166)"));
            }
            let Ok(country) = country_code.parse() else {
                continue;
            };
            let fields = headers
                .iter()
                .zip(row.iter())
                .map(|(k, v)| (k.to_owned(), v.to_owned()))
                .collect();
            let record = MarketRecord {
                code,
                operating_mic,
                country,
                kind,
                reported_status,
                creation_date,
                last_update_date,
                last_validation_date,
                expiry_date,
                fields,
            };
            if record.is_active_on(as_of) {
                records.insert(code, record);
            }
        }
        for record in records.values() {
            let operating = records.get(&record.operating_mic);
            if operating.is_none_or(|parent| {
                parent.kind != MicKind::Operating || parent.operating_mic != parent.code
            }) || (record.kind == MicKind::Operating && record.operating_mic != record.code)
            {
                return Err(MarketRegistryError::InvalidOperatingMic {
                    code: record.code,
                    operating_mic: record.operating_mic,
                });
            }
        }
        Ok(Self { as_of, records })
    }

    pub const fn as_of(&self) -> MicDate {
        self.as_of
    }
    pub fn len(&self) -> usize {
        self.records.len()
    }
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
    pub fn get(&self, code: Iso10383) -> Option<&MarketRecord> {
        self.records.get(&code)
    }
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &MarketRecord> + ExactSizeIterator {
        self.records.values()
    }
    pub fn for_country(&self, country: Iso3166_1) -> impl Iterator<Item = &MarketRecord> {
        self.iter().filter(move |r| r.country == country)
    }
    pub fn segments(&self, operating_mic: Iso10383) -> impl Iterator<Item = &MarketRecord> {
        self.iter()
            .filter(move |r| r.kind == MicKind::Segment && r.operating_mic == operating_mic)
    }
}

#[derive(Debug)]
pub enum MarketRegistryError {
    Csv(csv::Error),
    MissingColumn(&'static str),
    DuplicateColumn(String),
    InvalidRecord {
        row: usize,
        field: &'static str,
        value: String,
    },
    DuplicateCode(Iso10383),
    InvalidOperatingMic {
        code: Iso10383,
        operating_mic: Iso10383,
    },
}

impl std::fmt::Display for MarketRegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Csv(error) => error.fmt(f),
            Self::MissingColumn(column) => write!(f, "MIC registry requires column {column}"),
            Self::DuplicateColumn(column) => write!(f, "duplicate MIC registry column {column}"),
            Self::InvalidRecord { row, field, value } => write!(
                f,
                "invalid MIC registry field {field} at row {row}: {value:?}"
            ),
            Self::DuplicateCode(code) => write!(f, "duplicate MIC registry code {code}"),
            Self::InvalidOperatingMic {
                code,
                operating_mic,
            } => write!(
                f,
                "MIC {code} references unavailable or invalid operating MIC {operating_mic}"
            ),
        }
    }
}

impl std::error::Error for MarketRegistryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Csv(error) => Some(error),
            _ => None,
        }
    }
}

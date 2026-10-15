#![doc = include_str!("../README.md")]

mod data;

use iso_types::{Iso3166_1, ParseIsoCodeError};

/// A selected UN/LOCODE identity. Parse its five-character code for persistence.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UnLocode(u32);

impl UnLocode {
    pub const RELEASE: &str = "2025-1";
    pub const COUNT: usize = data::COUNT;
    pub const RECORD_COUNT: usize = data::RECORD_COUNT;

    /// All identities, in canonical code order.
    pub fn iter() -> impl DoubleEndedIterator<Item = Self> + ExactSizeIterator {
        (0..Self::COUNT as u32).map(Self)
    }

    /// Location identities belonging to a selected ISO country, in code order.
    pub fn for_country(
        country: Iso3166_1,
    ) -> impl DoubleEndedIterator<Item = Self> + ExactSizeIterator {
        let range = data::COUNTRIES
            .binary_search_by_key(&country.code(), |(code, _, _)| *code)
            .ok()
            .map(|index| {
                let (_, start, end) = data::COUNTRIES[index];
                start..end
            })
            .unwrap_or(0..0);
        range.map(Self)
    }

    fn block(self) -> &'static str {
        let index = self.0 as usize;
        &data::DIRECTORY[data::OFFSETS[index] as usize..data::OFFSETS[index + 1] as usize]
    }

    pub fn code(self) -> &'static str {
        &self.block()[..5]
    }

    pub fn country(self) -> Iso3166_1 {
        self.code()[..2]
            .parse()
            .expect("generated location country belongs to the ISO catalog")
    }

    /// Every published listing for this identity, including alternative names
    /// and any differences in subdivision, function, status, or coordinates.
    pub fn records(self) -> impl Iterator<Item = LocationRecord> {
        self.block().lines().map(move |line| {
            let mut fields = line.split('\t');
            let mut next = || fields.next().expect("generated listing has all columns");
            let code = next();
            debug_assert_eq!(code, self.code());
            LocationRecord {
                location: self,
                name: next(),
                name_without_diacritics: next(),
                subdivision_code: optional(next()),
                function_code: next(),
                status: optional(next())
                    .map(|value| value.parse().expect("validated location status")),
                source_date: optional(next()),
                iata_code: optional(next()),
                coordinate_text: optional(next()),
                remarks: optional(next()),
                change_indicator: optional(next()),
            }
        })
    }
}

impl std::str::FromStr for UnLocode {
    type Err = ParseIsoCodeError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut low = 0;
        let mut high = Self::COUNT;
        while low < high {
            let mid = low + (high - low) / 2;
            match Self(mid as u32).code().cmp(input) {
                std::cmp::Ordering::Less => low = mid + 1,
                std::cmp::Ordering::Greater => high = mid,
                std::cmp::Ordering::Equal => return Ok(Self(mid as u32)),
            }
        }
        Err(ParseIsoCodeError {
            standard: "unlocode",
            input: input.to_owned(),
        })
    }
}

impl TryFrom<&str> for UnLocode {
    type Error = ParseIsoCodeError;
    fn try_from(input: &str) -> Result<Self, Self::Error> {
        input.parse()
    }
}

impl std::fmt::Display for UnLocode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.code())
    }
}

/// One source listing. Names retain the publisher's Roman-alphabet spelling.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LocationRecord {
    pub location: UnLocode,
    pub name: &'static str,
    pub name_without_diacritics: &'static str,
    pub subdivision_code: Option<&'static str>,
    pub function_code: &'static str,
    pub status: Option<LocationStatus>,
    /// Source year/month in the original four-character spelling.
    pub source_date: Option<&'static str>,
    pub iata_code: Option<&'static str>,
    /// Original coordinate text, including incomplete upstream values.
    pub coordinate_text: Option<&'static str>,
    pub remarks: Option<&'static str>,
    pub change_indicator: Option<&'static str>,
}

impl LocationRecord {
    /// Decode recognized markers in the eight defined byte positions.
    /// Missing or unrecognized markers and trailing bytes are ignored.
    pub fn functions(self) -> impl Iterator<Item = LocationFunction> {
        let code = self.function_code.as_bytes();
        LocationFunction::ALL
            .into_iter()
            .enumerate()
            .filter_map(move |(index, value)| {
                (code.get(index) == Some(&b"1234567B"[index])).then_some(value)
            })
    }

    /// Parse degree/minute coordinates; malformed source values return `None`.
    pub fn coordinates(self) -> Option<Coordinates> {
        let text = self.coordinate_text?.as_bytes();
        if text.len() != 12 || text[5] != b' ' {
            return None;
        }
        let parse = |bytes: &[u8]| {
            bytes.iter().try_fold(0_i16, |n, &b| {
                b.is_ascii_digit().then(|| n * 10 + i16::from(b - b'0'))
            })
        };
        let latitude = parse(&text[..2])?;
        let latitude_minutes = parse(&text[2..4])?;
        let longitude = parse(&text[6..9])?;
        let longitude_minutes = parse(&text[9..11])?;
        if latitude_minutes >= 60
            || longitude_minutes >= 60
            || latitude > 90
            || longitude > 180
            || (latitude == 90 && latitude_minutes != 0)
            || (longitude == 180 && longitude_minutes != 0)
        {
            return None;
        }
        let sign = |b, positive, negative| {
            if b == positive {
                Some(1)
            } else if b == negative {
                Some(-1)
            } else {
                None
            }
        };
        Some(Coordinates {
            latitude_minutes: (latitude * 60 + latitude_minutes) * sign(text[4], b'N', b'S')?,
            longitude_minutes: (longitude * 60 + longitude_minutes) * sign(text[11], b'E', b'W')?,
        })
    }

    #[cfg(feature = "subdivisions")]
    pub fn subdivision(self) -> Option<iso_types::Iso3166_2> {
        self.location.country().subdivision(self.subdivision_code?)
    }
}

/// Coordinates at the publisher's whole-minute precision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Coordinates {
    latitude_minutes: i16,
    longitude_minutes: i16,
}

impl Coordinates {
    pub const fn latitude_minutes(self) -> i16 {
        self.latitude_minutes
    }
    pub const fn longitude_minutes(self) -> i16 {
        self.longitude_minutes
    }
    pub fn latitude_degrees(self) -> f64 {
        f64::from(self.latitude_minutes) / 60.0
    }
    pub fn longitude_degrees(self) -> f64 {
        f64::from(self.longitude_minutes) / 60.0
    }
}

/// The facility functions defined by Recommendation 16.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum LocationFunction {
    Port,
    RailTerminal,
    RoadTerminal,
    Airport,
    PostalExchange,
    Multimodal,
    FixedTransport,
    BorderCrossing,
}

impl LocationFunction {
    pub const ALL: [Self; 8] = [
        Self::Port,
        Self::RailTerminal,
        Self::RoadTerminal,
        Self::Airport,
        Self::PostalExchange,
        Self::Multimodal,
        Self::FixedTransport,
        Self::BorderCrossing,
    ];
}

/// The source's two-character approval/status code.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum LocationStatus {
    Aa,
    Ac,
    Af,
    Ai,
    Am,
    Aq,
    As,
    Qq,
    Rl,
    Rn,
    Rq,
    Ur,
    Xx,
}

impl LocationStatus {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Aa => "AA",
            Self::Ac => "AC",
            Self::Af => "AF",
            Self::Ai => "AI",
            Self::Am => "AM",
            Self::Aq => "AQ",
            Self::As => "AS",
            Self::Qq => "QQ",
            Self::Rl => "RL",
            Self::Rn => "RN",
            Self::Rq => "RQ",
            Self::Ur => "UR",
            Self::Xx => "XX",
        }
    }
}

impl std::str::FromStr for LocationStatus {
    type Err = ParseIsoCodeError;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "AA" => Ok(Self::Aa),
            "AC" => Ok(Self::Ac),
            "AF" => Ok(Self::Af),
            "AI" => Ok(Self::Ai),
            "AM" => Ok(Self::Am),
            "AQ" => Ok(Self::Aq),
            "AS" => Ok(Self::As),
            "QQ" => Ok(Self::Qq),
            "RL" => Ok(Self::Rl),
            "RN" => Ok(Self::Rn),
            "RQ" => Ok(Self::Rq),
            "UR" => Ok(Self::Ur),
            "XX" => Ok(Self::Xx),
            _ => Err(ParseIsoCodeError {
                standard: "unlocode-status",
                input: input.to_owned(),
            }),
        }
    }
}

fn optional(value: &'static str) -> Option<&'static str> {
    (!value.is_empty()).then_some(value)
}

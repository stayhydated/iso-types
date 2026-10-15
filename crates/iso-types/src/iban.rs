//! Registered national IBAN formats and structural/checksum validation.

mod data;

use crate::Iso3166_1;

/// The permitted ASCII character class of a fixed-width BBAN component.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BbanCharacterKind {
    Numeric,
    Alphabetic,
    Alphanumeric,
}

/// One ordered fixed-width component of the registered BBAN structure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BbanComponent {
    pub length: u8,
    pub kind: BbanCharacterKind,
}

/// National bank-account structure from the ISO 13616 registration authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IbanFormat {
    pub prefix_country: Iso3166_1,
    pub iban_length: u8,
    pub bban_structure: &'static str,
    pub iban_structure: &'static str,
    pub effective_date: &'static str,
    pub electronic_example: &'static str,
    pub print_example: &'static str,
    pub countries: &'static [Iso3166_1],
    pub components: &'static [BbanComponent],
    bank_range: Option<(u8, u8)>,
    branch_range: Option<(u8, u8)>,
}

impl IbanFormat {
    pub const ALL: &'static [Self] = data::FORMATS;
    pub const REGISTRY_RELEASE: &str = "103";

    /// Zero-based, half-open positions within the BBAN.
    pub const fn bank_identifier_range(self) -> Option<std::ops::Range<usize>> {
        match self.bank_range {
            Some((start, end)) => Some(start as usize..end as usize),
            None => None,
        }
    }

    /// Zero-based, half-open positions within the BBAN.
    pub const fn branch_identifier_range(self) -> Option<std::ops::Range<usize>> {
        match self.branch_range {
            Some((start, end)) => Some(start as usize..end as usize),
            None => None,
        }
    }
}

/// A canonical electronic IBAN with a registered structure and valid MOD 97 checksum.
/// Validation covers syntax, country format, and the international check digits.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Iban(String);

impl Iban {
    pub fn as_str(&self) -> &str {
        &self.0
    }
    pub fn bban(&self) -> &str {
        &self.0[4..]
    }

    pub fn country(&self) -> Iso3166_1 {
        self.0[..2]
            .parse()
            .expect("validated IBAN has a selected ISO prefix")
    }

    pub fn format(&self) -> &'static IbanFormat {
        self.country()
            .iban_format()
            .expect("validated IBAN has a registered format")
    }

    pub fn bank_identifier(&self) -> Option<&str> {
        Some(&self.bban()[self.format().bank_identifier_range()?])
    }

    pub fn branch_identifier(&self) -> Option<&str> {
        Some(&self.bban()[self.format().branch_identifier_range()?])
    }

    /// Conventional print representation, grouped in four-character blocks.
    pub fn print_format(&self) -> String {
        let mut output = String::with_capacity(self.0.len() + self.0.len() / 4);
        for (index, byte) in self.0.bytes().enumerate() {
            if index != 0 && index % 4 == 0 {
                output.push(' ');
            }
            output.push(char::from(byte));
        }
        output
    }
}

/// Invalid input is distinguished from an unregistered country format or checksum.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParseIbanError {
    InvalidCharacters,
    UnsupportedCountry,
    InvalidLength { expected: usize, actual: usize },
    InvalidStructure,
    InvalidChecksum,
}

impl std::fmt::Display for ParseIbanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidCharacters => {
                f.write_str("IBAN requires ASCII letters, digits, and optional spaces")
            }
            Self::UnsupportedCountry => {
                f.write_str("IBAN prefix has no format in the selected registry")
            }
            Self::InvalidLength { expected, actual } => {
                write!(f, "IBAN length is {actual}; expected {expected}")
            }
            Self::InvalidStructure => {
                f.write_str("IBAN does not match the registered national structure")
            }
            Self::InvalidChecksum => f.write_str("IBAN check digits are invalid"),
        }
    }
}

impl std::error::Error for ParseIbanError {}

impl std::str::FromStr for Iban {
    type Err = ParseIbanError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let length = input.bytes().filter(|&byte| byte != b' ').count();
        if length > 34 {
            return Err(ParseIbanError::InvalidLength {
                expected: 34,
                actual: length,
            });
        }
        let mut normalized = String::with_capacity(34);
        for byte in input.bytes() {
            if byte == b' ' {
                continue;
            }
            if !byte.is_ascii_alphanumeric() {
                return Err(ParseIbanError::InvalidCharacters);
            }
            normalized.push(char::from(byte.to_ascii_uppercase()));
        }
        let bytes = normalized.as_bytes();
        if bytes.len() < 4 {
            return Err(ParseIbanError::InvalidStructure);
        }
        let country: Iso3166_1 = normalized[..2]
            .parse()
            .map_err(|_| ParseIbanError::UnsupportedCountry)?;
        let format = country
            .iban_format()
            .ok_or(ParseIbanError::UnsupportedCountry)?;
        if format.prefix_country != country {
            return Err(ParseIbanError::UnsupportedCountry);
        }
        if bytes.len() != usize::from(format.iban_length) {
            return Err(ParseIbanError::InvalidLength {
                expected: usize::from(format.iban_length),
                actual: bytes.len(),
            });
        }
        if !bytes[2..4].iter().all(u8::is_ascii_digit) {
            return Err(ParseIbanError::InvalidStructure);
        }
        let mut position = 4;
        for component in format.components {
            let end = position + usize::from(component.length);
            let matches = bytes[position..end].iter().all(|b| match component.kind {
                BbanCharacterKind::Numeric => b.is_ascii_digit(),
                BbanCharacterKind::Alphabetic => b.is_ascii_uppercase(),
                BbanCharacterKind::Alphanumeric => b.is_ascii_alphanumeric(),
            });
            if !matches {
                return Err(ParseIbanError::InvalidStructure);
            }
            position = end;
        }
        let check = (bytes[2] - b'0') * 10 + bytes[3] - b'0';
        let remainder = bytes[4..]
            .iter()
            .chain(&bytes[..4])
            .fold(0_u32, |remainder, byte| {
                if byte.is_ascii_digit() {
                    (remainder * 10 + u32::from(byte - b'0')) % 97
                } else {
                    (remainder * 100 + u32::from(byte - b'A' + 10)) % 97
                }
            });
        if !(2..=98).contains(&check) || remainder != 1 {
            return Err(ParseIbanError::InvalidChecksum);
        }
        Ok(Self(normalized))
    }
}

impl TryFrom<&str> for Iban {
    type Error = ParseIbanError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl AsRef<str> for Iban {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for Iban {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

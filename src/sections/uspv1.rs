use crate::sections::SectionDecodeError;
use std::str::{Chars, FromStr};

const USP_V1_VERSION: u8 = 1;
const KIND: &str = "uspv1";

#[derive(Debug, Eq, PartialEq)]
pub enum Consent {
    Yes,
    No,
    NotApplicable,
}

impl Consent {
    fn from_char(c: char) -> Option<Self> {
        match c {
            'Y' => Some(Self::Yes),
            'N' => Some(Self::No),
            '-' => Some(Self::NotApplicable),
            _ => None,
        }
    }
}

// See https://github.com/InteractiveAdvertisingBureau/USPrivacy/blob/master/CCPA/US%20Privacy%20String.md#us-privacy-string-format
#[derive(Debug, Eq, PartialEq)]
pub struct UspV1 {
    version: u8,
    notice: Consent,
    optout_sale: Consent,
    lspa_covered: Consent,
}

impl FromStr for UspV1 {
    type Err = SectionDecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();

        let version = chars
            .next()
            .ok_or(SectionDecodeError::UnexpectedEndOfString(s.to_string()))?;
        let version = version
            .to_digit(10)
            .ok_or(SectionDecodeError::InvalidCharacter {
                character: version,
                kind: KIND,
                s: s.to_string(),
            })? as u8;
        if version != USP_V1_VERSION {
            return Err(SectionDecodeError::InvalidSectionVersion {
                expected: USP_V1_VERSION,
                found: version,
            });
        }

        let notice = parse_next_consent_char(&mut chars, s)?;
        let optout_sale = parse_next_consent_char(&mut chars, s)?;
        let lspa_covered = parse_next_consent_char(&mut chars, s)?;

        Ok(Self {
            version,
            notice,
            optout_sale,
            lspa_covered,
        })
    }
}

fn parse_next_consent_char(
    chars: &mut Chars,
    original_str: &str,
) -> Result<Consent, SectionDecodeError> {
    let consent = chars
        .next()
        .ok_or(SectionDecodeError::UnexpectedEndOfString(
            original_str.to_string(),
        ))?;

    Consent::from_char(consent).ok_or(SectionDecodeError::InvalidCharacter {
        character: consent,
        kind: KIND,
        s: original_str.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("1YN-" => UspV1 {
        version: 1,
        notice: Consent::Yes,
        optout_sale: Consent::No,
        lspa_covered: Consent::NotApplicable,
    } ; "mix")]
    #[test_case("1NNN" => UspV1 {
        version: 1,
        notice: Consent::No,
        optout_sale: Consent::No,
        lspa_covered: Consent::No,
    } ; "all no")]
    #[test_case("1YYY" => UspV1 {
        version: 1,
        notice: Consent::Yes,
        optout_sale: Consent::Yes,
        lspa_covered: Consent::Yes,
    } ; "all yes")]
    fn parse(s: &str) -> UspV1 {
        UspV1::from_str(s).unwrap()
    }

    #[test_case("ZYN-" => matches SectionDecodeError::InvalidCharacter { character: 'Z', .. } ; "invalid version character")]
    #[test_case("2YN-" => matches SectionDecodeError::InvalidSectionVersion {
        expected: USP_V1_VERSION,
        found: 2
    } ; "invalid version number")]
    #[test_case("" => matches SectionDecodeError::UnexpectedEndOfString(_) ; "empty string")]
    #[test_case("1" => matches SectionDecodeError::UnexpectedEndOfString(_) ; "header only")]
    #[test_case("1N" => matches SectionDecodeError::UnexpectedEndOfString(_) ; "missing characters")]
    #[test_case("1A" => matches SectionDecodeError::InvalidCharacter { character: 'A', .. } ; "invalid consent character")]
    fn error(s: &str) -> SectionDecodeError {
        UspV1::from_str(s).unwrap_err()
    }
}

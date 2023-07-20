use crate::sections::SectionDecodeError;
use std::str::{Chars, FromStr};

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

#[derive(Debug, Eq, PartialEq)]
pub struct UspV1 {
    version: u32,
    notice: Consent,
    optout_sale: Consent,
    lspa_covered: Consent,
}

const KIND: &str = "uspv1";

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
            })?;

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

    #[test]
    fn test_ok() {
        let test_cases = [
            (
                "1YN-",
                UspV1 {
                    version: 1,
                    notice: Consent::Yes,
                    optout_sale: Consent::No,
                    lspa_covered: Consent::NotApplicable,
                },
            ),
            (
                "1NNN",
                UspV1 {
                    version: 1,
                    notice: Consent::No,
                    optout_sale: Consent::No,
                    lspa_covered: Consent::No,
                },
            ),
        ];

        for (s, expected) in test_cases {
            assert_eq!(UspV1::from_str(s).unwrap(), expected);
        }
    }

    #[test]
    fn test_error() {
        assert!(matches!(
            UspV1::from_str("ZYN-").unwrap_err(),
            SectionDecodeError::InvalidCharacter { character: 'Z', .. }
        ));

        assert!(matches!(
            UspV1::from_str("").unwrap_err(),
            SectionDecodeError::UnexpectedEndOfString(_)
        ));

        assert!(matches!(
            UspV1::from_str("1").unwrap_err(),
            SectionDecodeError::UnexpectedEndOfString(_)
        ));

        assert!(matches!(
            UspV1::from_str("1N").unwrap_err(),
            SectionDecodeError::UnexpectedEndOfString(_)
        ));

        assert!(matches!(
            UspV1::from_str("1A").unwrap_err(),
            SectionDecodeError::InvalidCharacter { character: 'A', .. }
        ));
    }
}

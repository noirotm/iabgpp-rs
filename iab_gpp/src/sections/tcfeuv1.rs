use crate::core::DataReader;
use crate::sections::{IdSet, SectionDecodeError};
use iab_gpp_derive::GPPSection;
use std::collections::BTreeSet;

// See https://github.com/InteractiveAdvertisingBureau/GDPR-Transparency-and-Consent-Framework/blob/master/Consent%20string%20and%20vendor%20list%20formats%20v1.1%20Final.md
#[derive(Debug, Eq, PartialEq, GPPSection)]
#[gpp(section_version = 1)]
pub struct TcfEuV1 {
    #[gpp(datetime_as_unix_timestamp)]
    pub created: i64,
    #[gpp(datetime_as_unix_timestamp)]
    pub last_updated: i64,
    pub cmp_id: u16,
    pub cmp_version: u16,
    pub consent_screen: u8,
    #[gpp(string(2))]
    pub consent_language: String,
    pub vendor_list_version: u16,
    #[gpp(fixed_bitfield(24))]
    pub purposes_allowed: IdSet,
    #[gpp(parse_with = parse_vendor_consents)]
    pub vendor_consents: IdSet,
}

fn parse_vendor_consents(r: &mut DataReader) -> Result<IdSet, SectionDecodeError> {
    let max_vendor_id = r.read_fixed_integer(16)?;
    let is_range = r.read_bool()?;
    Ok(if is_range {
        // range section
        let default_consent = r.read_bool()?;
        let ids = BTreeSet::from_iter(r.read_integer_range()?);

        // create final vendor list based on the default consent:
        // only return list of vendors who consent
        (1..=max_vendor_id)
            .filter(|id| {
                //(default_consent && !ids.contains(id)) || (!default_consent && ids.contains(id))
                default_consent ^ ids.contains(id)
            })
            .collect()
    } else {
        // bitfield section
        r.read_fixed_bitfield(max_vendor_id as usize)?
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use test_case::test_case;

    #[test]
    fn success() {
        let actual = TcfEuV1::from_str("BOEFEAyOEFEAyAHABDENAI4AAAB9vABAASA").unwrap();
        let expected = TcfEuV1 {
            created: 1510082155,
            last_updated: 1510082155,
            cmp_id: 7,
            cmp_version: 1,
            consent_screen: 3,
            consent_language: "EN".to_string(),
            vendor_list_version: 8,
            purposes_allowed: [1, 2, 3].into(),
            vendor_consents: (1..=2011).filter(|&id| id != 9).collect(),
        };

        assert_eq!(actual, expected);
    }

    #[test_case("BO5a1L7O5a1L7AAABBENC2-AAAAtH" => matches SectionDecodeError::Read(_) ; "missing data")]
    #[test_case("" => matches SectionDecodeError::Read(_) ; "empty string")]
    #[test_case("DOEFEAyOEFEAyAHABDENAI4AAAB9vABAASA" => matches SectionDecodeError::UnknownSegmentVersion { segment_version: 3 } ; "unknown segment version")]
    fn error(s: &str) -> SectionDecodeError {
        TcfEuV1::from_str(s).unwrap_err()
    }
}

use crate::core::{DataReader, DecodeExt};
use crate::sections::{SectionDecodeError, VendorList};
use std::collections::BTreeSet;
use std::str::FromStr;

const TCF_EU_V1_VERSION: u8 = 1;

// See https://github.com/InteractiveAdvertisingBureau/GDPR-Transparency-and-Consent-Framework/blob/master/Consent%20string%20and%20vendor%20list%20formats%20v1.1%20Final.md
#[derive(Debug, Eq, PartialEq)]
pub struct TcfEuV1 {
    pub version: u8,
    pub created: i64,
    pub last_updated: i64,
    pub cmp_id: u16,
    pub cmp_version: u16,
    pub consent_screen: u8,
    pub consent_language: String,
    pub vendor_list_version: u16,
    pub purposes_allowed: Vec<bool>,
    pub vendor_consents: VendorList,
}

impl FromStr for TcfEuV1 {
    type Err = SectionDecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let b = s.decode_base64_url()?;
        let mut r = DataReader::new(&b);

        let version = r.read_fixed_integer::<u8>(6)?;
        if version != TCF_EU_V1_VERSION {
            return Err(SectionDecodeError::InvalidSectionVersion {
                expected: TCF_EU_V1_VERSION,
                found: version,
            });
        }

        let created = r.read_datetime_as_unix_timestamp()?;
        let last_updated = r.read_datetime_as_unix_timestamp()?;
        let cmp_id = r.read_fixed_integer(12)?;
        let cmp_version = r.read_fixed_integer(12)?;
        let consent_screen = r.read_fixed_integer(6)?;
        let consent_language = r.read_string(2)?;
        let vendor_list_version = r.read_fixed_integer(12)?;
        let purposes_allowed = r.read_fixed_bitfield(24)?;
        let vendor_consents = Self::parse_vendor_consents(&mut r)?;

        Ok(Self {
            version,
            created,
            last_updated,
            cmp_id,
            cmp_version,
            consent_screen,
            consent_language,
            vendor_list_version,
            purposes_allowed,
            vendor_consents,
        })
    }
}

impl TcfEuV1 {
    fn parse_vendor_consents(r: &mut DataReader) -> Result<VendorList, SectionDecodeError> {
        let max_vendor_id = r.read_fixed_integer::<u16>(16)?;
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
                .iter()
                .enumerate()
                .filter_map(|(i, b)| b.then_some((i + 1) as u16))
                .collect()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::iter::repeat;

    #[test]
    fn parse() {
        let actual = TcfEuV1::from_str("BOEFEAyOEFEAyAHABDENAI4AAAB9vABAASA").unwrap();
        let expected = TcfEuV1 {
            version: TCF_EU_V1_VERSION,
            created: 1510082155,
            last_updated: 1510082155,
            cmp_id: 7,
            cmp_version: 1,
            consent_screen: 3,
            consent_language: "EN".to_string(),
            vendor_list_version: 8,
            purposes_allowed: repeat(true).take(3).chain(repeat(false).take(21)).collect(),
            vendor_consents: (1..=2011).filter(|&id| id != 9).collect(),
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn missing_data() {
        let r = TcfEuV1::from_str("BO5a1L7O5a1L7AAABBENC2-AAAAtHAA");
        dbg!(&r);
        assert!(matches!(r.unwrap_err(), SectionDecodeError::Read(_)));
    }
}

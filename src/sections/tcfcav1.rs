use crate::core::{DataReader, FromDataReader};
use crate::sections::{IdSet, OptionalSegmentParser, SectionDecodeError, SegmentedStr};
use std::str::FromStr;

const TCF_CA_V1_VERSION: u8 = 1;
const TCF_CA_V1_PUBLISHER_PURPOSES_SEGMENT_TYPE: u8 = 3;

#[derive(Debug, Eq, PartialEq)]
pub struct TcfCaV1 {
    pub core: Core,
    pub publisher_purposes: Option<PublisherPurposes>,
}

impl FromStr for TcfCaV1 {
    type Err = SectionDecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse_segmented_str()
    }
}

impl FromDataReader for TcfCaV1 {
    type Err = SectionDecodeError;

    fn from_data_reader(r: &mut DataReader) -> Result<Self, Self::Err> {
        Ok(Self {
            core: r.parse()?,
            publisher_purposes: None,
        })
    }
}

impl OptionalSegmentParser for TcfCaV1 {
    fn parse_optional_segment(
        segment_type: u8,
        r: &mut DataReader,
        into: &mut Self,
    ) -> Result<(), SectionDecodeError> {
        match segment_type {
            TCF_CA_V1_PUBLISHER_PURPOSES_SEGMENT_TYPE => {
                into.publisher_purposes = Some(r.parse()?);
            }
            n => {
                return Err(SectionDecodeError::UnknownSegmentType { segment_type: n });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Core {
    pub created: i64,
    pub last_updated: i64,
    pub cmp_id: u16,
    pub cmp_version: u16,
    pub consent_screen: u8,
    pub consent_language: String,
    pub vendor_list_version: u16,
    pub policy_version: u8,
    pub use_non_standard_stacks: bool,
    pub special_feature_express_consents: IdSet,
    pub purpose_express_consents: IdSet,
    pub purpose_implied_consents: IdSet,
    pub vendor_express_consents: IdSet,
    pub vendor_implied_consents: IdSet,
}

impl FromDataReader for Core {
    type Err = SectionDecodeError;

    fn from_data_reader(r: &mut DataReader) -> Result<Self, Self::Err> {
        let version = r.read_fixed_integer::<u8>(6)?;
        if version != TCF_CA_V1_VERSION {
            return Err(SectionDecodeError::InvalidSegmentVersion {
                expected: TCF_CA_V1_VERSION,
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
        let policy_version = r.read_fixed_integer(6)?;
        let use_non_standard_stacks = r.read_bool()?;
        let special_feature_express_consents = r.read_fixed_bitfield(12)?;
        let purpose_express_consents = r.read_fixed_bitfield(24)?;
        let purpose_implied_consents = r.read_fixed_bitfield(24)?;
        let vendor_express_consents = r.read_optimized_range()?;
        let vendor_implied_consents = r.read_optimized_range()?;

        Ok(Self {
            created,
            last_updated,
            cmp_id,
            cmp_version,
            consent_screen,
            consent_language,
            vendor_list_version,
            policy_version,
            use_non_standard_stacks,
            special_feature_express_consents,
            purpose_express_consents,
            purpose_implied_consents,
            vendor_express_consents,
            vendor_implied_consents,
        })
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct PublisherPurposes {
    pub purpose_express_consents: IdSet,
    pub purpose_implied_consents: IdSet,
    pub custom_purpose_express_consents: IdSet,
    pub custom_purpose_implied_consents: IdSet,
}

impl FromDataReader for PublisherPurposes {
    type Err = SectionDecodeError;

    fn from_data_reader(r: &mut DataReader) -> Result<Self, Self::Err> {
        let purpose_express_consents = r.read_fixed_bitfield(24)?;
        let purpose_implied_consents = r.read_fixed_bitfield(24)?;
        let custom_purposes_num = r.read_fixed_integer::<u8>(6)?;
        let custom_purpose_express_consents =
            r.read_fixed_bitfield(custom_purposes_num as usize)?;
        let custom_purpose_implied_consents =
            r.read_fixed_bitfield(custom_purposes_num as usize)?;

        Ok(Self {
            purpose_express_consents,
            purpose_implied_consents,
            custom_purpose_express_consents,
            custom_purpose_implied_consents,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test]
    fn basic() {
        let actual = TcfCaV1::from_str("BPXuQIAPXuQIAAfKABENB-CgAAAAAAAAAAAAAAAA").unwrap();
        let expected = TcfCaV1 {
            core: Core {
                created: 1650412800,
                last_updated: 1650412800,
                cmp_id: 31,
                cmp_version: 640,
                consent_screen: 1,
                consent_language: "EN".to_string(),
                vendor_list_version: 126,
                policy_version: 2,
                use_non_standard_stacks: true,
                special_feature_express_consents: Default::default(),
                purpose_express_consents: Default::default(),
                purpose_implied_consents: Default::default(),
                vendor_express_consents: Default::default(),
                vendor_implied_consents: Default::default(),
            },
            publisher_purposes: None,
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn with_publisher_purposes() {
        let actual =
            TcfCaV1::from_str("BPXuQIAPXuQIAAfKABENB-CgAAAAAAAAAAAAAAAA.YAAAAAAAAAA").unwrap();
        let expected = TcfCaV1 {
            core: Core {
                created: 1650412800,
                last_updated: 1650412800,
                cmp_id: 31,
                cmp_version: 640,
                consent_screen: 1,
                consent_language: "EN".to_string(),
                vendor_list_version: 126,
                policy_version: 2,
                use_non_standard_stacks: true,
                special_feature_express_consents: Default::default(),
                purpose_express_consents: Default::default(),
                purpose_implied_consents: Default::default(),
                vendor_express_consents: Default::default(),
                vendor_implied_consents: Default::default(),
            },
            publisher_purposes: Some(PublisherPurposes {
                purpose_express_consents: Default::default(),
                purpose_implied_consents: Default::default(),
                custom_purpose_express_consents: Default::default(),
                custom_purpose_implied_consents: Default::default(),
            }),
        };

        assert_eq!(actual, expected);
    }

    #[test_case("BPX" => matches SectionDecodeError::DecodeSegment(_) ; "decode error")]
    #[test_case("" => matches SectionDecodeError::Read(_) ; "empty string")]
    fn error(s: &str) -> SectionDecodeError {
        TcfCaV1::from_str(s).unwrap_err()
    }
}

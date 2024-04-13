use crate::core::{DataReader, FromDataReader, Range};
use crate::sections::{IdSet, OptionalSegmentParser, SectionDecodeError, SegmentedStr};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::str::FromStr;

const TCF_EU_V2_VERSION: u8 = 2;
const TCF_EU_V2_DISCLOSED_VENDORS_SEGMENT_TYPE: u8 = 1;
const TCF_EU_V2_PUBLISHER_PURPOSES_SEGMENT_TYPE: u8 = 3;

#[derive(Debug, Eq, PartialEq)]
pub struct TcfEuV2 {
    pub core: Core,
    pub disclosed_vendors: Option<IdSet>,
    pub publisher_purposes: Option<PublisherPurposes>,
}

impl FromStr for TcfEuV2 {
    type Err = SectionDecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse_segmented_str()
    }
}

impl FromDataReader for TcfEuV2 {
    type Err = SectionDecodeError;

    fn from_data_reader(r: &mut DataReader) -> Result<Self, Self::Err> {
        Ok(Self {
            core: r.parse()?,
            disclosed_vendors: None,
            publisher_purposes: None,
        })
    }
}

impl OptionalSegmentParser for TcfEuV2 {
    fn parse_optional_segment(
        segment_type: u8,
        r: &mut DataReader,
        into: &mut Self,
    ) -> Result<(), SectionDecodeError> {
        match segment_type {
            TCF_EU_V2_DISCLOSED_VENDORS_SEGMENT_TYPE => {
                into.disclosed_vendors = Some(r.read_optimized_integer_range()?);
            }
            TCF_EU_V2_PUBLISHER_PURPOSES_SEGMENT_TYPE => {
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
    pub is_service_specific: bool,
    pub use_non_standard_stacks: bool,
    pub special_feature_optins: IdSet,
    pub purpose_consents: IdSet,
    pub purpose_legitimate_interests: IdSet,
    pub purpose_one_treatment: bool,
    pub publisher_country_code: String,
    pub vendor_consents: IdSet,
    pub vendor_legitimate_interests: IdSet,
    pub publisher_restrictions: Vec<PublisherRestriction>,
}

impl FromDataReader for Core {
    type Err = SectionDecodeError;

    fn from_data_reader(r: &mut DataReader) -> Result<Self, Self::Err> {
        let version = r.read_fixed_integer(6)?;
        if version != TCF_EU_V2_VERSION {
            return Err(SectionDecodeError::InvalidSegmentVersion {
                expected: TCF_EU_V2_VERSION,
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
        let is_service_specific = r.read_bool()?;
        let use_non_standard_stacks = r.read_bool()?;
        let special_feature_optins = r.read_fixed_bitfield(12)?;
        let purpose_consents = r.read_fixed_bitfield(24)?;
        let purpose_legitimate_interests = r.read_fixed_bitfield(24)?;
        let purpose_one_treatment = r.read_bool()?;
        let publisher_country_code = r.read_string(2)?;
        let vendor_consents = r.read_optimized_integer_range()?;
        let vendor_legitimate_interests = r.read_optimized_integer_range()?;
        let publisher_restrictions = r
            .read_array_of_ranges()?
            .into_iter()
            .map(PublisherRestriction::from)
            .collect();

        Ok(Self {
            created,
            last_updated,
            cmp_id,
            cmp_version,
            consent_screen,
            consent_language,
            vendor_list_version,
            policy_version,
            is_service_specific,
            use_non_standard_stacks,
            special_feature_optins,
            purpose_consents,
            purpose_legitimate_interests,
            purpose_one_treatment,
            publisher_country_code,
            vendor_consents,
            vendor_legitimate_interests,
            publisher_restrictions,
        })
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct PublisherRestriction {
    pub purpose_id: u8,
    pub restriction_type: RestrictionType,
    pub restricted_vendor_ids: IdSet,
}

impl From<Range> for PublisherRestriction {
    fn from(r: Range) -> Self {
        Self {
            purpose_id: r.key,
            restriction_type: RestrictionType::from_u8(r.range_type)
                .unwrap_or(RestrictionType::Undefined),
            restricted_vendor_ids: r.ids,
        }
    }
}

#[derive(Debug, Eq, PartialEq, FromPrimitive)]
pub enum RestrictionType {
    NotAllowed = 0,
    RequireConsent = 1,
    RequireLegitimateInterest = 2,
    Undefined = 3,
}

#[derive(Debug, Eq, PartialEq)]
pub struct PublisherPurposes {
    pub consents: IdSet,
    pub legitimate_interests: IdSet,
    pub custom_consents: IdSet,
    pub custom_legitimate_interests: IdSet,
}

impl FromDataReader for PublisherPurposes {
    type Err = SectionDecodeError;

    fn from_data_reader(r: &mut DataReader) -> Result<Self, SectionDecodeError> {
        let consents = r.read_fixed_bitfield(24)?;
        let legitimate_interests = r.read_fixed_bitfield(24)?;
        let custom_purposes_num = r.read_fixed_integer::<u8>(6)? as usize;
        let custom_consents = r.read_fixed_bitfield(custom_purposes_num)?;
        let custom_legitimate_interests = r.read_fixed_bitfield(custom_purposes_num)?;

        Ok(Self {
            consents,
            legitimate_interests,
            custom_consents,
            custom_legitimate_interests,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test]
    fn core_only() {
        let actual = TcfEuV2::from_str("CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA").unwrap();
        let expected = TcfEuV2 {
            core: Core {
                created: 1650492000,
                last_updated: 1650492000,
                cmp_id: 31,
                cmp_version: 640,
                consent_screen: 1,
                consent_language: "EN".to_string(),
                vendor_list_version: 126,
                policy_version: 2,
                is_service_specific: true,
                use_non_standard_stacks: false,
                special_feature_optins: Default::default(),
                purpose_consents: Default::default(),
                purpose_legitimate_interests: Default::default(),
                purpose_one_treatment: false,
                publisher_country_code: "DE".to_string(),
                vendor_consents: Default::default(),
                vendor_legitimate_interests: Default::default(),
                publisher_restrictions: vec![],
            },
            disclosed_vendors: None,
            publisher_purposes: None,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn with_disclosed_vendors() {
        let actual = TcfEuV2::from_str("COvFyGBOvFyGBAbAAAENAPCAAOAAAAAAAAAAAEEUACCKAAA.IFoEUQQgAIQwgIwQABAEAAAAOIAACAIAAAAQAIAgEAACEAAAAAgAQBAAAAAAAGBAAgAAAAAAAFAAECAAAgAAQARAEQAAAAAJAAIAAgAAAYQEAAAQmAgBC3ZAYzUw").unwrap();

        let expected = TcfEuV2 {
            core: Core {
                created: 1582243059,
                last_updated: 1582243059,
                cmp_id: 27,
                cmp_version: 0,
                consent_screen: 0,
                consent_language: "EN".to_string(),
                vendor_list_version: 15,
                policy_version: 2,
                is_service_specific: false,
                use_non_standard_stacks: false,
                special_feature_optins: Default::default(),
                purpose_consents: [1, 2, 3].into(),
                purpose_legitimate_interests: Default::default(),
                purpose_one_treatment: false,
                publisher_country_code: "AA".to_string(),
                vendor_consents: [2, 6, 8].into(),
                vendor_legitimate_interests: [2, 6, 8].into(),
                publisher_restrictions: vec![],
            },
            disclosed_vendors: Some(
                [
                    2, 6, 8, 12, 18, 23, 37, 42, 47, 48, 53, 61, 65, 66, 72, 88, 98, 127, 128, 129,
                    133, 153, 163, 192, 205, 215, 224, 243, 248, 281, 294, 304, 350, 351, 358, 371,
                    422, 424, 440, 447, 467, 486, 498, 502, 512, 516, 553, 556, 571, 587, 612, 613,
                    618, 626, 648, 653, 656, 657, 665, 676, 681, 683, 684, 686, 687, 688, 690, 691,
                    694, 702, 703, 707, 708, 711, 712, 714, 716, 719, 720,
                ]
                .into(),
            ),
            publisher_purposes: None,
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn with_publisher_purposes() {
        let actual =
            TcfEuV2::from_str("COvFyGBOvFyGBAbAAAENAPCAAOAAAAAAAAAAAEEUACCKAAA.ZAAgH9794ulA")
                .unwrap();

        let expected = TcfEuV2 {
            core: Core {
                created: 1582243059,
                last_updated: 1582243059,
                cmp_id: 27,
                cmp_version: 0,
                consent_screen: 0,
                consent_language: "EN".to_string(),
                vendor_list_version: 15,
                policy_version: 2,
                is_service_specific: false,
                use_non_standard_stacks: false,
                special_feature_optins: Default::default(),
                purpose_consents: [1, 2, 3].into(),
                purpose_legitimate_interests: Default::default(),
                purpose_one_treatment: false,
                publisher_country_code: "AA".to_string(),
                vendor_consents: [2, 6, 8].into(),
                vendor_legitimate_interests: [2, 6, 8].into(),
                publisher_restrictions: vec![],
            },
            disclosed_vendors: None,
            publisher_purposes: Some(PublisherPurposes {
                consents: [3, 16].into(),
                legitimate_interests: [
                    1, 2, 3, 4, 5, 6, 7, 9, 10, 11, 12, 14, 15, 16, 17, 18, 19, 21, 22, 23, 24,
                ]
                .into(),
                custom_consents: [1, 2, 4].into(),
                custom_legitimate_interests: [2, 4].into(),
            }),
        };

        assert_eq!(actual, expected);
    }

    #[test_case("COvFyGBOvFyGBAbAAAENAPCAAOAAAAAAAAAAAEEUACCKAAA.ZAAgH9794ulA.IFoEUQQgAIQwgIwQABAEAAAAOIAACAIAAAAQAIAgEAACEAAAAAgAQBAAAAAAAGBAAgAAAAAAAFAAECAAAgAAQARAEQAAAAAJAAIAAgAAAYQEAAAQmAgBC3ZAYzUw" ; "publisher purposes first")]
    #[test_case("COvFyGBOvFyGBAbAAAENAPCAAOAAAAAAAAAAAEEUACCKAAA.IFoEUQQgAIQwgIwQABAEAAAAOIAACAIAAAAQAIAgEAACEAAAAAgAQBAAAAAAAGBAAgAAAAAAAFAAECAAAgAAQARAEQAAAAAJAAIAAgAAAYQEAAAQmAgBC3ZAYzUw.ZAAgH9794ulA" ; "disclosed vendors first")]
    fn with_all_segments(s: &str) {
        let expected = TcfEuV2 {
            core: Core {
                created: 1582243059,
                last_updated: 1582243059,
                cmp_id: 27,
                cmp_version: 0,
                consent_screen: 0,
                consent_language: "EN".to_string(),
                vendor_list_version: 15,
                policy_version: 2,
                is_service_specific: false,
                use_non_standard_stacks: false,
                special_feature_optins: Default::default(),
                purpose_consents: [1, 2, 3].into(),
                purpose_legitimate_interests: Default::default(),
                purpose_one_treatment: false,
                publisher_country_code: "AA".to_string(),
                vendor_consents: [2, 6, 8].into(),
                vendor_legitimate_interests: [2, 6, 8].into(),
                publisher_restrictions: vec![],
            },
            disclosed_vendors: Some(
                [
                    2, 6, 8, 12, 18, 23, 37, 42, 47, 48, 53, 61, 65, 66, 72, 88, 98, 127, 128, 129,
                    133, 153, 163, 192, 205, 215, 224, 243, 248, 281, 294, 304, 350, 351, 358, 371,
                    422, 424, 440, 447, 467, 486, 498, 502, 512, 516, 553, 556, 571, 587, 612, 613,
                    618, 626, 648, 653, 656, 657, 665, 676, 681, 683, 684, 686, 687, 688, 690, 691,
                    694, 702, 703, 707, 708, 711, 712, 714, 716, 719, 720,
                ]
                .into(),
            ),
            publisher_purposes: Some(PublisherPurposes {
                consents: [3, 16].into(),
                legitimate_interests: [
                    1, 2, 3, 4, 5, 6, 7, 9, 10, 11, 12, 14, 15, 16, 17, 18, 19, 21, 22, 23, 24,
                ]
                .into(),
                custom_consents: [1, 2, 4].into(),
                custom_legitimate_interests: [2, 4].into(),
            }),
        };

        let actual = TcfEuV2::from_str(s).unwrap();
        assert_eq!(actual, expected);
    }

    #[test_case("CPX" => matches SectionDecodeError::Read(_) ; "decode error")]
    #[test_case("" => matches SectionDecodeError::Read(_) ; "empty string")]
    #[test_case("IFoEUQQgAIQwgIwQABAEAAAAOIAACAIAAAAQAIAgEAACEAAAAAgAQBAAAAAAAGBAAgAAAAAAAFAAECAAAgAAQARAEQAAAAAJAAIAAgAAAYQEAAAQmAgBC3ZAYzUw" => matches SectionDecodeError::InvalidSegmentVersion { .. } ; "disclosed vendors only")]
    #[test_case("ZAAgH9794ulA" => matches SectionDecodeError::InvalidSegmentVersion { .. } ; "publisher purposes only")]
    #[test_case("IFoEUQQgAIQwgIwQABAEAAAAOIAACAIAAAAQAIAgEAACEAAAAAgAQBAAAAAAAGBAAgAAAAAAAFAAECAAAgAAQARAEQAAAAAJAAIAAgAAAYQEAAAQmAgBC3ZAYzUw.ZAAgH9794ulA" => matches SectionDecodeError::InvalidSegmentVersion { .. } ; "disclosed vendors and publisher purposes")]
    #[test_case("ZAAgH9794ulA.IFoEUQQgAIQwgIwQABAEAAAAOIAACAIAAAAQAIAgEAACEAAAAAgAQBAAAAAAAGBAAgAAAAAAAFAAECAAAgAAQARAEQAAAAAJAAIAAgAAAYQEAAAQmAgBC3ZAYzUw" => matches SectionDecodeError::InvalidSegmentVersion { .. } ; "publisher purposes and disclosed vendors")]
    fn error(s: &str) -> SectionDecodeError {
        TcfEuV2::from_str(s).unwrap_err()
    }
}

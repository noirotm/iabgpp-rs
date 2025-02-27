use crate::core::{DataReader, Range};
use crate::sections::{IdSet, SectionDecodeError};
use iab_gpp_derive::{FromDataReader, GPPSection};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, Eq, PartialEq, GPPSection)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[non_exhaustive]
#[gpp(with_optional_segments)]
pub struct TcfEuV2 {
    pub core: Core,
    #[gpp(optional_segment_type = 1, optimized_integer_range)]
    pub disclosed_vendors: Option<IdSet>,
    #[gpp(optional_segment_type = 2, optimized_integer_range)]
    pub allowed_vendors: Option<IdSet>,
    #[gpp(optional_segment_type = 3)]
    pub publisher_purposes: Option<PublisherPurposes>,
}

#[derive(Debug, Eq, PartialEq, FromDataReader)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[non_exhaustive]
#[gpp(section_version = 2)]
pub struct Core {
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
    pub policy_version: u8,
    pub is_service_specific: bool,
    pub use_non_standard_stacks: bool,
    #[gpp(fixed_bitfield(12))]
    pub special_feature_optins: IdSet,
    #[gpp(fixed_bitfield(24))]
    pub purpose_consents: IdSet,
    #[gpp(fixed_bitfield(24))]
    pub purpose_legitimate_interests: IdSet,
    pub purpose_one_treatment: bool,
    #[gpp(string(2))]
    pub publisher_country_code: String,
    #[gpp(optimized_integer_range)]
    pub vendor_consents: IdSet,
    #[gpp(optimized_integer_range)]
    pub vendor_legitimate_interests: IdSet,
    #[gpp(parse_with = parse_publisher_restrictions)]
    pub publisher_restrictions: Vec<PublisherRestriction>,
}

fn parse_publisher_restrictions(
    r: &mut DataReader,
) -> Result<Vec<PublisherRestriction>, SectionDecodeError> {
    Ok(r.read_array_of_ranges()?
        .into_iter()
        .map(PublisherRestriction::from)
        .collect())
}

#[derive(Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
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
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum RestrictionType {
    NotAllowed = 0,
    RequireConsent = 1,
    RequireLegitimateInterest = 2,
    Undefined = 3,
}

#[derive(Debug, Eq, PartialEq, FromDataReader)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[non_exhaustive]
pub struct PublisherPurposes {
    #[gpp(fixed_bitfield(24))]
    pub consents: IdSet,
    #[gpp(fixed_bitfield(24))]
    pub legitimate_interests: IdSet,
    #[gpp(fixed_bitfield(n as usize), where(n = fixed_integer(6)))]
    pub custom_consents: IdSet,
    #[gpp(fixed_bitfield(n as usize))]
    pub custom_legitimate_interests: IdSet,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
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
            allowed_vendors: None,
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
            allowed_vendors: None,
            publisher_purposes: None,
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn with_allowed_vendors() {
        let actual = TcfEuV2::from_str("COvFyGBOvFyGBAbAAAENAPCAAOAAAAAAAAAAAEEUACCKAAA.QFoEUQQgAIQwgIwQABAEAAAAOIAACAIAAAAQAIAgEAACEAAAAAgAQBAAAAAAAGBAAgAAAAAAAFAAECAAAgAAQARAEQAAAAAJAAIAAgAAAYQEAAAQmAgBC3ZAYzUw").unwrap();

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
            allowed_vendors: Some(
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
            allowed_vendors: None,
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
            allowed_vendors: None,
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
    #[test_case("IFoEUQQgAIQwgIwQABAEAAAAOIAACAIAAAAQAIAgEAACEAAAAAgAQBAAAAAAAGBAAgAAAAAAAFAAECAAAgAAQARAEQAAAAAJAAIAAgAAAYQEAAAQmAgBC3ZAYzUw" => matches SectionDecodeError::UnknownSegmentVersion { .. } ; "disclosed vendors only")]
    #[test_case("ZAAgH9794ulA" => matches SectionDecodeError::UnknownSegmentVersion { .. } ; "publisher purposes only")]
    #[test_case("IFoEUQQgAIQwgIwQABAEAAAAOIAACAIAAAAQAIAgEAACEAAAAAgAQBAAAAAAAGBAAgAAAAAAAFAAECAAAgAAQARAEQAAAAAJAAIAAgAAAYQEAAAQmAgBC3ZAYzUw.ZAAgH9794ulA" => matches SectionDecodeError::UnknownSegmentVersion { .. } ; "disclosed vendors and publisher purposes")]
    #[test_case("ZAAgH9794ulA.IFoEUQQgAIQwgIwQABAEAAAAOIAACAIAAAAQAIAgEAACEAAAAAgAQBAAAAAAAGBAAgAAAAAAAFAAECAAAgAAQARAEQAAAAAJAAIAAgAAAYQEAAAQmAgBC3ZAYzUw" => matches SectionDecodeError::UnknownSegmentVersion { .. } ; "publisher purposes and disclosed vendors")]
    fn error(s: &str) -> SectionDecodeError {
        TcfEuV2::from_str(s).unwrap_err()
    }
}

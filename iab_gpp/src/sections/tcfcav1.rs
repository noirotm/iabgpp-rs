use crate::core::{DataReader, GenericRange};
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
pub struct TcfCaV1 {
    pub core: Core,
    #[gpp(optional_segment_type = 1, optimized_range)]
    pub disclosed_vendors: Option<IdSet>,
    #[gpp(optional_segment_type = 3)]
    pub publisher_purposes: Option<PublisherPurposes>,
}

#[derive(Debug, Eq, PartialEq, FromDataReader)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[non_exhaustive]
#[gpp(section_version = 1)]
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
    pub use_non_standard_stacks: bool,
    #[gpp(fixed_bitfield(12))]
    pub special_feature_express_consents: IdSet,
    #[gpp(fixed_bitfield(24))]
    pub purpose_express_consents: IdSet,
    #[gpp(fixed_bitfield(24))]
    pub purpose_implied_consents: IdSet,
    // BUG: specification says optimized_range
    #[gpp(optimized_integer_range)]
    pub vendor_express_consents: IdSet,
    // BUG: specification says optimized_range
    #[gpp(optimized_integer_range)]
    pub vendor_implied_consents: IdSet,
    /// Introduced in TCF CA v1.1
    #[gpp(parse_with = parse_publisher_restrictions)]
    pub pub_restrictions: Vec<PublisherRestriction>,
}

fn parse_publisher_restrictions(
    r: &mut DataReader,
) -> Result<Vec<PublisherRestriction>, SectionDecodeError> {
    Ok(r.read_n_array_of_ranges(6, 2)
        .unwrap_or_default()
        .into_iter()
        .map(|r| PublisherRestriction {
            purpose_id: r.key,
            restriction_type: RestrictionType::from_u8(r.range_type)
                .unwrap_or(RestrictionType::Undefined),
            restricted_vendor_ids: r.ids,
        })
        .collect())
}

#[derive(Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct PublisherRestriction {
    pub purpose_id: u8,
    pub restriction_type: RestrictionType,
    pub restricted_vendor_ids: IdSet,
}

impl From<GenericRange<u8, u8>> for PublisherRestriction {
    fn from(r: GenericRange<u8, u8>) -> Self {
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
    RequireExpressConsent = 1,
    RequireImpliedConsent = 2,
    Undefined = 3,
}

#[derive(Debug, Eq, PartialEq, FromDataReader)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[non_exhaustive]
pub struct PublisherPurposes {
    #[gpp(fixed_bitfield(24))]
    pub purpose_express_consents: IdSet,
    #[gpp(fixed_bitfield(24))]
    pub purpose_implied_consents: IdSet,
    #[gpp(fixed_bitfield(n as usize), where(n = fixed_integer(6)))]
    pub custom_purpose_express_consents: IdSet,
    #[gpp(fixed_bitfield(n as usize))]
    pub custom_purpose_implied_consents: IdSet,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
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
                pub_restrictions: Default::default(),
            },
            disclosed_vendors: None,
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
                pub_restrictions: Default::default(),
            },
            disclosed_vendors: None,
            publisher_purposes: Some(PublisherPurposes {
                purpose_express_consents: Default::default(),
                purpose_implied_consents: Default::default(),
                custom_purpose_express_consents: Default::default(),
                custom_purpose_implied_consents: Default::default(),
            }),
        };

        assert_eq!(actual, expected);
    }

    #[test_case("BPX" => matches SectionDecodeError::Read(_) ; "decode error")]
    #[test_case("" => matches SectionDecodeError::Read(_) ; "empty string")]
    fn error(s: &str) -> SectionDecodeError {
        TcfCaV1::from_str(s).unwrap_err()
    }

    #[test]
    fn full_str() {
        let actual =
            TcfCaV1::from_str("BQMC4oAQMC4oAPoABABGB0CYAf8AAf8AAAqdA-AAUABwAFQALQAaABLACgAF0ANoAdwA_QCCAIQARQAnwBWgC3AGUANMAc4A7gCAQElASYAnYBPwDFAGaAM6AZ8A14B_AEngJyAT-Ao8BUQCpQFvALhAXQAvcBf4DBwGYANNAbUA3EBxoDxAHmgPkAgIBCQCNwEpYJgAmCBNUCa4E5gJ-AUmApYBU4FToHwACgAOAAqABaADQAJYAUAAugBtADuAH6AQQBCACKAE-AK0AW4AygBpgDnAHcAQCAkoCTAE7AJ-AYoAzQBnQDPgGvAP4Ak8BOQCfwFHgKiAVKAt4BcIC6AF7gL_AYOAzABpoDagG4gONAeIA80B8gEBAISARuAlLBMAEwQJqgTXAnMBPwCkwFLAKnAAAA.YAAAAAAAAAA").unwrap();
        let expected = TcfCaV1 {
            core: Core {
                created: 1738195200,
                last_updated: 1738195200,
                cmp_id: 1000,
                cmp_version: 1,
                consent_screen: 0,
                consent_language: "BG".to_string(),
                vendor_list_version: 116,
                policy_version: 2,
                use_non_standard_stacks: false,
                special_feature_express_consents: [1, 2].into(),
                purpose_express_consents: [1, 2, 3, 4, 5, 6, 7, 8, 9].into(),
                purpose_implied_consents: [1, 2, 3, 4, 5, 6, 7, 8, 9].into(),
                vendor_express_consents: [
                    10, 28, 42, 45, 52, 75, 80, 93, 109, 119, 126, 130, 131, 132, 138, 159, 173,
                    183, 202, 211, 231, 238, 256, 293, 294, 315, 319, 394, 410, 413, 415, 431, 508,
                    591, 626, 639, 655, 674, 677, 734, 737, 744, 759, 767, 775, 816, 845, 874, 881,
                    909, 964, 973, 996, 1028, 1060, 1134, 1189, 1216, 1217, 1237, 1239, 1254, 1276,
                    1318, 1324, 1358,
                ]
                .into(),
                vendor_implied_consents: [
                    10, 28, 42, 45, 52, 75, 80, 93, 109, 119, 126, 130, 131, 132, 138, 159, 173,
                    183, 202, 211, 231, 238, 256, 293, 294, 315, 319, 394, 410, 413, 415, 431, 508,
                    591, 626, 639, 655, 674, 677, 734, 737, 744, 759, 767, 775, 816, 845, 874, 881,
                    909, 964, 973, 996, 1028, 1060, 1134, 1189, 1216, 1217, 1237, 1239, 1254, 1276,
                    1318, 1324, 1358,
                ]
                .into(),
                pub_restrictions: Default::default(),
            },
            disclosed_vendors: None,
            publisher_purposes: Some(PublisherPurposes {
                purpose_express_consents: Default::default(),
                purpose_implied_consents: Default::default(),
                custom_purpose_express_consents: Default::default(),
                custom_purpose_implied_consents: Default::default(),
            }),
        };

        assert_eq!(actual, expected);
    }
}

use crate::core::{DataReader, DecodeExt};
use crate::sections::{SectionDecodeError, VendorList};
use std::iter::repeat_with;
use std::str::FromStr;

const TCF_EU_V2_VERSION: u8 = 2;
const TCF_EU_V2_DISCLOSED_VENDORS_SEGMENT_TYPE: u8 = 1;
const TCF_EU_V2_PUBLISHER_PURPOSES_SEGMENT_TYPE: u8 = 3;

#[derive(Debug, Eq, PartialEq)]
pub struct TcfEuV2 {
    pub core: Core,
    pub disclosed_vendors: Option<VendorList>,
    pub publisher_purposes: Option<PublisherPurposes>,
}

impl FromStr for TcfEuV2 {
    type Err = SectionDecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut sections_iter = s.split('.');

        // first mandatory section is the core segment
        let core = sections_iter
            .next()
            .ok_or_else(|| SectionDecodeError::UnexpectedEndOfString(s.to_string()))?;
        let core = Core::from_str(core)?;

        let mut tcfeuv2 = Self {
            core,
            disclosed_vendors: None,
            publisher_purposes: None,
        };

        // next sections are optional and type depend on first int(3) value
        for s in sections_iter {
            let s = s.decode_base64_url()?;
            let mut r = DataReader::new(&s);

            let section_type = r.read_fixed_integer::<u8>(3)?;
            match section_type {
                TCF_EU_V2_DISCLOSED_VENDORS_SEGMENT_TYPE => {
                    tcfeuv2.disclosed_vendors = Some(r.read_optimized_integer_range()?);
                }
                TCF_EU_V2_PUBLISHER_PURPOSES_SEGMENT_TYPE => {
                    tcfeuv2.publisher_purposes = Some(PublisherPurposes::parse(&mut r)?);
                }
                n => {
                    return Err(SectionDecodeError::UnknownSegmentType { segment_type: n });
                }
            }
        }

        Ok(tcfeuv2)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Core {
    pub version: u8,
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
    pub special_feature_optins: Vec<bool>,
    pub purpose_consents: Vec<bool>,
    pub purpose_legitimate_interests: Vec<bool>,
    pub purpose_one_treatment: bool,
    pub publisher_country_code: String,
    pub vendor_consents: VendorList,
    pub vendor_legitimate_interests: VendorList,
    pub publisher_restrictions: Vec<PublisherRestriction>,
}

impl FromStr for Core {
    type Err = SectionDecodeError;

    fn from_str(core: &str) -> Result<Core, SectionDecodeError> {
        let core = core.decode_base64_url()?;
        let mut r = DataReader::new(&core);

        let version = r.read_fixed_integer::<u8>(6)?;
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

        let publisher_restrictions_num = r.read_fixed_integer::<u8>(6)?;
        let publisher_restrictions = repeat_with(|| PublisherRestriction::parse(&mut r))
            .take(publisher_restrictions_num as usize)
            .collect::<Result<_, _>>()?;

        Ok(Self {
            version,
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
    pub restricted_vendor_ids: VendorList,
}

impl PublisherRestriction {
    fn parse(r: &mut DataReader) -> Result<Self, SectionDecodeError> {
        let purpose_id = r.read_fixed_integer(6)?;
        let restriction_type = match r.read_fixed_integer::<u8>(2)? {
            0 => RestrictionType::NotAllowed,
            1 => RestrictionType::RequireConsent,
            2 => RestrictionType::RequireLegitimateInterest,
            3 => RestrictionType::Undefined,
            _ => unreachable!(), // any other value can't exist here as we read 2 bits only
        };
        let restricted_vendor_ids = r.read_optimized_integer_range()?;

        Ok(Self {
            purpose_id,
            restriction_type,
            restricted_vendor_ids,
        })
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum RestrictionType {
    NotAllowed,
    RequireConsent,
    RequireLegitimateInterest,
    Undefined,
}

#[derive(Debug, Eq, PartialEq)]
pub struct PublisherPurposes {
    pub consents: Vec<bool>,
    pub legitimate_interests: Vec<bool>,
    pub custom_purposes_num: u8,
    pub custom_consents: Vec<bool>,
    pub custom_legitimate_interests: Vec<bool>,
}

impl PublisherPurposes {
    fn parse(r: &mut DataReader) -> Result<Self, SectionDecodeError> {
        let consents = r.read_fixed_bitfield(24)?;
        let legitimate_interests = r.read_fixed_bitfield(24)?;
        let custom_purposes_num = r.read_fixed_integer::<u8>(6)?;
        let custom_consents = r.read_fixed_bitfield(custom_purposes_num as usize)?;
        let custom_legitimate_interests = r.read_fixed_bitfield(custom_purposes_num as usize)?;

        Ok(Self {
            consents,
            legitimate_interests,
            custom_purposes_num,
            custom_consents,
            custom_legitimate_interests,
        })
    }
}

/*pub struct BitField<const N: usize> {
    bits: [bool; N],
}

impl<const N: usize> BitField<N> {
    fn try_from_slice(bits: &[bool]) -> Result<Self, TryFromSliceError> {
        Ok(Self {
            bits: bits.try_into()?,
        })
    }

    fn iter(&self) -> impl Iterator<Item = (usize, &bool)> {
        self.bits.iter().enumerate().map(|(i, b)| (i + 1, b))
    }

    fn get_by_id(&self, id: usize) -> Option<bool> {
        self.bits.get(id.checked_sub(1)?).cloned()
    }
}*/

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sections::uspv1::Consent::No;
    use base64::Engine;
    use std::iter::repeat;

    #[test]
    fn core_only() {
        let actual = TcfEuV2::from_str("CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA").unwrap();
        let expected = TcfEuV2 {
            core: Core {
                version: TCF_EU_V2_VERSION,
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
                special_feature_optins: vec![false; 12],
                purpose_consents: vec![false; 24],
                purpose_legitimate_interests: vec![false; 24],
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
                version: TCF_EU_V2_VERSION,
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
                special_feature_optins: vec![false; 12],
                purpose_consents: repeat(true).take(3).chain(repeat(false).take(21)).collect(),
                purpose_legitimate_interests: vec![false; 24],
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
                version: TCF_EU_V2_VERSION,
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
                special_feature_optins: vec![false; 12],
                purpose_consents: repeat(true).take(3).chain(repeat(false).take(21)).collect(),
                purpose_legitimate_interests: vec![false; 24],
                purpose_one_treatment: false,
                publisher_country_code: "AA".to_string(),
                vendor_consents: [2, 6, 8].into(),
                vendor_legitimate_interests: [2, 6, 8].into(),
                publisher_restrictions: vec![],
            },
            disclosed_vendors: None,
            publisher_purposes: Some(PublisherPurposes {
                consents: vec![
                    false, false, true, false, false, false, false, false, false, false, false,
                    false, false, false, false, true, false, false, false, false, false, false,
                    false, false,
                ],
                legitimate_interests: vec![
                    true, true, true, true, true, true, true, false, true, true, true, true, false,
                    true, true, true, true, true, true, false, true, true, true, true,
                ],
                custom_purposes_num: 5,
                custom_consents: vec![true, true, false, true, false],
                custom_legitimate_interests: vec![false, true, false, true, false],
            }),
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn decode_error() {
        let r = TcfEuV2::from_str("CPX");
        assert!(matches!(r, Err(SectionDecodeError::DecodeSegment(_))));
    }

    #[test]
    fn empty_string() {
        let r = TcfEuV2::from_str("");
        assert!(matches!(r, Err(SectionDecodeError::Read(_))));
    }
}

use crate::core::{DataReader, DecodeExt};
use crate::sections::SectionDecodeError;
use std::array::TryFromSliceError;
use std::iter::repeat_with;
use std::num::NonZeroUsize;
use std::ops::Index;
use std::str::FromStr;

const TCFEUV2_CORE_SEGMENT_VERSION: u8 = 2;
const TCFEUV2_DISCLOSED_VENDORS_SEGMENT_TYPE: u8 = 1;
const TCFEUV2_PUBLISHER_PURPOSES_SEGMENT_TYPE: u8 = 3;

pub struct TcfEuV2 {
    core: Core,
    disclosed_vendors: Option<Vec<u64>>,
    publisher_purposes: Option<PublisherPurposes>,
}

pub struct Core {
    version: u8,
    created: i64,
    last_updated: i64,
    cmp_id: u16,
    cmp_version: u16,
    consent_screen: u8,
    consent_language: String,
    vendor_list_version: u16,
    policy_version: u8,
    is_service_specific: bool,
    use_non_standard_stacks: bool,
    special_feature_optins: Vec<bool>,
    purpose_consents: Vec<bool>,
    purpose_legitimate_interests: Vec<bool>,
    purpose_one_treatment: bool,
    publisher_country_code: String,
    vendor_consents: Vec<u64>,
    vendor_legitimate_interests: Vec<u64>,
    publisher_restrictions: Vec<PublisherRestriction>,
}

pub struct PublisherRestriction {
    purpose_id: u8,
    restriction_type: RestrictionType,
    restricted_vendor_ids: Vec<u64>,
}

pub enum RestrictionType {
    NotAllowed,
    RequireConsent,
    RequireLegitimateInterest,
    Undefined,
}

pub struct PublisherPurposes {
    consents: Vec<bool>,
    legitimate_interests: Vec<bool>,
    custom_purposes_num: u8,
    custom_consents: Vec<bool>,
    custom_legitimate_interests: Vec<bool>,
}

impl FromStr for TcfEuV2 {
    type Err = SectionDecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut sections_iter = s.split('.');

        // first mandatory section is the core segment
        let core = sections_iter
            .next()
            .ok_or_else(|| SectionDecodeError::UnexpectedEndOfString(s.to_string()))?;
        let core = TcfEuV2::parse_core_segment(core)?;

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
                TCFEUV2_DISCLOSED_VENDORS_SEGMENT_TYPE => {
                    tcfeuv2.disclosed_vendors = Some(r.read_optimized_integer_range()?);
                }
                TCFEUV2_PUBLISHER_PURPOSES_SEGMENT_TYPE => {
                    tcfeuv2.publisher_purposes = Some(TcfEuV2::parse_publisher_purposes(&mut r)?);
                }
                n => {
                    return Err(SectionDecodeError::UnknownSegmentType { segment_type: n });
                }
            }
        }

        Ok(tcfeuv2)
    }
}

impl TcfEuV2 {
    fn parse_core_segment(core: &str) -> Result<Core, SectionDecodeError> {
        let core = core.decode_base64_url()?;
        let mut r = DataReader::new(&core);

        let version = r.read_fixed_integer::<u8>(6)?;
        if version != TCFEUV2_CORE_SEGMENT_VERSION {
            return Err(SectionDecodeError::InvalidSegmentVersion {
                expected: TCFEUV2_CORE_SEGMENT_VERSION,
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
        let publisher_restrictions = repeat_with(|| Self::parse_publisher_restriction(&mut r))
            .take(publisher_restrictions_num as usize)
            .collect::<Result<_, _>>()?;

        Ok(Core {
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

    fn parse_publisher_restriction(
        r: &mut DataReader,
    ) -> Result<PublisherRestriction, SectionDecodeError> {
        let purpose_id = r.read_fixed_integer(6)?;
        let restriction_type = match r.read_fixed_integer::<u8>(2)? {
            0 => RestrictionType::NotAllowed,
            1 => RestrictionType::RequireConsent,
            2 => RestrictionType::RequireLegitimateInterest,
            3 => RestrictionType::Undefined,
            _ => unreachable!(), // any other value can't exist here as we read 2 bits only
        };
        let restricted_vendor_ids = r.read_optimized_integer_range()?;

        Ok(PublisherRestriction {
            purpose_id,
            restriction_type,
            restricted_vendor_ids,
        })
    }

    fn parse_publisher_purposes(
        r: &mut DataReader,
    ) -> Result<PublisherPurposes, SectionDecodeError> {
        let consents = r.read_fixed_bitfield(24)?;
        let legitimate_interests = r.read_fixed_bitfield(24)?;
        let custom_purposes_num = r.read_fixed_integer::<u8>(6)?;
        let custom_consents = r.read_fixed_bitfield(custom_purposes_num as usize)?;
        let custom_legitimate_interests = r.read_fixed_bitfield(custom_purposes_num as usize)?;

        Ok(PublisherPurposes {
            consents,
            legitimate_interests,
            custom_purposes_num,
            custom_consents,
            custom_legitimate_interests,
        })
    }
}

pub struct BitField<const N: usize> {
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
}

impl<const N: usize> Index<NonZeroUsize> for BitField<N> {
    type Output = bool;

    fn index(&self, index: NonZeroUsize) -> &Self::Output {
        &self.bits[index.get()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tcfeuv2() {
        let t = TcfEuV2::from_str("CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA").unwrap();

        assert_eq!(t.core.version, TCFEUV2_CORE_SEGMENT_VERSION);
        assert_eq!(t.core.created, 1650492000);
        assert_eq!(t.core.last_updated, 1650492000);
        assert_eq!(t.core.cmp_id, 31);
        assert_eq!(t.core.cmp_version, 640);
        assert_eq!(t.core.consent_screen, 1);
        assert_eq!(t.core.consent_language, "EN");
        assert_eq!(t.core.vendor_list_version, 126);
        assert_eq!(t.core.policy_version, 2);
        assert!(t.core.is_service_specific);
        assert!(!t.core.use_non_standard_stacks);
        assert_eq!(t.core.special_feature_optins, vec![false; 12]);
        assert_eq!(t.core.purpose_consents, vec![false; 24]);
        assert_eq!(t.core.purpose_legitimate_interests, vec![false; 24]);
        assert!(!t.core.purpose_one_treatment);
        assert_eq!(t.core.publisher_country_code, "DE");
        assert!(t.core.vendor_consents.is_empty());
        assert!(t.core.vendor_legitimate_interests.is_empty());
        assert!(t.core.publisher_restrictions.is_empty());
        assert!(t.disclosed_vendors.is_none());
        assert!(t.publisher_purposes.is_none());
    }

    #[test]
    fn test_decode_error() {
        let e = TcfEuV2::from_str("CPX");
        assert!(matches!(e, Err(SectionDecodeError::DecodeSegment(_))));
    }
}

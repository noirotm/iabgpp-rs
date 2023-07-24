use crate::core::{DataReader, DecodeExt};
use crate::sections::{SectionDecodeError, VendorList};
use std::iter::repeat_with;
use std::str::FromStr;

const TCFEUV2_CORE_SEGMENT_VERSION: u8 = 2;
const TCFEUV2_DISCLOSED_VENDORS_SEGMENT_TYPE: u8 = 1;
const TCFEUV2_PUBLISHER_PURPOSES_SEGMENT_TYPE: u8 = 3;

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
                TCFEUV2_DISCLOSED_VENDORS_SEGMENT_TYPE => {
                    tcfeuv2.disclosed_vendors = Some(r.read_optimized_integer_range()?);
                }
                TCFEUV2_PUBLISHER_PURPOSES_SEGMENT_TYPE => {
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

pub enum RestrictionType {
    NotAllowed,
    RequireConsent,
    RequireLegitimateInterest,
    Undefined,
}

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

    #[test]
    fn parse_tcfeuv2() {
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
    fn decode_error() {
        let e = TcfEuV2::from_str("CPX");
        assert!(matches!(e, Err(SectionDecodeError::DecodeSegment(_))));
    }
}

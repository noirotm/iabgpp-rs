use crate::core::{DataRead, Range};
use crate::sections::{IdSet, SectionDecodeError};
use bitstream_io::BitRead;
use iab_gpp_derive::{FromBitStream, GPPSection};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, GPPSection)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

#[derive(Debug, Eq, PartialEq, FromBitStream)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[non_exhaustive]
#[gpp(section_version = 2)]
pub struct Core {
    #[gpp(datetime_as_unix_timestamp)]
    pub created: u64,
    #[gpp(datetime_as_unix_timestamp)]
    pub last_updated: u64,
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

fn parse_publisher_restrictions<R: BitRead + ?Sized>(
    mut r: &mut R,
) -> Result<Vec<PublisherRestriction>, SectionDecodeError> {
    Ok(r.read_array_of_ranges()?
        .into_iter()
        .map(PublisherRestriction::from)
        .collect())
}

#[derive(Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum RestrictionType {
    NotAllowed = 0,
    RequireConsent = 1,
    RequireLegitimateInterest = 2,
    Undefined = 3,
}

#[derive(Debug, Eq, PartialEq, FromBitStream)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[non_exhaustive]
pub struct PublisherPurposes {
    #[gpp(fixed_bitfield(24))]
    pub consents: IdSet,
    #[gpp(fixed_bitfield(24))]
    pub legitimate_interests: IdSet,
    #[gpp(fixed_bitfield(n as usize), where(n = unsigned_var(6)))]
    pub custom_consents: IdSet,
    #[gpp(fixed_bitfield(n as usize))]
    pub custom_legitimate_interests: IdSet,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use test_case::test_case;

    #[test_case("CPX" => matches SectionDecodeError::Read { .. } ; "decode error")]
    #[test_case("" => matches SectionDecodeError::Read { .. } ; "empty string")]
    #[test_case("IFoEUQQgAIQwgIwQABAEAAAAOIAACAIAAAAQAIAgEAACEAAAAAgAQBAAAAAAAGBAAgAAAAAAAFAAECAAAgAAQARAEQAAAAAJAAIAAgAAAYQEAAAQmAgBC3ZAYzUw" => matches SectionDecodeError::UnknownSegmentVersion { .. } ; "disclosed vendors only")]
    #[test_case("ZAAgH9794ulA" => matches SectionDecodeError::UnknownSegmentVersion { .. } ; "publisher purposes only")]
    #[test_case("IFoEUQQgAIQwgIwQABAEAAAAOIAACAIAAAAQAIAgEAACEAAAAAgAQBAAAAAAAGBAAgAAAAAAAFAAECAAAgAAQARAEQAAAAAJAAIAAgAAAYQEAAAQmAgBC3ZAYzUw.ZAAgH9794ulA" => matches SectionDecodeError::UnknownSegmentVersion { .. } ; "disclosed vendors and publisher purposes")]
    #[test_case("ZAAgH9794ulA.IFoEUQQgAIQwgIwQABAEAAAAOIAACAIAAAAQAIAgEAACEAAAAAgAQBAAAAAAAGBAAgAAAAAAAFAAECAAAgAAQARAEQAAAAAJAAIAAgAAAYQEAAAQmAgBC3ZAYzUw" => matches SectionDecodeError::UnknownSegmentVersion { .. } ; "publisher purposes and disclosed vendors")]
    fn error(s: &str) -> SectionDecodeError {
        TcfEuV2::from_str(s).unwrap_err()
    }
}

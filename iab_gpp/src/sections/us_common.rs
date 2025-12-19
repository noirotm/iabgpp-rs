use crate::sections::SectionDecodeError;
use bitstream_io::BitRead;
use iab_gpp_derive::FromBitStream;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::FromPrimitive;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Eq, PartialEq, FromPrimitive, ToPrimitive, FromBitStream)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Notice {
    #[default]
    NotApplicable = 0,
    Provided = 1,
    NotProvided = 2,
}

#[derive(Debug, Default, Eq, PartialEq, FromPrimitive, ToPrimitive, FromBitStream)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum OptOut {
    #[default]
    NotApplicable = 0,
    OptedOut = 1,
    DidNotOptOut = 2,
}

#[derive(Debug, Default, Eq, PartialEq, FromPrimitive, ToPrimitive, FromBitStream)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Consent {
    #[default]
    NotApplicable = 0,
    NoConsent = 1,
    Consent = 2,
}

#[derive(Debug, Default, Eq, PartialEq, FromPrimitive, ToPrimitive, FromBitStream)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum MspaSupport {
    #[default]
    NotApplicable = 0,
    Yes = 1,
    No = 2,
}

#[derive(Debug, Default, Eq, PartialEq, FromPrimitive, ToPrimitive, FromBitStream)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum MspaMode {
    #[default]
    NotApplicable = 0,
    OptOutOption = 1,
    ServiceProvider = 2,
}

pub(crate) fn parse_mspa_covered_transaction<R: BitRead + ?Sized>(
    r: &mut R,
) -> Result<bool, SectionDecodeError> {
    let val = r.read_unsigned::<2, u8>()?;
    match val {
        1 => Ok(true),
        2 => Ok(false),
        v => Err(SectionDecodeError::InvalidFieldValue {
            expected: "1 or 2".to_string(),
            found: v.to_string(),
        }),
    }
}

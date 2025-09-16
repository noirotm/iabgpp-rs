use crate::sections::SectionDecodeError;
use bitstream_io::{BitRead, FromBitStream};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::FromPrimitive;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::io;

#[derive(Debug, Eq, PartialEq, FromPrimitive, ToPrimitive)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Notice {
    NotApplicable = 0,
    Provided = 1,
    NotProvided = 2,
}

impl FromBitStream for Notice {
    type Error = io::Error;

    fn from_reader<R: BitRead + ?Sized>(r: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(Self::from_u8(r.read_unsigned::<2, u8>()?).unwrap_or(Self::NotApplicable))
    }
}

#[derive(Debug, Eq, PartialEq, FromPrimitive, ToPrimitive)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum OptOut {
    NotApplicable = 0,
    OptedOut = 1,
    DidNotOptOut = 2,
}

impl FromBitStream for OptOut {
    type Error = io::Error;

    fn from_reader<R: BitRead + ?Sized>(r: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(Self::from_u8(r.read_unsigned::<2, u8>()?).unwrap_or(Self::NotApplicable))
    }
}

#[derive(Debug, Eq, PartialEq, FromPrimitive, ToPrimitive)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Consent {
    NotApplicable = 0,
    NoConsent = 1,
    Consent = 2,
}

impl FromBitStream for Consent {
    type Error = io::Error;

    fn from_reader<R: BitRead + ?Sized>(r: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(Self::from_u8(r.read_unsigned::<2, u8>()?).unwrap_or(Self::NotApplicable))
    }
}

#[derive(Debug, Eq, PartialEq, FromPrimitive, ToPrimitive)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum MspaMode {
    NotApplicable = 0,
    Yes = 1,
    No = 2,
}

impl FromBitStream for MspaMode {
    type Error = io::Error;

    fn from_reader<R: BitRead + ?Sized>(r: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(Self::from_u8(r.read_unsigned::<2, u8>()?).unwrap_or(Self::NotApplicable))
    }
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

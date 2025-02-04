use crate::core::{DataReader, FromDataReader};
use crate::sections::SectionDecodeError;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::FromPrimitive;
#[cfg(feature = "serde")]
use serde::Serialize;
use std::io;

#[derive(Debug, Eq, PartialEq, FromPrimitive, ToPrimitive)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum Notice {
    NotApplicable = 0,
    Provided = 1,
    NotProvided = 2,
}

impl FromDataReader for Notice {
    type Err = io::Error;

    fn from_data_reader(r: &mut DataReader) -> Result<Self, Self::Err> {
        Ok(Self::from_u8(r.read_fixed_integer(2)?).unwrap_or(Self::NotApplicable))
    }
}

#[derive(Debug, Eq, PartialEq, FromPrimitive, ToPrimitive)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum OptOut {
    NotApplicable = 0,
    OptedOut = 1,
    DidNotOptOut = 2,
}

impl FromDataReader for OptOut {
    type Err = io::Error;

    fn from_data_reader(r: &mut DataReader) -> Result<Self, Self::Err> {
        Ok(Self::from_u8(r.read_fixed_integer(2)?).unwrap_or(Self::NotApplicable))
    }
}

#[derive(Debug, Eq, PartialEq, FromPrimitive, ToPrimitive)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum Consent {
    NotApplicable = 0,
    NoConsent = 1,
    Consent = 2,
}

impl FromDataReader for Consent {
    type Err = io::Error;

    fn from_data_reader(r: &mut DataReader) -> Result<Self, Self::Err> {
        Ok(Self::from_u8(r.read_fixed_integer(2)?).unwrap_or(Self::NotApplicable))
    }
}

#[derive(Debug, Eq, PartialEq, FromPrimitive, ToPrimitive)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum MspaMode {
    NotApplicable = 0,
    Yes = 1,
    No = 2,
}

impl FromDataReader for MspaMode {
    type Err = io::Error;

    fn from_data_reader(r: &mut DataReader) -> Result<Self, Self::Err> {
        Ok(Self::from_u8(r.read_fixed_integer(2)?).unwrap_or(Self::NotApplicable))
    }
}

pub(crate) fn parse_mspa_covered_transaction(
    r: &mut DataReader,
) -> Result<bool, SectionDecodeError> {
    let val = r.read_fixed_integer(2)?;
    match val {
        1 => Ok(true),
        2 => Ok(false),
        v => Err(SectionDecodeError::InvalidFieldValue {
            expected: "1 or 2".to_string(),
            found: v.to_string(),
        }),
    }
}

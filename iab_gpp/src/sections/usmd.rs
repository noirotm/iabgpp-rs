use crate::sections::us_common::{Consent, Notice, OptOut};
use bitstream_io::{BitRead, FromBitStream};
use iab_gpp_derive::{FromBitStream, GPPSection};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::FromPrimitive;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::io;

#[derive(Debug, Eq, PartialEq, GPPSection)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[non_exhaustive]
#[gpp(with_header)]
pub struct UsMd {
    pub core: Core,
    #[gpp(optional_segment_type = 1)]
    pub gpc: Option<bool>,
}

#[derive(Debug, Eq, PartialEq, FromBitStream)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[non_exhaustive]
pub struct Core {
    pub mspa_version: u8,
    pub mspa_covered_transaction: bool,
    pub mspa_mode: MspaMode,
    pub processing_notice: Notice,
    pub sale_opt_out_notice: Notice,
    pub targeted_advertising_opt_out_notice: Notice,
    pub sale_opt_out: OptOut,
    pub targeted_advertising_opt_out: OptOut,
    pub additional_data_processing_consent: Consent,
}

#[derive(Debug, Eq, PartialEq, FromPrimitive, ToPrimitive)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum MspaMode {
    NotApplicable = 0,
    OptOutOption = 1,
    ServiceProvider = 2,
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

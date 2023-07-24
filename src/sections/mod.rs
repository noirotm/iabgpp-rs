use crate::sections::tcfeuv2::TcfEuV2;
use crate::sections::uspv1::UspV1;
use std::collections::BTreeSet;
use std::io;
use std::str::FromStr;
use thiserror::Error;

mod tcfeuv1;
mod tcfeuv2;
mod uspv1;

pub mod id {
    pub const TCF_EU_V1: u64 = 1;
    pub const TCF_EU_V2: u64 = 2;
    pub const TCF_CA: u64 = 5;
    pub const USP_V1: u64 = 6;
    pub const US_NAT: u64 = 7;
    pub const US_CA: u64 = 8;
    pub const US_VA: u64 = 9;
    pub const US_CO: u64 = 10;
    pub const US_UT: u64 = 11;
    pub const US_CT: u64 = 12;
}

pub type VendorList = BTreeSet<u16>;

#[derive(Error, Debug)]
pub enum SectionDecodeError {
    #[error("unable to read string")]
    Read(#[from] io::Error),
    #[error("unexpected end of string in {0}")]
    UnexpectedEndOfString(String),
    #[error("invalid character {character:?} in {kind} string {s:?}")]
    InvalidCharacter {
        character: char,
        kind: &'static str,
        s: String,
    },
    #[error("unable to decode segment")]
    DecodeSegment(#[from] base64::DecodeError),
    #[error("invalid segment version (expected {expected}, found {found})")]
    InvalidSegmentVersion { expected: u8, found: u8 },
    #[error("unknown segment type {segment_type}")]
    UnknownSegmentType { segment_type: u8 },
}

pub enum Section {
    TcfEuV1,
    TcfEuV2(TcfEuV2),
    TcfCa,
    UspV1(UspV1),
    UsNAT,
    UsCa,
    UsVa,
    UsCo,
    UsUt,
    UsCt,
    Unsupported(String),
}

pub(crate) fn decode_section(id: u64, s: &str) -> Result<Section, SectionDecodeError> {
    match id {
        id::TCF_EU_V2 => TcfEuV2::from_str(s).map(Section::TcfEuV2),
        id::USP_V1 => UspV1::from_str(s).map(Section::UspV1),

        _ => Ok(Section::Unsupported(s.to_string())),
    }
}

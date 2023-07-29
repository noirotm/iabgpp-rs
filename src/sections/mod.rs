use crate::sections::tcfeuv1::TcfEuV1;
use crate::sections::tcfeuv2::TcfEuV2;
use crate::sections::uspv1::UspV1;
use std::collections::BTreeSet;
use std::io;
use std::str::FromStr;
use thiserror::Error;

pub mod tcfeuv1;
pub mod tcfeuv2;
pub mod uspv1;

pub mod id {
    pub const TCF_EU_V1: u8 = 1;
    pub const TCF_EU_V2: u8 = 2;
    pub const GPP_HEADER: u8 = 3;
    pub const GPP_SIGNAL_INTEGRITY: u8 = 4;
    pub const TCF_CA: u8 = 5;
    pub const USP_V1: u8 = 6;
    pub const US_NAT: u8 = 7;
    pub const US_CA: u8 = 8;
    pub const US_VA: u8 = 9;
    pub const US_CO: u8 = 10;
    pub const US_UT: u8 = 11;
    pub const US_CT: u8 = 12;
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
    #[error("invalid section version (expected {expected}, found {found})")]
    InvalidSectionVersion { expected: u8, found: u8 },
    #[error("unable to decode segment")]
    DecodeSegment(#[from] base64::DecodeError),
    #[error("invalid segment version (expected {expected}, found {found})")]
    InvalidSegmentVersion { expected: u8, found: u8 },
    #[error("unknown segment type {segment_type}")]
    UnknownSegmentType { segment_type: u8 },
}

pub enum Section {
    TcfEuV1(TcfEuV1),
    TcfEuV2(TcfEuV2),
    TcfCa,
    UspV1(UspV1),
    UsNat,
    UsCa,
    UsVa,
    UsCo,
    UsUt,
    UsCt,
    Unsupported(String),
}

pub(crate) fn decode_section(id: u8, s: &str) -> Result<Section, SectionDecodeError> {
    match id {
        id::TCF_EU_V1 => TcfEuV1::from_str(s).map(Section::TcfEuV1),
        id::TCF_EU_V2 => TcfEuV2::from_str(s).map(Section::TcfEuV2),
        id::USP_V1 => UspV1::from_str(s).map(Section::UspV1),

        _ => Ok(Section::Unsupported(s.to_string())),
    }
}

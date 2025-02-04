//! Traits, helpers, and type definitions for working with GPP sections.
//!
//! All supported section IDs are listed in the [`SectionId`] and [`Section`] enums.
//!
//! Implementation of each section is done in its corresponding submodule.
//! Note that the GPP specification states that each section specification is supposed to be
//! independent. As a consequence, there is a lot of duplication between implementations of
//! these sections.
//!
//! A few sections are marked as deprecated in the official specification, and other sections
//! should be used in their place. They are still implemented in this crate and will stay here
//! for the time being.
//!
//! Similarly, if incompatible new versions of sections are added to the standard, they will
//! be added here, and the previous versions will remain.
//!
//! If new versions are backward compatible with older ones, new fields will be added for the
//! existing version (for example in TCF Canada v1.1 versus v1). For this reason, most of the
//! section types are marked with the `#[non_exhaustive]` attribute to preserve minor version
//! compatibility.
//!
use crate::core::base64::DecodeError;
use crate::core::{DataReader, DecodeExt, FromDataReader};
use crate::sections::tcfcav1::TcfCaV1;
use crate::sections::tcfeuv1::TcfEuV1;
use crate::sections::tcfeuv2::TcfEuV2;
use crate::sections::usca::UsCa;
use crate::sections::usco::UsCo;
use crate::sections::usct::UsCt;
use crate::sections::usde::UsDe;
use crate::sections::usfl::UsFl;
use crate::sections::usia::UsIa;
use crate::sections::usmt::UsMt;
use crate::sections::usnat::UsNat;
use crate::sections::usne::UsNe;
use crate::sections::usnh::UsNh;
use crate::sections::usnj::UsNj;
use crate::sections::usor::UsOr;
use crate::sections::uspv1::UspV1;
use crate::sections::ustn::UsTn;
use crate::sections::ustx::UsTx;
use crate::sections::usut::UsUt;
use crate::sections::usva::UsVa;
use num_derive::{FromPrimitive, ToPrimitive};
#[cfg(feature = "serde")]
use serde::Serialize;
use std::collections::BTreeSet;
use std::io;
use std::str::FromStr;
use strum_macros::Display;
use thiserror::Error;

pub mod tcfcav1;
pub mod tcfeuv1;
pub mod tcfeuv2;
pub mod us_common;
pub mod usca;
pub mod usco;
pub mod usct;
pub mod usde;
pub mod usfl;
pub mod usia;
pub mod usmt;
pub mod usnat;
pub mod usne;
pub mod usnh;
pub mod usnj;
pub mod usor;
pub mod uspv1;
pub mod ustn;
pub mod ustx;
pub mod usut;
pub mod usva;

#[derive(Clone, Copy, Debug, Display, Eq, PartialEq, Hash, FromPrimitive, ToPrimitive)]
#[non_exhaustive]
pub enum SectionId {
    TcfEuV1 = 1,
    TcfEuV2 = 2,
    GppHeader = 3,
    GppSignalIntegrity = 4,
    TcfCaV1 = 5,
    UspV1 = 6,
    UsNat = 7,
    UsCa = 8,
    UsVa = 9,
    UsCo = 10,
    UsUt = 11,
    UsCt = 12,
    UsFl = 13,
    UsMt = 14,
    UsOr = 15,
    UsTx = 16,
    UsDe = 17,
    UsIa = 18,
    UsNe = 19,
    UsNh = 20,
    UsNj = 21,
    UsTn = 22,
}

pub trait DecodableSection: FromStr<Err = SectionDecodeError> {
    const ID: SectionId;
}

pub type IdSet = BTreeSet<u16>;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum SectionDecodeError {
    #[error("missing section {0}")]
    MissingSection(SectionId),
    #[error("unsupported section id {0}")]
    UnsupportedSectionId(SectionId),
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
    DecodeSegment(#[from] DecodeError),
    #[error("invalid segment version ({segment_version})")]
    UnknownSegmentVersion { segment_version: u8 },
    #[error("unknown segment type {segment_type}")]
    UnknownSegmentType { segment_type: u8 },
    #[error("duplicate segment type {segment_type}")]
    DuplicateSegmentType { segment_type: u8 },
    #[error("invalid field value (expected {expected}, found {found})")]
    InvalidFieldValue { expected: String, found: String },
}

#[derive(Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[non_exhaustive]
pub enum Section {
    TcfEuV1(TcfEuV1),
    TcfEuV2(TcfEuV2),
    TcfCaV1(TcfCaV1),
    UspV1(UspV1),
    UsNat(UsNat),
    UsCa(UsCa),
    UsVa(UsVa),
    UsCo(UsCo),
    UsUt(UsUt),
    UsCt(UsCt),
    UsFl(UsFl),
    UsMt(UsMt),
    UsOr(UsOr),
    UsTx(UsTx),
    UsDe(UsDe),
    UsIa(UsIa),
    UsNe(UsNe),
    UsNh(UsNh),
    UsNj(UsNj),
    UsTn(UsTn),
}

impl Section {
    pub fn id(&self) -> SectionId {
        match self {
            Section::TcfEuV1(_) => SectionId::TcfEuV1,
            Section::TcfEuV2(_) => SectionId::TcfEuV2,
            Section::TcfCaV1(_) => SectionId::TcfCaV1,
            Section::UspV1(_) => SectionId::UspV1,
            Section::UsNat(_) => SectionId::UsNat,
            Section::UsCa(_) => SectionId::UsCa,
            Section::UsVa(_) => SectionId::UsVa,
            Section::UsCo(_) => SectionId::UsCo,
            Section::UsUt(_) => SectionId::UsUt,
            Section::UsCt(_) => SectionId::UsCt,
            Section::UsFl(_) => SectionId::UsFl,
            Section::UsMt(_) => SectionId::UsMt,
            Section::UsOr(_) => SectionId::UsOr,
            Section::UsTx(_) => SectionId::UsTx,
            Section::UsDe(_) => SectionId::UsDe,
            Section::UsIa(_) => SectionId::UsIa,
            Section::UsNe(_) => SectionId::UsNe,
            Section::UsNh(_) => SectionId::UsNh,
            Section::UsNj(_) => SectionId::UsNj,
            Section::UsTn(_) => SectionId::UsTn,
        }
    }
}

pub(crate) fn decode_section(id: SectionId, s: &str) -> Result<Section, SectionDecodeError> {
    Ok(match id {
        SectionId::TcfEuV1 => Section::TcfEuV1(s.parse()?),
        SectionId::TcfEuV2 => Section::TcfEuV2(s.parse()?),
        SectionId::TcfCaV1 => Section::TcfCaV1(s.parse()?),
        SectionId::UspV1 => Section::UspV1(s.parse()?),
        SectionId::UsNat => Section::UsNat(s.parse()?),
        SectionId::UsCa => Section::UsCa(s.parse()?),
        SectionId::UsVa => Section::UsVa(s.parse()?),
        SectionId::UsCo => Section::UsCo(s.parse()?),
        SectionId::UsUt => Section::UsUt(s.parse()?),
        SectionId::UsCt => Section::UsCt(s.parse()?),
        SectionId::UsFl => Section::UsFl(s.parse()?),
        SectionId::UsMt => Section::UsMt(s.parse()?),
        SectionId::UsOr => Section::UsOr(s.parse()?),
        SectionId::UsTx => Section::UsTx(s.parse()?),
        SectionId::UsDe => Section::UsDe(s.parse()?),
        SectionId::UsIa => Section::UsIa(s.parse()?),
        SectionId::UsNe => Section::UsNe(s.parse()?),
        SectionId::UsNh => Section::UsNh(s.parse()?),
        SectionId::UsNj => Section::UsNj(s.parse()?),
        SectionId::UsTn => Section::UsTn(s.parse()?),
        id => Err(SectionDecodeError::UnsupportedSectionId(id))?,
    })
}

pub(crate) trait Base64EncodedStr<T> {
    fn parse_base64_str(&self) -> Result<T, SectionDecodeError>;
}

impl<T> Base64EncodedStr<T> for str
where
    T: FromDataReader<Err = SectionDecodeError>,
{
    fn parse_base64_str(&self) -> Result<T, SectionDecodeError> {
        let r = self.decode_base64_url()?;
        DataReader::new(&r).parse()
    }
}

/// A trait representing an operation to parse segments for a Base64-URL encoded string
/// using '.' as separators into a type composed of a mandatory core segment and an arbitrary
/// number of optional segments.
///
/// This guarantees a given segment cannot appear twice.
pub(crate) trait SegmentedStr<T> {
    fn parse_segmented_str(&self) -> Result<T, SectionDecodeError>;
}

impl<T> SegmentedStr<T> for str
where
    T: OptionalSegmentParser,
{
    fn parse_segmented_str(&self) -> Result<T, SectionDecodeError> {
        let mut sections_iter = self.split('.');

        // first mandatory section is the core segment
        let core = sections_iter
            .next()
            .ok_or_else(|| SectionDecodeError::UnexpectedEndOfString(self.to_string()))?
            .decode_base64_url()?;
        let mut r = DataReader::new(&core);
        let mut output = r.parse()?;
        let mut segments = BTreeSet::new();

        // parse each optional segment and fill the output
        for s in sections_iter {
            let b = s.decode_base64_url()?;
            let mut r = DataReader::new(&b);

            let segment_type = T::read_segment_type(&mut r)?;
            T::parse_optional_segment(segment_type, &mut r, &mut output)?;

            // already present, duplicate segments is an error
            if !segments.insert(segment_type) {
                return Err(SectionDecodeError::DuplicateSegmentType { segment_type });
            }
        }

        Ok(output)
    }
}

/// A trait representing an operation to parse optional segments for a Base64-URL encoded string
pub(crate) trait OptionalSegmentParser:
    Sized + FromDataReader<Err = SectionDecodeError>
{
    fn read_segment_type(r: &mut DataReader) -> Result<u8, SectionDecodeError> {
        Ok(r.read_fixed_integer(3)?)
    }

    fn parse_optional_segment(
        segment_type: u8,
        r: &mut DataReader,
        into: &mut Self,
    ) -> Result<(), SectionDecodeError>;
}

use crate::core::base64::DecodeError;
use crate::core::{DataReader, DecodeExt, FromDataReader};
use crate::sections::tcfcav1::TcfCaV1;
use crate::sections::tcfeuv1::TcfEuV1;
use crate::sections::tcfeuv2::TcfEuV2;
use crate::sections::usca::UsCa;
use crate::sections::usco::UsCo;
use crate::sections::usct::UsCt;
use crate::sections::usnat::UsNat;
use crate::sections::uspv1::UspV1;
use crate::sections::usut::UsUt;
use crate::sections::usva::UsVa;
use num_derive::{FromPrimitive, ToPrimitive};
use std::collections::BTreeSet;
use std::io;
use std::str::FromStr;
use strum_macros::Display;
use thiserror::Error;

pub mod tcfcav1;
pub mod tcfeuv1;
pub mod tcfeuv2;
pub mod usca;
pub mod usco;
pub mod usct;
pub mod usnat;
pub mod uspv1;
pub mod usut;
pub mod usva;

#[derive(Clone, Copy, Debug, Display, Eq, PartialEq, Hash, FromPrimitive, ToPrimitive)]
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
}

pub trait DecodableSection: FromStr<Err = SectionDecodeError> {
    const ID: SectionId;
}

pub type IdSet = BTreeSet<u16>;

#[derive(Error, Debug)]
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
    #[error("invalid segment version (expected {expected}, found {found})")]
    InvalidSegmentVersion { expected: u8, found: u8 },
    #[error("unknown segment type {segment_type}")]
    UnknownSegmentType { segment_type: u8 },
    #[error("duplicate segment type {segment_type}")]
    DuplicateSegmentType { segment_type: u8 },
    #[error("missing core segment")]
    MissingCoreSegment,
    #[error("invalid field value (expected {expected}, found {found})")]
    InvalidFieldValue { expected: String, found: String },
}

#[derive(Debug)]
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
/// using '.' as separators into a type composed of a mandatory code segment and an arbitrary
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

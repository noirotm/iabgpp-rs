use crate::core::{DataReader, DecodeExt, FromDataReader};
use crate::sections::tcfcav1::TcfCaV1;
use crate::sections::tcfeuv1::TcfEuV1;
use crate::sections::tcfeuv2::TcfEuV2;
use crate::sections::uspv1::UspV1;
use std::collections::BTreeSet;
use std::io;
use thiserror::Error;

pub mod tcfcav1;
pub mod tcfeuv1;
pub mod tcfeuv2;
pub mod uspv1;

pub mod id {
    pub const TCF_EU_V1: u8 = 1;
    pub const TCF_EU_V2: u8 = 2;
    pub const GPP_HEADER: u8 = 3;
    pub const GPP_SIGNAL_INTEGRITY: u8 = 4;
    pub const TCF_CA_V1: u8 = 5;
    pub const USP_V1: u8 = 6;
    pub const US_NAT: u8 = 7;
    pub const US_CA: u8 = 8;
    pub const US_VA: u8 = 9;
    pub const US_CO: u8 = 10;
    pub const US_UT: u8 = 11;
    pub const US_CT: u8 = 12;
}

pub type IdList = BTreeSet<u16>;

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
    #[error("duplicate segment type {segment_type}")]
    DuplicateSegmentType { segment_type: u8 },
    #[error("missing core segment")]
    MissingCoreSegment,
}

pub enum Section {
    TcfEuV1(TcfEuV1),
    TcfEuV2(TcfEuV2),
    TcfCaV1(TcfCaV1),
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
    Ok(match id {
        id::TCF_EU_V1 => Section::TcfEuV1(s.parse()?),
        id::TCF_EU_V2 => Section::TcfEuV2(s.parse()?),
        id::TCF_CA_V1 => Section::TcfCaV1(s.parse()?),
        id::USP_V1 => Section::UspV1(s.parse()?),
        _ => Section::Unsupported(s.to_string()),
    })
}

/// A trait representing an operation to parse segments for a Base64-URL encoded string
/// using '.' as separators into a typed composed of a mandatory code segment and an arbitrary
/// number of optional segments.
///
/// This guarantees a given segment cannot appear twice.
pub(crate) trait OptionalSegmentParser:
    Sized + FromDataReader<Err = SectionDecodeError>
{
    fn parse_optional_segment(
        segment_type: u8,
        r: &mut DataReader,
        into: &mut Self,
    ) -> Result<(), SectionDecodeError>;
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
            let s = s.decode_base64_url()?;
            let mut r = DataReader::new(&s);

            let segment_type = r.read_fixed_integer::<u8>(3)?;
            T::parse_optional_segment(segment_type, &mut r, &mut output)?;

            // already present, duplicate segments is an error
            if !segments.insert(segment_type) {
                return Err(SectionDecodeError::DuplicateSegmentType { segment_type });
            }
        }

        Ok(output)
    }
}

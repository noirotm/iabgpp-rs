//! Version 1 of the IAB Global Privacy Platform string.
//!
//! A GPP string contains a header which lists the sections which are present
//! in the next optional parts.
//!
//! A typical GPP string will look like this:
//!
//! ```text
//! DBACNY~CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA~1YNN
//! ```
//!
//! It contains a header (`DBACNY`) and two sections separated by a `~` character.
//!
//! GPP string sections are usually encoded in a variation of URL-safe Base64.
//!
//! It is not mandatory though, and certain sections, such as the deprecated USP v1 are using
//! a simpler character set.
//! In the example above, the first section is a base64 encoded TCF EU v2.2 section.
//! The second section is a USP v1 section where `Y` and `N` characters simply mean yes and no
//! respectively.
//!
//! In order to obtain a [`GPPString`] instance, several ways are possible.
//!
//! # Examples
//!
//! You can use the [`FromStr`] trait directly to try to parse a consent string:
//!
//! ```
//! # use std::error::Error;
//! #
//! # fn main() -> Result<(), Box<dyn Error>> {
//! use std::str::FromStr;
//! use iab_gpp::v1::GPPString;
//!
//! let s = GPPString::from_str("DBABTA~1YNN")?;
//! # Ok(())
//! # }
//! ```
//!
//! You can also use [`str::parse`]:
//!
//! ```
//! # use std::error::Error;
//! #
//! # fn main() -> Result<(), Box<dyn Error>> {
//! use iab_gpp::v1::GPPString;
//!
//! let s: GPPString = "DBABTA~1YNN".parse()?;
//! # Ok(())
//! # }
//! ```
//!
//! If parsing fails, a [`GPPDecodeError`] will be returned instead.
//!
use crate::core::base64::DecodeError;
use crate::core::{DataReader, DecodeExt};
use crate::sections::{decode_section, DecodableSection, Section, SectionDecodeError, SectionId};
use fnv::FnvHashMap;
use num_traits::FromPrimitive;
use std::io;
use std::iter::FusedIterator;
use std::slice::Iter;
use std::str::FromStr;
use thiserror::Error;

const GPP_HEADER: u8 = 3;
const GPP_VERSION: u8 = 1;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum GPPDecodeError {
    #[error("no header found")]
    NoHeaderFound,
    #[error("unable to decode header")]
    DecodeHeader(#[from] DecodeError),
    #[error("invalid header type (expected {GPP_HEADER}, found {found})")]
    InvalidHeaderType { found: u8 },
    #[error("invalid GPP version (expected {GPP_VERSION}, found {found})")]
    InvalidGPPVersion { found: u8 },
    #[error("unable to read string")]
    Read(#[from] io::Error),
    #[error("unsupported section id {0}")]
    UnsupportedSectionId(u8),
    #[error("ids do not match sections (number of ids {ids}, number of sections {sections}")]
    IdSectionMismatch { ids: usize, sections: usize },
    #[error("unable to decode section")]
    SectionDecode(#[from] SectionDecodeError),
}

/// The representation of a parsed GPP consent string.
///
/// This structure gives access to the list of section IDs which it contains, as well as the raw
/// section strings.
///
/// It also offers methods to decode either a specific section, or all sections at once.
///
#[derive(Debug)]
pub struct GPPString {
    section_ids: Vec<SectionId>,
    sections: FnvHashMap<SectionId, String>,
}

impl GPPString {
    /// Returns a reference to a raw section contained in this GPP string.
    ///
    /// The method takes the section ID as parameter, and returns the reference
    /// to the raw string representing that section.
    ///
    /// If the given section is not present within the GPP string, the method returns None.
    ///
    /// # Example
    ///
    /// ```
    ///
    /// ```
    pub fn section(&self, id: SectionId) -> Option<&str> {
        self.sections.get(&id).map(|s| s.as_str())
    }

    /// Returns the list of section IDs present in this GPP string.
    ///
    /// The list is returned as a slice.
    pub fn section_ids(&self) -> SectionIds {
        SectionIds(self.section_ids.iter())
    }

    pub fn sections(&self) -> Sections {
        Sections {
            gpp_str: self,
            idx: 0,
        }
    }

    pub fn decode_section(&self, id: SectionId) -> Result<Section, SectionDecodeError> {
        let s = self
            .section(id)
            .ok_or(SectionDecodeError::MissingSection(id))?;
        decode_section(id, s)
    }

    pub fn decode<T>(&self) -> Result<T, SectionDecodeError>
    where
        T: DecodableSection,
    {
        self.section(T::ID)
            .ok_or(SectionDecodeError::MissingSection(T::ID))?
            .parse()
    }

    pub fn decode_all_sections(&self) -> Vec<Result<Section, SectionDecodeError>> {
        self.section_ids
            .iter()
            .map(|id| self.decode_section(*id))
            .collect()
    }
}

impl FromStr for GPPString {
    type Err = GPPDecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (section_ids, sections) = extract_gpp_sections_from_str(s)?;

        let sections = section_ids
            .iter()
            .zip(sections)
            .map(|(&id, s)| (id, s.to_string()))
            .collect();

        Ok(Self {
            section_ids,
            sections,
        })
    }
}

fn extract_gpp_sections_from_str(s: &str) -> Result<(Vec<SectionId>, Vec<&str>), GPPDecodeError> {
    let mut sections_iter = s.split('~');

    let header_str = sections_iter.next().ok_or(GPPDecodeError::NoHeaderFound)?;
    let header = header_str.decode_base64_url()?;
    let mut reader = DataReader::new(&header);

    let header_type = reader.read_fixed_integer(6)?;
    if header_type != GPP_HEADER {
        return Err(GPPDecodeError::InvalidHeaderType { found: header_type });
    }

    let gpp_version = reader.read_fixed_integer(6)?;
    if gpp_version != GPP_VERSION {
        return Err(GPPDecodeError::InvalidGPPVersion { found: gpp_version });
    }

    let section_ids = reader
        .read_fibonacci_range()?
        .into_iter()
        .map(|id| SectionId::from_u8(id).ok_or(GPPDecodeError::UnsupportedSectionId(id)))
        .collect::<Result<Vec<_>, _>>()?;

    let sections = sections_iter.collect::<Vec<_>>();
    if sections.len() != section_ids.len() {
        return Err(GPPDecodeError::IdSectionMismatch {
            ids: section_ids.len(),
            sections: sections.len(),
        });
    }

    Ok((section_ids, sections))
}

pub struct Sections<'a> {
    gpp_str: &'a GPPString,
    idx: usize,
}

impl<'a> Iterator for Sections<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let section_id = self.gpp_str.section_ids.get(self.idx)?;
        self.idx += 1;
        self.gpp_str.section(*section_id)
    }
}

impl<'a> ExactSizeIterator for Sections<'a> {
    fn len(&self) -> usize {
        self.gpp_str.section_ids.len()
    }
}

impl<'a> FusedIterator for Sections<'a> {}

pub struct SectionIds<'a>(Iter<'a, SectionId>);

impl<'a> Iterator for SectionIds<'a> {
    type Item = &'a SectionId;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<'a> ExactSizeIterator for SectionIds<'a> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a> FusedIterator for SectionIds<'a> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sections::uspv1::UspV1;
    use test_case::test_case;

    #[test_case("DBABjw~BPXuQIAPXuQIAAfKABENB-CgAAAAAAAAAAAAAAAA.YAAAAAAAAAA~1YNN", SectionId::TcfCaV1 => Some("BPXuQIAPXuQIAAfKABENB-CgAAAAAAAAAAAAAAAA.YAAAAAAAAAA".to_string()) ; "tcf ca")]
    #[test_case("DBABjw~BPXuQIAPXuQIAAfKABENB-CgAAAAAAAAAAAAAAAA.YAAAAAAAAAA~1YNN", SectionId::UspV1 => Some("1YNN".to_string()) ; "usp v1")]
    #[test_case("DBABjw~BPXuQIAPXuQIAAfKABENB-CgAAAAAAAAAAAAAAAA.YAAAAAAAAAA~1YNN", SectionId::TcfEuV2 => None ; "tcf eu v2")]
    fn gpp_string_section(s: &str, section_id: SectionId) -> Option<String> {
        GPPString::from_str(s)
            .unwrap()
            .section(section_id)
            .map(|s| s.to_string())
    }

    #[test_case("DBABM~CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA" => vec![SectionId::TcfEuV2] ; "single section")]
    #[test_case("DBACNY~CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA~1YNN" => vec![SectionId::TcfEuV2, SectionId::UspV1] ; "tcf eu and us sections")]
    #[test_case("DBABjw~BPXuQIAPXuQIAAfKABENB-CgAAAAAAAAAAAAAAAA.YAAAAAAAAAA~1YNN" => vec![SectionId::TcfCaV1, SectionId::UspV1] ; "tcf ca and us sections")]
    fn gpp_string_section_ids(s: &str) -> Vec<SectionId> {
        GPPString::from_str(s).unwrap().section_ids
    }

    #[test_case("DBABM~CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA" => vec!["CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA"] ; "single section")]
    #[test_case("DBACNY~CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA~1YNN" => vec!["CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA", "1YNN"] ; "tcf eu and us sections")]
    #[test_case("DBABjw~BPXuQIAPXuQIAAfKABENB-CgAAAAAAAAAAAAAAAA.YAAAAAAAAAA~1YNN" => vec!["BPXuQIAPXuQIAAfKABENB-CgAAAAAAAAAAAAAAAA.YAAAAAAAAAA", "1YNN"] ; "tcf ca and us sections")]
    fn gpp_string_sections(s: &str) -> Vec<String> {
        GPPString::from_str(s)
            .unwrap()
            .sections()
            .map(|s| s.to_string())
            .collect()
    }

    #[test_case("DBABM~CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA" => vec![SectionId::TcfEuV2] ; "single section")]
    #[test_case("DBACNY~CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA~1YNN" => vec![SectionId::TcfEuV2, SectionId::UspV1] ; "tcf eu and us sections")]
    #[test_case("DBABjw~BPXuQIAPXuQIAAfKABENB-CgAAAAAAAAAAAAAAAA.YAAAAAAAAAA~1YNN" => vec![SectionId::TcfCaV1, SectionId::UspV1] ; "tcf ca and us sections")]
    fn gpp_string_decode_section(s: &str) -> Vec<SectionId> {
        let s = GPPString::from_str(s).unwrap();
        s.section_ids
            .iter()
            .map(|id| s.decode_section(*id).unwrap().id())
            .collect()
    }

    #[test_case("DBABM~CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA" => vec![SectionId::TcfEuV2] ; "single section")]
    #[test_case("DBACNY~CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA~1YNN" => vec![SectionId::TcfEuV2, SectionId::UspV1] ; "tcf eu and us sections")]
    #[test_case("DBABjw~BPXuQIAPXuQIAAfKABENB-CgAAAAAAAAAAAAAAAA.YAAAAAAAAAA~1YNN" => vec![SectionId::TcfCaV1, SectionId::UspV1] ; "tcf ca and us sections")]
    fn gpp_string_decode_all(s: &str) -> Vec<SectionId> {
        GPPString::from_str(s)
            .unwrap()
            .decode_all_sections()
            .into_iter()
            .map(|s| s.unwrap().id())
            .collect()
    }

    #[test_case("DBABTA~1YN-" => UspV1 {
        opt_out_notice: crate::sections::uspv1::Notice::Yes,
        opt_out_sale: crate::sections::uspv1::OptOut::No,
        lspa_covered: crate::sections::uspv1::Covered::NotApplicable,
    } ; "mix")]
    #[test_case("DBABTA~1NNN" => UspV1 {
        opt_out_notice: crate::sections::uspv1::Notice::No,
        opt_out_sale: crate::sections::uspv1::OptOut::No,
        lspa_covered: crate::sections::uspv1::Covered::No,
    } ; "all no")]
    #[test_case("DBABTA~1YYY" => UspV1 {
        opt_out_notice: crate::sections::uspv1::Notice::Yes,
        opt_out_sale: crate::sections::uspv1::OptOut::Yes,
        lspa_covered: crate::sections::uspv1::Covered::Yes,
    } ; "all yes")]
    #[test_case("DBACNY~CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA~1YNN" => UspV1 {
        opt_out_notice: crate::sections::uspv1::Notice::Yes,
        opt_out_sale: crate::sections::uspv1::OptOut::No,
        lspa_covered: crate::sections::uspv1::Covered::No,
    } ; "with other section")]
    fn gpp_string_decode_uspv1(s: &str) -> UspV1 {
        GPPString::from_str(s).unwrap().decode().unwrap()
    }

    #[test]
    fn truncated_string() {
        let r = GPPString::from_str("DBACNY~CPytTYAPytTYABEACBENDXCoAP_AAH_AAAIwgoNf_X__b3_v-_7___t0eY1f9_7__-0zjhfdt-8N3f_X_L8X_2M7");
        assert!(matches!(
            r,
            Err(GPPDecodeError::IdSectionMismatch {
                ids: 2,
                sections: 1
            })
        ));
    }

    #[test]
    fn non_gpp_tcfeuv2_string() {
        let r = GPPString::from_str("CP48G0AP48G0AEsACCPLAkEgAAAAAEPgAB5YAAAQaQD2F2K2kKFkPCmQWYAQBCijYEAhQAAAAkCBIAAgAUgQAgFIIAgAIFAAAAAAAAAQEgCQAAQABAAAIACgAAAAAAIAAAAAAAQQAAAAAIAAAAAAAAEAAAAAAAQAAAAIAABEhCAAQQAEAAAAAAAQAAAAAAAAAAABAAAAAAAAAAAAAAAAAAAAgAA");
        assert!(matches!(
            r,
            Err(GPPDecodeError::InvalidHeaderType { found: 2 })
        ));
    }

    #[test]
    fn invalid_tcfca_section() {
        let r = GPPString::from_str("DBABjw~CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA~1YNN")
            .unwrap()
            .decode_all_sections();
        assert!(matches!(
            r[0],
            Err(SectionDecodeError::InvalidSegmentVersion {
                expected: 1,
                found: 2,
            })
        ));
    }
}

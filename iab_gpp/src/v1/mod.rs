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
//! You can use the [`GPPString::parse_str`] method to try to parse a consent string:
//!
//! ```
//! use iab_gpp::v1::GPPString;
//! use iab_gpp::v1::GPPDecodeError;
//!
//! fn main() -> Result<(), GPPDecodeError> {
//!     let s = GPPString::parse_str("DBABTA~1YNN")?;
//!     Ok(())
//! }
//! ```
//! Since [`GPPString`] implements the [`FromStr`] trait, you can use it directly to try to parse
//! a consent string:
//!
//! ```
//! use iab_gpp::v1::GPPString;
//! use iab_gpp::v1::GPPDecodeError;
//! use std::str::FromStr;
//!
//! fn main() -> Result<(), GPPDecodeError> {
//!     let s = GPPString::from_str("DBABTA~1YNN")?;
//!     Ok(())
//! }
//! ```
//!
//! You can also use [`str::parse`]:
//!
//! ```
//! use iab_gpp::v1::GPPString;
//! use iab_gpp::v1::GPPDecodeError;
//!
//! fn main() -> Result<(), GPPDecodeError> {
//!     let s: GPPString = "DBABTA~1YNN".parse()?;
//!     Ok(())
//! }
//! ```
//!
//! If parsing fails, a [`GPPDecodeError`] is returned instead.
//!
use crate::core::{base64_bit_reader, DataRead};
use crate::sections::{decode_section, DecodableSection, Section, SectionDecodeError, SectionId};
use bitstream_io::BitRead;
use fnv::FnvHashMap;
use num_traits::FromPrimitive;
use std::io;
use std::iter::FusedIterator;
use std::slice::Iter;
use std::str::FromStr;
use thiserror::Error;

const GPP_HEADER: u8 = 3;
const GPP_VERSION: u8 = 1;

/// The error type for GPP String decoding operations.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum GPPDecodeError {
    /// The string does not contain the mandatory header section.
    #[error("no header found")]
    NoHeaderFound,
    /// The header has an invalid type for this version of GPP.
    #[error("invalid header type (expected {GPP_HEADER}, found {found})")]
    InvalidHeaderType { found: u8 },
    /// The header has an invalid GPP version.
    ///
    /// Note that there is currently only V1 of the standard.
    /// If new versions are released, they will be implemented in other modules.
    #[error("invalid GPP version (expected {GPP_VERSION}, found {found})")]
    InvalidGPPVersion { found: u8 },
    /// An I/O error occured while reading the string.
    ///
    /// This usually occurs if the input string is truncated.
    #[error("unable to read string: {source}")]
    Read {
        #[from]
        source: io::Error,
    },
    /// A section with an unknown or unsupported identifier is listed in the string header.
    #[error("unsupported section id {0}")]
    UnsupportedSectionId(u8),
    /// The number of sections listed in the header does not match the number of actual sections
    /// present in the string.
    #[error("ids do not match sections (number of ids {ids}, number of sections {sections}")]
    IdSectionMismatch { ids: usize, sections: usize },
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
    /// Parses a string and returns a [`GPPString`] if successful.
    ///
    /// # Errors
    ///
    /// Returns a [`GPPDecodeError`] if unable to parse the string.
    ///
    /// # Example
    ///
    /// ```
    /// use iab_gpp::v1::GPPString;
    /// use iab_gpp::v1::GPPDecodeError;
    ///
    /// let r = GPPString::parse_str("DBABTA~1YNN");
    ///
    /// assert!(matches!(r, Ok(gpp_str)));
    /// ```
    ///
    pub fn parse_str(s: &str) -> Result<Self, GPPDecodeError> {
        s.parse()
    }

    /// Returns a reference to a raw section contained in this GPP string.
    ///
    /// The method takes the section ID as parameter, and returns the reference
    /// to the raw string representing that section.
    ///
    /// If the given section is not present within the GPP string, the method returns [`None`].
    ///
    /// # Example
    ///
    /// ```
    /// use std::str::FromStr;
    /// use iab_gpp::sections::SectionId;
    /// use iab_gpp::v1::GPPString;
    /// use iab_gpp::v1::GPPDecodeError;
    ///
    /// fn main() -> Result<(), GPPDecodeError> {
    ///     let gpp_str = GPPString::from_str("DBABTA~1YNN")?;
    ///     let s = gpp_str.section(SectionId::UspV1);
    ///
    ///     assert_eq!(s, Some("1YNN"));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn section(&self, id: SectionId) -> Option<&str> {
        self.sections.get(&id).map(|s| s.as_str())
    }

    /// Returns an iterator that yields the list of section IDs present in this GPP string.
    ///
    /// # Example
    ///
    /// ```
    /// use iab_gpp::sections::SectionId;
    /// use iab_gpp::v1::GPPString;
    /// use iab_gpp::v1::GPPDecodeError;
    ///
    /// fn main() -> Result<(), GPPDecodeError> {
    ///     let gpp_str = GPPString::parse_str("DBABTA~1YNN")?;
    ///     let mut it = gpp_str.section_ids();
    ///
    ///     assert_eq!(it.next(), Some(&SectionId::UspV1));
    ///     assert_eq!(it.next(), None);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn section_ids(&self) -> SectionIds<'_> {
        SectionIds(self.section_ids.iter())
    }

    /// Returns an iterator that yields the list of raw section strings present in this GPP string.
    ///
    /// # Example
    ///
    /// ```
    /// use iab_gpp::sections::SectionId;
    /// use iab_gpp::v1::GPPString;
    /// use iab_gpp::v1::GPPDecodeError;
    ///
    /// fn main() -> Result<(), GPPDecodeError> {
    ///     let gpp_str = GPPString::parse_str("DBABTA~1YNN")?;
    ///     let mut it = gpp_str.sections();
    ///
    ///     assert_eq!(it.next(), Some("1YNN"));
    ///     assert_eq!(it.next(), None);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn sections(&self) -> Sections<'_> {
        Sections {
            gpp_str: self,
            idx: 0,
        }
    }

    /// Decodes and returns a single section of this GPP string.
    ///
    /// Takes the section ID to decode as parameter.
    ///
    /// The returned section is wrapped in a [`Section`] enum, meaning that it must be
    /// explicitly matched. Therefore, this method is better used in loops, or when
    /// the type of section to parse is not known by advance.
    ///
    /// If you know by advance which section type you want to decode, use the generic
    /// [`decode`](GPPString::decode) method instead.
    ///
    /// # Example
    ///
    /// ```
    /// use iab_gpp::sections::SectionId;
    /// use iab_gpp::sections::Section;
    /// use iab_gpp::v1::GPPString;
    /// use iab_gpp::v1::GPPDecodeError;
    ///
    /// fn main() -> Result<(), GPPDecodeError> {
    ///     let gpp_str = GPPString::parse_str("DBABTA~1YNN")?;
    ///     let r = gpp_str.decode_section(SectionId::UspV1);
    ///
    ///     assert!(matches!(r, Ok(Section::UspV1(_))));
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`SectionDecodeError`] if decoding the section fails or if the section is not
    /// present in the string.
    ///
    pub fn decode_section(&self, id: SectionId) -> Result<Section, SectionDecodeError> {
        let s = self
            .section(id)
            .ok_or(SectionDecodeError::MissingSection(id))?;
        decode_section(id, s)
    }

    /// Decodes and returns a single section of this GPP string.
    ///
    /// Takes the section to return as a type parameter.
    ///
    /// As opposed to [`decode_section`](GPPString::decode_section), the returned section is
    /// returned directly. This is the easiest method to use if you know which section you expect to
    /// be present in the string.
    ///
    /// # Example
    ///
    /// ```
    /// use iab_gpp::sections::SectionId;
    /// use iab_gpp::sections::Section;
    /// use iab_gpp::sections::uspv1::UspV1;
    /// use iab_gpp::v1::GPPString;
    /// use iab_gpp::v1::GPPDecodeError;
    ///
    /// fn main() -> Result<(), GPPDecodeError> {
    ///     let gpp_str = GPPString::parse_str("DBABTA~1YNN")?;
    ///     let r = gpp_str.decode::<UspV1>();
    ///
    ///     assert!(matches!(r, Ok(UspV1{ .. })));
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`SectionDecodeError`] if decoding the section fails or if the section is not
    /// present in the string.
    ///
    pub fn decode<T>(&self) -> Result<T, SectionDecodeError>
    where
        T: DecodableSection,
    {
        self.section(T::ID)
            .ok_or(SectionDecodeError::MissingSection(T::ID))?
            .parse()
    }

    /// Decodes and returns all sections present in this GPP string.
    ///
    /// This is a convenience method which tries to decode all sections, and returns them
    /// in a [Vec] where each entry is either the decoded section or an error if decoding fails.
    ///
    /// # Example
    ///
    /// ```
    /// use iab_gpp::v1::GPPDecodeError;
    /// use iab_gpp::v1::GPPString;
    ///     
    /// fn main() -> Result<(), GPPDecodeError> {
    ///     let s = "DBACNY~CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA~1YNN";
    ///     let gpp_string = GPPString::parse_str(s)?;
    ///
    ///     for r in gpp_string.decode_all_sections() {
    ///         assert!(matches!(r, Ok(_)));
    ///         println!("Section: {:?}", &r);
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a [`SectionDecodeError`] for each section which fails to decode.
    ///
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
    let mut bit_reader = base64_bit_reader(header_str.as_bytes());

    let header_type = bit_reader.read_unsigned::<6, u8>()?;
    if header_type != GPP_HEADER {
        return Err(GPPDecodeError::InvalidHeaderType { found: header_type });
    }

    let gpp_version = bit_reader.read_unsigned::<6, u8>()?;
    if gpp_version != GPP_VERSION {
        return Err(GPPDecodeError::InvalidGPPVersion { found: gpp_version });
    }

    let section_ids = bit_reader
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

/// Created with the method [`sections`](GPPString::sections).
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

/// Created with the method [`section_ids`](GPPString::section_ids).
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
        opt_out_notice: crate::sections::uspv1::Flag::Yes,
        opt_out_sale: crate::sections::uspv1::Flag::No,
        lspa_covered_transaction: crate::sections::uspv1::Flag::NotApplicable,
    } ; "mix")]
    #[test_case("DBABTA~1NNN" => UspV1 {
        opt_out_notice: crate::sections::uspv1::Flag::No,
        opt_out_sale: crate::sections::uspv1::Flag::No,
        lspa_covered_transaction: crate::sections::uspv1::Flag::No,
    } ; "all no")]
    #[test_case("DBABTA~1YYY" => UspV1 {
        opt_out_notice: crate::sections::uspv1::Flag::Yes,
        opt_out_sale: crate::sections::uspv1::Flag::Yes,
        lspa_covered_transaction: crate::sections::uspv1::Flag::Yes,
    } ; "all yes")]
    #[test_case("DBACNY~CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA~1YNN" => UspV1 {
        opt_out_notice: crate::sections::uspv1::Flag::Yes,
        opt_out_sale: crate::sections::uspv1::Flag::No,
        lspa_covered_transaction: crate::sections::uspv1::Flag::No,
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
            Err(SectionDecodeError::UnknownSegmentVersion { segment_version: 2 })
        ));
    }

    #[test]
    fn invalid_tcfeuv2_section() {
        let r = GPPString::from_str("DBABMA~CQLvHAAQLvHAAAKA4DENBaFsAP_gAEPgAAwIKxtX_G9_bXlr8X736ftkeY1f99h77sQxBhZBk-4FzLvW_JwX32E7NA36tqYKmRIAu3TBIQNlHJDURVCgaogVrTDMaEyUoTtKJ6BkiFMRY2dYCFxvm4tjeQCY5vr991d52R-tbdrs3dzyy4hnv3a9_-S1WJCdA5-tDfv9bROb89IO5_x8v4v4_N7pE2_eT1l_tWvp7D9-ctv_9XX99_fbff9Pn_-uB_-_X__f_H37grAAQYCABAEAQICAAAAAQAAEAAEABAAAAAAACgAABEEAAEDAAAQAIAQAAABAABAAAAIAAAAAgACAAAAAEAgAAAACgADAAAAAAAYAAAMAEgIAAAAAQACmABAIFAAEJAFAEACEAEEAIQAABAEACAEABRwBAACBAoAAAQAAEAAAFgIDgAQEpAgACIgEAAAIAEAggAAEQjYACCAASCqqBAiiCAQLBoQFPaQAkgBACDgmQAgABQAHAAsA.f_gAAAAAAAAA").unwrap()
            .decode_all_sections();
        assert!(matches!(r[0], Err(SectionDecodeError::Read { .. })));
    }

    #[test]
    fn very_large_string() {
        let s = "DBACMYA~CQMC4oAQMC4oAPoABABGBaEAAP_gAP_gAAqIKxtX_G__bXlv-X736ftkeY1f99h77sQxBhbJs-4FzLvW_JwX32E7NE36tqYKmRIAu3TBIQNtHJjURVChaogVrTDsaEyUoTtKJ-BkiHMRY2dYCFxvm4tjeQCZ5vr_91d52R_t7dr-3dzyy5hnv3a9_-S1WJidK5-tHfv9bROb-_I-9_x-_4v4_N7pE2_eT1t_tWvt739-8tv_9__99__7_f______3_-_f__f____grG1f8b_9teW_5fvfp-2R5jV_32HvuxDEGFsmz7gXMu9b8nBffYTs0Tfq2pgqZEgC7dMEhA20cmNRFUKFqiBWtMOxoTJShO0on4GSIcxFjZ1gIXG-bi2N5AJnm-v_3V3nZH-3t2v7d3PLLmGe_dr3_5LVYmJ0rn60d-_1tE5v78j73_H7_i_j83ukTb95PW3-1a-3vf37y2__3__33__v9_______f_79__9____-AAA.QKxtX_G__bXlv-X736ftkeY1f99h77sQxBhbJs-4FzLvW_JwX32E7NE36tqYKmRIAu3TBIQNtHJjURVChaogVrTDsaEyUoTtKJ-BkiHMRY2dYCFxvm4tjeQCZ5vr_91d52R_t7dr-3dzyy5hnv3a9_-S1WJidK5-tHfv9bROb-_I-9_x-_4v4_N7pE2_eT1t_tWvt739-8tv_9__99__7_f______3_-_f__f____gAA.IKxtX_G__bXlv-X736ftkeY1f99h77sQxBhbJs-4FzLvW_JwX32E7NE36tqYKmRIAu3TBIQNtHJjURVChaogVrTDsaEyUoTtKJ-BkiHMRY2dYCFxvm4tjeQCZ5vr_91d52R_t7dr-3dzyy5hnv3a9_-S1WJidK5-tHfv9bROb-_I-9_x-_4v4_N7pE2_eT1t_tWvt739-8tv_9__99__7_f______3_-_f__f____gAA~BQMC4oAQMC4oAPoABABGB0CYAf8AAf8AAAqdA-AAUABwAFQALQAaABLACgAF0ANoAdwA_QCCAIQARQAnwBWgC3AGUANMAc4A7gCAQElASYAnYBPwDFAGaAM6AZ8A14B_AEngJyAT-Ao8BUQCpQFvALhAXQAvcBf4DBwGYANNAbUA3EBxoDxAHmgPkAgIBCQCNwEpYJgAmCBNUCa4E5gJ-AUmApYBU4FToHwACgAOAAqABaADQAJYAUAAugBtADuAH6AQQBCACKAE-AK0AW4AygBpgDnAHcAQCAkoCTAE7AJ-AYoAzQBnQDPgGvAP4Ak8BOQCfwFHgKiAVKAt4BcIC6AF7gL_AYOAzABpoDagG4gONAeIA80B8gEBAISARuAlLBMAEwQJqgTXAnMBPwCkwFLAKnAAAA.YAAAAAAAAAA";
        let r = GPPString::from_str(s);
        assert!(matches!(r, Ok(GPPString { .. })));

        let gpp = r.unwrap();

        assert_eq!(
            gpp.section_ids,
            vec![SectionId::TcfEuV2, SectionId::TcfCaV1]
        );

        let tcfeuv2 = gpp.decode_section(SectionId::TcfEuV2);
        assert!(
            matches!(tcfeuv2, Ok(Section::TcfEuV2(_))),
            "got {:?}",
            tcfeuv2
        );

        let tcfcav1 = gpp.decode_section(SectionId::TcfCaV1);
        assert!(
            matches!(tcfcav1, Ok(Section::TcfCaV1(_))),
            "got {:?}",
            tcfcav1
        );
    }

    macro_rules! assert_implements {
        ($type:ty, [$($trait:path),+]) => {
            {
                $(const _: fn() = || {
                    fn _assert_impl<T: $trait>() {}
                    _assert_impl::<$type>();
                };)+
            }
        };
    }

    #[test]
    fn gpp_string_implements_traits() {
        assert_implements!(GPPString, [Send, Sync]);
    }

    #[test]
    fn section_implements_traits() {
        assert_implements!(Section, [Send, Sync]);
    }
}

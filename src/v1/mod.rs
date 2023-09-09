use crate::core::{DataReader, DecodeExt};
use crate::sections::id::GPP_HEADER;
use crate::sections::{decode_section, Section, SectionDecodeError};
use fnv::FnvHashMap;
use std::io;
use std::str::FromStr;
use thiserror::Error;

const GPP_VERSION: u8 = 1;

#[derive(Error, Debug)]
pub enum GPPDecodeError {
    #[error("no header found")]
    NoHeaderFound,
    #[error("unable to decode header")]
    DecodeHeader(#[from] base64::DecodeError),
    #[error("invalid header type (expected {GPP_HEADER}, found {found})")]
    InvalidHeaderType { found: u8 },
    #[error("invalid GPP version (expected {GPP_VERSION}, found {found})")]
    InvalidGPPVersion { found: u8 },
    #[error("unable to read string")]
    Read(#[from] io::Error),
    #[error("ids do not match sections (number of ids {ids}, number of sections {sections}")]
    IdSectionMismatch { ids: usize, sections: usize },
    #[error("unable to decode section")]
    SectionDecode(#[from] SectionDecodeError),
}

pub struct GPPModel {
    pub section_ids: Vec<u8>,
    pub sections: Vec<Section>,
}

impl FromStr for GPPModel {
    type Err = GPPDecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (section_ids, sections) = extract_gpp_sections_from_str(s)?;

        let sections = section_ids
            .iter()
            .zip(sections)
            .map(|(&id, s)| decode_section(id, s))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            section_ids,
            sections,
        })
    }
}

pub trait SectionMapper {
    fn section(&self, id: u8) -> Option<&str>;
    fn section_ids(&self) -> &[u8];

    fn decode_section(&self, id: u8) -> Option<Result<Section, SectionDecodeError>> {
        let s = self.section(id)?;
        Some(decode_section(id, s))
    }

    fn decode_all_sections(&self) -> Result<Vec<Section>, SectionDecodeError> {
        self.section_ids()
            .iter()
            .filter_map(|id| self.decode_section(*id))
            .collect::<Result<Vec<_>, _>>()
    }
}

pub trait ToGPPStr {
    fn to_gpp_str(&self) -> Result<GPPStr, GPPDecodeError>;
}

impl ToGPPStr for &str {
    fn to_gpp_str(&self) -> Result<GPPStr, GPPDecodeError> {
        GPPStr::extract_from_str(self)
    }
}

pub struct GPPStr<'a> {
    pub section_ids: Vec<u8>,
    pub sections: FnvHashMap<u8, &'a str>,
}

impl<'a> GPPStr<'a> {
    pub fn extract_from_str(s: &'a str) -> Result<Self, GPPDecodeError> {
        let (section_ids, sections) = extract_gpp_sections_from_str(s)?;

        let sections = section_ids
            .iter()
            .zip(sections)
            .map(|(&id, s)| (id, s))
            .collect();

        Ok(Self {
            section_ids,
            sections,
        })
    }
}

impl<'a> SectionMapper for GPPStr<'a> {
    fn section(&self, id: u8) -> Option<&str> {
        self.sections.get(&id).copied()
    }

    fn section_ids(&self) -> &[u8] {
        &self.section_ids
    }
}

pub struct GPPString {
    pub section_ids: Vec<u8>,
    pub sections: FnvHashMap<u8, String>,
}

impl GPPString {
    pub fn as_gpp_str(&self) -> GPPStr {
        GPPStr {
            section_ids: self.section_ids.clone(),
            sections: self
                .sections
                .iter()
                .map(|(&id, s)| (id, s.as_str()))
                .collect(),
        }
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

impl SectionMapper for GPPString {
    fn section(&self, id: u8) -> Option<&str> {
        self.sections.get(&id).map(|s| s.as_str())
    }

    fn section_ids(&self) -> &[u8] {
        &self.section_ids
    }
}

fn extract_gpp_sections_from_str(s: &str) -> Result<(Vec<u8>, Vec<&str>), GPPDecodeError> {
    let mut sections_iter = s.split('~');

    let header_str = sections_iter.next().ok_or(GPPDecodeError::NoHeaderFound)?;
    let header = header_str.decode_base64_url()?;
    let mut reader = DataReader::new(&header);

    let header_type = reader.read_fixed_integer::<u8>(6)?;
    if header_type != GPP_HEADER {
        return Err(GPPDecodeError::InvalidHeaderType { found: header_type });
    }

    let gpp_version = reader.read_fixed_integer::<u8>(6)?;
    if gpp_version != GPP_VERSION {
        return Err(GPPDecodeError::InvalidGPPVersion { found: gpp_version });
    }

    let section_ids = reader.read_fibonacci_range()?;
    let sections = sections_iter.collect::<Vec<_>>();
    if sections.len() != section_ids.len() {
        return Err(GPPDecodeError::IdSectionMismatch {
            ids: section_ids.len(),
            sections: sections.len(),
        });
    }

    Ok((section_ids, sections))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sections::id::{TCF_CA_V1, TCF_EU_V2, USP_V1};

    #[test]
    fn gpp_model_parse_str() {
        let r = GPPModel::from_str("DBABMA~CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA").unwrap();

        assert_eq!(r.section_ids, vec![TCF_EU_V2]);
        assert!(matches!(r.sections[0], Section::TcfEuV2(_)));
    }

    #[test]
    fn gpp_model_parse_str_multiple_sections() {
        let r = GPPModel::from_str("DBACNYA~CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA~1YNN")
            .unwrap();

        assert_eq!(r.section_ids, vec![TCF_EU_V2, USP_V1]);
        assert!(matches!(r.sections[0], Section::TcfEuV2(_)));
        assert!(matches!(r.sections[1], Section::UspV1(_)));
    }

    #[test]
    fn gpp_model_parse_str_multiple_sections_unsupported() {
        let r =
            GPPModel::from_str("DBABjw~BPXuQIAPXuQIAAfKABENB-CgAAAAAAAAAAAAAAAA.YAAAAAAAAAA~1YNN")
                .unwrap();

        assert_eq!(r.section_ids, vec![TCF_CA_V1, USP_V1]);
        assert!(matches!(&r.sections[0], Section::TcfCaV1(_)));
        assert!(matches!(r.sections[1], Section::UspV1(_)));
    }

    #[test]
    fn str_as_gpp_str() {
        let r = "DBABMA~CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA"
            .to_gpp_str()
            .unwrap();

        assert_eq!(r.section_ids, vec![TCF_EU_V2]);
        assert!(matches!(
            r.section(TCF_EU_V2).unwrap(),
            "CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA"
        ));
    }

    #[test]
    fn gpp_str_parse_str() {
        let r = GPPStr::extract_from_str("DBABMA~CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA")
            .unwrap();

        assert_eq!(r.section_ids, vec![TCF_EU_V2]);
        assert_eq!(
            r.sections[&TCF_EU_V2],
            "CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA"
        );
        assert!(matches!(
            r.decode_section(TCF_EU_V2),
            Some(Ok(Section::TcfEuV2(_)))
        ));
    }

    #[test]
    fn gpp_str_parse_str_multiple_sections() {
        let r =
            GPPStr::extract_from_str("DBACNYA~CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA~1YNN")
                .unwrap();

        assert_eq!(r.section_ids, vec![TCF_EU_V2, USP_V1]);
        assert_eq!(
            r.sections[&TCF_EU_V2],
            "CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA"
        );
        assert_eq!(r.sections[&USP_V1], "1YNN");
        assert!(matches!(
            r.decode_section(TCF_EU_V2),
            Some(Ok(Section::TcfEuV2(_)))
        ));
        assert!(matches!(
            r.decode_section(USP_V1),
            Some(Ok(Section::UspV1(_)))
        ));
    }

    #[test]
    fn gpp_str_decode_all() {
        let r =
            GPPStr::extract_from_str("DBACNYA~CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA~1YNN")
                .unwrap();

        let s = r.decode_all_sections().unwrap();

        assert!(matches!(s[0], Section::TcfEuV2(_)));
        assert!(matches!(s[1], Section::UspV1(_)));
    }

    #[test]
    fn gpp_string_parse_str() {
        let r = GPPString::from_str("DBABMA~CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA").unwrap();

        assert_eq!(r.section_ids, vec![TCF_EU_V2]);
        assert_eq!(
            r.sections[&TCF_EU_V2].as_str(),
            "CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA"
        );
        assert!(matches!(
            r.decode_section(TCF_EU_V2),
            Some(Ok(Section::TcfEuV2(_)))
        ));
    }

    #[test]
    fn gpp_string_parse_str_multiple_sections() {
        let r = GPPString::from_str("DBACNYA~CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA~1YNN")
            .unwrap();

        assert_eq!(r.section_ids, vec![TCF_EU_V2, USP_V1]);
        assert_eq!(
            r.sections[&TCF_EU_V2].as_str(),
            "CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA"
        );
        assert_eq!(r.sections[&USP_V1].as_str(), "1YNN");
        assert!(matches!(
            r.decode_section(TCF_EU_V2),
            Some(Ok(Section::TcfEuV2(_)))
        ));
        assert!(matches!(
            r.decode_section(USP_V1),
            Some(Ok(Section::UspV1(_)))
        ));
    }

    #[test]
    fn gpp_string_decode_all() {
        let r = GPPString::from_str("DBACNYA~CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA~1YNN")
            .unwrap();

        let s = r.decode_all_sections().unwrap();

        assert!(matches!(s[0], Section::TcfEuV2(_)));
        assert!(matches!(s[1], Section::UspV1(_)));
    }

    #[test]
    fn gpp_string_as_gpp_str() {
        let r = GPPString::from_str("DBABMA~CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA").unwrap();
        let r = r.as_gpp_str();

        assert_eq!(r.section_ids, vec![TCF_EU_V2]);
        assert!(matches!(
            r.section(TCF_EU_V2).unwrap(),
            "CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA"
        ));
    }
}

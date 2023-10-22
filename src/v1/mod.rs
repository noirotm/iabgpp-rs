use crate::core::{DataReader, DecodeExt};
use crate::sections::{decode_section, Section, SectionDecodeError, SectionId};
use fnv::FnvHashMap;
use num_traits::FromPrimitive;
use std::io;
use std::str::FromStr;
use thiserror::Error;

const GPP_HEADER: u8 = 3;
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
    #[error("unsupported section id {0}")]
    UnsupportedSectionId(u8),
    #[error("ids do not match sections (number of ids {ids}, number of sections {sections}")]
    IdSectionMismatch { ids: usize, sections: usize },
    #[error("unable to decode section")]
    SectionDecode(#[from] SectionDecodeError),
}

pub trait SectionMapper {
    fn section(&self, id: SectionId) -> Option<&str>;
    fn section_ids(&self) -> &[SectionId];

    fn sections(&self) -> Vec<&str> {
        self.section_ids()
            .iter()
            .filter_map(|id| self.section(*id))
            .collect()
    }

    fn decode_section(&self, id: SectionId) -> Option<Result<Section, SectionDecodeError>> {
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
    section_ids: Vec<SectionId>,
    sections: FnvHashMap<SectionId, &'a str>,
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
    fn section(&self, id: SectionId) -> Option<&str> {
        self.sections.get(&id).copied()
    }

    fn section_ids(&self) -> &[SectionId] {
        &self.section_ids
    }
}

pub struct GPPString {
    section_ids: Vec<SectionId>,
    sections: FnvHashMap<SectionId, String>,
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
    fn section(&self, id: SectionId) -> Option<&str> {
        self.sections.get(&id).map(|s| s.as_str())
    }

    fn section_ids(&self) -> &[SectionId] {
        &self.section_ids
    }
}

fn extract_gpp_sections_from_str(s: &str) -> Result<(Vec<SectionId>, Vec<&str>), GPPDecodeError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("DBABMA~CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA" => vec![SectionId::TcfEuV2] ; "single section")]
    #[test_case("DBACNYA~CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA~1YNN" => vec![SectionId::TcfEuV2, SectionId::UspV1] ; "tcf eu and us sections")]
    #[test_case("DBABjw~BPXuQIAPXuQIAAfKABENB-CgAAAAAAAAAAAAAAAA.YAAAAAAAAAA~1YNN" => vec![SectionId::TcfCaV1, SectionId::UspV1] ; "tcf ca and us sections")]
    fn gpp_string_section_ids(s: &str) -> Vec<SectionId> {
        GPPString::from_str(s).unwrap().section_ids
    }

    #[test_case("DBABMA~CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA" => vec!["CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA"] ; "single section")]
    #[test_case("DBACNYA~CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA~1YNN" => vec!["CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA", "1YNN"] ; "tcf eu and us sections")]
    #[test_case("DBABjw~BPXuQIAPXuQIAAfKABENB-CgAAAAAAAAAAAAAAAA.YAAAAAAAAAA~1YNN" => vec!["BPXuQIAPXuQIAAfKABENB-CgAAAAAAAAAAAAAAAA.YAAAAAAAAAA", "1YNN"] ; "tcf ca and us sections")]
    fn gpp_string_sections(s: &str) -> Vec<String> {
        GPPString::from_str(s)
            .unwrap()
            .sections()
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    #[test_case("DBABMA~CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA" => vec![SectionId::TcfEuV2] ; "single section")]
    #[test_case("DBACNYA~CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA~1YNN" => vec![SectionId::TcfEuV2, SectionId::UspV1] ; "tcf eu and us sections")]
    #[test_case("DBABjw~BPXuQIAPXuQIAAfKABENB-CgAAAAAAAAAAAAAAAA.YAAAAAAAAAA~1YNN" => vec![SectionId::TcfCaV1, SectionId::UspV1] ; "tcf ca and us sections")]
    fn gpp_string_decode_section(s: &str) -> Vec<SectionId> {
        let s = GPPString::from_str(s).unwrap();
        s.section_ids
            .iter()
            .map(|id| s.decode_section(*id).unwrap().unwrap().id())
            .collect()
    }

    #[test_case("DBABMA~CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA" => vec![SectionId::TcfEuV2] ; "single section")]
    #[test_case("DBACNYA~CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA~1YNN" => vec![SectionId::TcfEuV2, SectionId::UspV1] ; "tcf eu and us sections")]
    #[test_case("DBABjw~BPXuQIAPXuQIAAfKABENB-CgAAAAAAAAAAAAAAAA.YAAAAAAAAAA~1YNN" => vec![SectionId::TcfCaV1, SectionId::UspV1] ; "tcf ca and us sections")]
    fn gpp_str_decode_all(s: &str) -> Vec<SectionId> {
        GPPString::from_str(s)
            .unwrap()
            .decode_all_sections()
            .unwrap()
            .iter()
            .map(|s| s.id())
            .collect()
    }

    #[test_case("DBABMA~CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA" => vec![SectionId::TcfEuV2] ; "single section")]
    #[test_case("DBACNYA~CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA~1YNN" => vec![SectionId::TcfEuV2, SectionId::UspV1] ; "tcf eu and us sections")]
    #[test_case("DBABjw~BPXuQIAPXuQIAAfKABENB-CgAAAAAAAAAAAAAAAA.YAAAAAAAAAA~1YNN" => vec![SectionId::TcfCaV1, SectionId::UspV1] ; "tcf ca and us sections")]
    fn gpp_str_section_ids(s: &str) -> Vec<SectionId> {
        GPPStr::extract_from_str(s).unwrap().section_ids
    }

    #[test_case("DBABMA~CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA" => vec!["CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA"] ; "single section")]
    #[test_case("DBACNYA~CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA~1YNN" => vec!["CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA", "1YNN"] ; "tcf eu and us sections")]
    #[test_case("DBABjw~BPXuQIAPXuQIAAfKABENB-CgAAAAAAAAAAAAAAAA.YAAAAAAAAAA~1YNN" => vec!["BPXuQIAPXuQIAAfKABENB-CgAAAAAAAAAAAAAAAA.YAAAAAAAAAA", "1YNN"] ; "tcf ca and us sections")]
    fn gpp_str_sections(s: &str) -> Vec<String> {
        GPPStr::extract_from_str(s)
            .unwrap()
            .sections()
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    #[test_case("DBABMA~CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA" => vec![SectionId::TcfEuV2] ; "single section")]
    #[test_case("DBACNYA~CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA~1YNN" => vec![SectionId::TcfEuV2, SectionId::UspV1] ; "tcf eu and us sections")]
    #[test_case("DBABjw~BPXuQIAPXuQIAAfKABENB-CgAAAAAAAAAAAAAAAA.YAAAAAAAAAA~1YNN" => vec![SectionId::TcfCaV1, SectionId::UspV1] ; "tcf ca and us sections")]
    fn gpp_str_decode_section(s: &str) -> Vec<SectionId> {
        let s = GPPStr::extract_from_str(s).unwrap();
        s.section_ids
            .iter()
            .map(|id| s.decode_section(*id).unwrap().unwrap().id())
            .collect()
    }

    #[test_case("DBABMA~CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA" => vec![SectionId::TcfEuV2] ; "single section")]
    #[test_case("DBACNYA~CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA~1YNN" => vec![SectionId::TcfEuV2, SectionId::UspV1] ; "tcf eu and us sections")]
    #[test_case("DBABjw~BPXuQIAPXuQIAAfKABENB-CgAAAAAAAAAAAAAAAA.YAAAAAAAAAA~1YNN" => vec![SectionId::TcfCaV1, SectionId::UspV1] ; "tcf ca and us sections")]
    fn gpp_string_decode_all(s: &str) -> Vec<SectionId> {
        GPPStr::extract_from_str(s)
            .unwrap()
            .decode_all_sections()
            .unwrap()
            .iter()
            .map(|s| s.id())
            .collect()
    }

    #[test]
    fn str_to_gpp_str() {
        let r = "DBABMA~CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA"
            .to_gpp_str()
            .unwrap();

        assert_eq!(r.section_ids, vec![SectionId::TcfEuV2]);
        assert!(matches!(
            r.section(SectionId::TcfEuV2).unwrap(),
            "CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA"
        ));
    }

    #[test]
    fn gpp_string_as_gpp_str() {
        let r = GPPString::from_str("DBABMA~CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA").unwrap();
        let r = r.as_gpp_str();

        assert_eq!(r.section_ids, vec![SectionId::TcfEuV2]);
        assert!(matches!(
            r.section(SectionId::TcfEuV2).unwrap(),
            "CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA"
        ));
    }

    #[test]
    fn truncated_string() {
        let r = GPPString::from_str("DBACNYA~CPytTYAPytTYABEACBENDXCoAP_AAH_AAAIwgoNf_X__b3_v-_7___t0eY1f9_7__-0zjhfdt-8N3f_X_L8X_2M7");
        assert!(matches!(
            r,
            Err(GPPDecodeError::IdSectionMismatch {
                ids: 2,
                sections: 1
            })
        ));
    }
}

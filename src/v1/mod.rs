use crate::core::{DataReader, DecodeExt};
use crate::sections::{decode_section, Section, SectionDecodeError};
use std::io;
use std::str::FromStr;
use thiserror::Error;

const GPP_HEADER_TYPE: u8 = 3;
const GPP_VERSION: u8 = 1;

#[derive(Error, Debug)]
pub enum GPPDecodeError {
    #[error("no header found")]
    NoHeaderFound,
    #[error("unable to decode header")]
    DecodeHeader(#[from] base64::DecodeError),
    #[error("invalid header type (expected {GPP_HEADER_TYPE}, found {found})")]
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
    pub section_ids: Vec<u64>,
    pub sections: Vec<Section>,
}

impl FromStr for GPPModel {
    type Err = GPPDecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut sections_iter = s.split('~');

        let header_str = sections_iter.next().ok_or(GPPDecodeError::NoHeaderFound)?;
        let header = header_str.decode_base64_url()?;
        let mut reader = DataReader::new(&header);

        let header_type = reader.read_fixed_integer::<u8>(6)?;
        if header_type != GPP_HEADER_TYPE {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sections::id::{TCF_EU_V2, USP_V1};

    #[test]
    fn test_parse_str() {
        let r = GPPModel::from_str("DBABMA~CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA").unwrap();

        assert_eq!(r.section_ids, vec![TCF_EU_V2]);
        assert!(matches!(r.sections[0], Section::TcfEuV2(_)));
    }

    #[test]
    fn test_parse_str_multiple_sections1() {
        let r = GPPModel::from_str("DBACNYA~CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA~1YNN")
            .unwrap();

        assert_eq!(r.section_ids, vec![TCF_EU_V2, USP_V1]);
        assert!(matches!(r.sections[0], Section::TcfEuV2(_)));
        assert!(matches!(r.sections[1], Section::UspV1(_)));
    }

    /*#[test]
    fn test_parse_str_multiple_sections2() {
        let r = GPPPayload::try_from("DBABjw~CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA~1YNN")
            .unwrap();

        assert_eq!(r.sections[0].id, SectionId::TCFCA);
        assert_eq!(
            r.sections[0].encoded_payload,
            "CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA"
        );
        assert_eq!(r.sections[1].id, SectionId::USPV1);
        assert_eq!(r.sections[1].encoded_payload, "1YNN");
    }*/
}

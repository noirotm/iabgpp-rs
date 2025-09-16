use assert_json_diff::assert_json_eq;
use iab_gpp::sections::{Section, SectionDecodeError};
use iab_gpp::v1::GPPString;
use serde::Deserialize;
use std::fs::File;
use std::io;
use std::io::ErrorKind;
use std::path::Path;
use std::str::FromStr;

#[derive(Deserialize)]
pub struct TestCase {
    gpp_string: String,
    expected_sections: Vec<Section>,
}

impl TestCase {
    pub fn load_from_file<P: AsRef<Path>>(p: P) -> io::Result<Self> {
        let f = File::open(p)?;
        let tc: Self = serde_json::from_reader(&f)
            .map_err(|e| io::Error::new(ErrorKind::InvalidData, e.to_string()))?;
        Ok(tc)
    }

    pub fn assert_json_matches(&self) {
        let s = GPPString::from_str(&self.gpp_string).expect("invalid GPP string");

        let sections = s
            .decode_all_sections()
            .into_iter()
            .collect::<Result<Vec<_>, SectionDecodeError>>();

        if sections.is_err() {
            panic!(
                "sections decode error: {:?}",
                sections.unwrap_err().to_string()
            );
        }

        assert_json_eq!(sections.unwrap(), self.expected_sections);
    }
}

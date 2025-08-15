use assert_json_diff::assert_json_eq;
use iab_gpp::sections::SectionDecodeError;
use iab_gpp::v1::GPPString;
use serde_json::Value;
use std::io::ErrorKind;
use std::path::Path;
use std::str::FromStr;
use std::{fs, io};

pub struct TestCase {
    gpp_string: String,
    expected_json: String,
}

impl TestCase {
    pub fn load_from_file<P: AsRef<Path>>(p: P) -> io::Result<Self> {
        let s = fs::read_to_string(&p)?;
        let (s, json) = s
            .split_once(['\n', '\r'])
            .ok_or(io::Error::new(ErrorKind::InvalidData, "invalid test data"))?;

        Ok(Self {
            gpp_string: s.to_string(),
            expected_json: json.to_string(),
        })
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

        let expected_value: Value =
            serde_json::from_str(&self.expected_json).expect("invalid JSON");

        assert_json_eq!(sections.unwrap(), expected_value);
    }
}

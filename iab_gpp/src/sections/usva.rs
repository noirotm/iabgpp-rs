use crate::sections::us_common::{
    parse_mspa_covered_transaction, Consent, MspaMode, Notice, OptOut,
};
use iab_gpp_derive::{FromDataReader, GPPSection};

#[derive(Debug, Eq, PartialEq, GPPSection)]
#[non_exhaustive]
pub struct UsVa {
    pub core: Core,
}

#[derive(Debug, Eq, PartialEq, FromDataReader)]
#[non_exhaustive]
#[gpp(section_version = 1)]
pub struct Core {
    pub sharing_notice: Notice,
    pub sale_opt_out_notice: Notice,
    pub targeted_advertising_opt_out_notice: Notice,
    pub sale_opt_out: OptOut,
    pub targeted_advertising_opt_out: OptOut,
    pub sensitive_data_processing: SensitiveDataProcessing,
    pub known_child_sensitive_data_consents: Consent,
    #[gpp(parse_with = parse_mspa_covered_transaction)]
    pub mspa_covered_transaction: bool,
    pub mspa_opt_out_option_mode: MspaMode,
    pub mspa_service_provider_mode: MspaMode,
}

#[derive(Debug, Eq, PartialEq, FromDataReader)]
#[non_exhaustive]
pub struct SensitiveDataProcessing {
    pub racial_or_ethnic_origin: Consent,
    pub religious_or_philosophical_beliefs: Consent,
    pub health_diagnosis_data: Consent,
    pub sex_life_or_sexual_orientation: Consent,
    pub citizenship_or_immigration_status: Consent,
    pub genetic_unique_identification: Consent,
    pub biometric_unique_identification: Consent,
    pub precise_geolocation_data: Consent,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sections::SectionDecodeError;
    use std::str::FromStr;
    use test_case::test_case;

    #[test]
    fn parse() {
        let test_cases = [
            (
                "BAAAABA",
                UsVa {
                    core: Core {
                        sharing_notice: Notice::NotApplicable,
                        sale_opt_out_notice: Notice::NotApplicable,
                        targeted_advertising_opt_out_notice: Notice::NotApplicable,
                        sale_opt_out: OptOut::NotApplicable,
                        targeted_advertising_opt_out: OptOut::NotApplicable,
                        sensitive_data_processing: SensitiveDataProcessing {
                            racial_or_ethnic_origin: Consent::NotApplicable,
                            religious_or_philosophical_beliefs: Consent::NotApplicable,
                            health_diagnosis_data: Consent::NotApplicable,
                            sex_life_or_sexual_orientation: Consent::NotApplicable,
                            citizenship_or_immigration_status: Consent::NotApplicable,
                            genetic_unique_identification: Consent::NotApplicable,
                            biometric_unique_identification: Consent::NotApplicable,
                            precise_geolocation_data: Consent::NotApplicable,
                        },
                        known_child_sensitive_data_consents: Consent::NotApplicable,
                        mspa_covered_transaction: true,
                        mspa_opt_out_option_mode: MspaMode::NotApplicable,
                        mspa_service_provider_mode: MspaMode::NotApplicable,
                    },
                },
            ),
            (
                "BVVVVWY",
                UsVa {
                    core: Core {
                        sharing_notice: Notice::Provided,
                        sale_opt_out_notice: Notice::Provided,
                        targeted_advertising_opt_out_notice: Notice::Provided,
                        sale_opt_out: OptOut::OptedOut,
                        targeted_advertising_opt_out: OptOut::OptedOut,
                        sensitive_data_processing: SensitiveDataProcessing {
                            racial_or_ethnic_origin: Consent::NoConsent,
                            religious_or_philosophical_beliefs: Consent::NoConsent,
                            health_diagnosis_data: Consent::NoConsent,
                            sex_life_or_sexual_orientation: Consent::NoConsent,
                            citizenship_or_immigration_status: Consent::NoConsent,
                            genetic_unique_identification: Consent::NoConsent,
                            biometric_unique_identification: Consent::NoConsent,
                            precise_geolocation_data: Consent::NoConsent,
                        },
                        known_child_sensitive_data_consents: Consent::NoConsent,
                        mspa_covered_transaction: false,
                        mspa_opt_out_option_mode: MspaMode::Yes,
                        mspa_service_provider_mode: MspaMode::No,
                    },
                },
            ),
        ];

        for (s, expected) in test_cases {
            let actual = UsVa::from_str(s).unwrap();
            assert_eq!(actual, expected);
        }
    }

    #[test_case("" => matches SectionDecodeError::Read(_) ; "empty string")]
    #[test_case("123" => matches SectionDecodeError::UnknownSegmentVersion { .. } ; "decode error")]
    #[test_case("CVVVVVVVVWA" => matches SectionDecodeError::UnknownSegmentVersion { .. } ; "unknown segment version")]
    fn error(s: &str) -> SectionDecodeError {
        UsVa::from_str(s).unwrap_err()
    }
}

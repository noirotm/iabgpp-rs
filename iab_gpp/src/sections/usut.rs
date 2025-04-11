use crate::sections::us_common::{
    parse_mspa_covered_transaction, Consent, MspaMode, Notice, OptOut,
};
use iab_gpp_derive::{FromBitStream, GPPSection};
#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, Eq, PartialEq, GPPSection)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[non_exhaustive]
pub struct UsUt {
    pub core: Core,
}

#[derive(Debug, Eq, PartialEq, FromBitStream)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[non_exhaustive]
#[gpp(section_version = 1)]
pub struct Core {
    pub sharing_notice: Notice,
    pub sale_opt_out_notice: Notice,
    pub targeted_advertising_opt_out_notice: Notice,
    pub sensitive_data_processing_opt_out_notice: Notice,
    pub sale_opt_out: OptOut,
    pub targeted_advertising_opt_out: OptOut,
    pub sensitive_data_processing: SensitiveDataProcessing,
    pub known_child_sensitive_data_consents: Consent,
    #[gpp(parse_with = parse_mspa_covered_transaction)]
    pub mspa_covered_transaction: bool,
    pub mspa_opt_out_option_mode: MspaMode,
    pub mspa_service_provider_mode: MspaMode,
}

#[derive(Debug, Eq, PartialEq, FromBitStream)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[non_exhaustive]
pub struct SensitiveDataProcessing {
    pub racial_or_ethnic_origin: Consent,
    pub religious_beliefs: Consent,
    pub sexual_orientation: Consent,
    pub citizenship_or_immigration_status: Consent,
    pub health_data: Consent,
    pub genetic_unique_identification: Consent,
    pub biometric_unique_identification: Consent,
    pub specific_geolocation_data: Consent,
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
                "BAAAAAQA",
                UsUt {
                    core: Core {
                        sharing_notice: Notice::NotApplicable,
                        sale_opt_out_notice: Notice::NotApplicable,
                        targeted_advertising_opt_out_notice: Notice::NotApplicable,
                        sensitive_data_processing_opt_out_notice: Notice::NotApplicable,
                        sale_opt_out: OptOut::NotApplicable,
                        targeted_advertising_opt_out: OptOut::NotApplicable,
                        sensitive_data_processing: SensitiveDataProcessing {
                            racial_or_ethnic_origin: Consent::NotApplicable,
                            religious_beliefs: Consent::NotApplicable,
                            sexual_orientation: Consent::NotApplicable,
                            citizenship_or_immigration_status: Consent::NotApplicable,
                            health_data: Consent::NotApplicable,
                            genetic_unique_identification: Consent::NotApplicable,
                            biometric_unique_identification: Consent::NotApplicable,
                            specific_geolocation_data: Consent::NotApplicable,
                        },
                        known_child_sensitive_data_consents: Consent::NotApplicable,
                        mspa_covered_transaction: true,
                        mspa_opt_out_option_mode: MspaMode::NotApplicable,
                        mspa_service_provider_mode: MspaMode::NotApplicable,
                    },
                },
            ),
            (
                "BVVVVVmA",
                UsUt {
                    core: Core {
                        sharing_notice: Notice::Provided,
                        sale_opt_out_notice: Notice::Provided,
                        targeted_advertising_opt_out_notice: Notice::Provided,
                        sensitive_data_processing_opt_out_notice: Notice::Provided,
                        sale_opt_out: OptOut::OptedOut,
                        targeted_advertising_opt_out: OptOut::OptedOut,
                        sensitive_data_processing: SensitiveDataProcessing {
                            racial_or_ethnic_origin: Consent::NoConsent,
                            religious_beliefs: Consent::NoConsent,
                            sexual_orientation: Consent::NoConsent,
                            citizenship_or_immigration_status: Consent::NoConsent,
                            health_data: Consent::NoConsent,
                            genetic_unique_identification: Consent::NoConsent,
                            biometric_unique_identification: Consent::NoConsent,
                            specific_geolocation_data: Consent::NoConsent,
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
            let actual = UsUt::from_str(s).unwrap();
            assert_eq!(actual, expected);
        }
    }

    #[test_case("" => matches SectionDecodeError::Read { .. }; "empty string")]
    #[test_case("123" => matches SectionDecodeError::UnknownSegmentVersion { .. }; "decode error")]
    #[test_case("CVVVVVVVVWA" => matches SectionDecodeError::UnknownSegmentVersion { .. }; "unknown segment version")]
    fn error(s: &str) -> SectionDecodeError {
        UsUt::from_str(s).unwrap_err()
    }
}

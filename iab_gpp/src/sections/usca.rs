use crate::sections::us_common::{
    is_notice_and_opt_out_combination_ok, parse_mspa_covered_transaction, Consent, MspaMode,
    Notice, OptOut, ValidationError,
};
use iab_gpp_derive::{FromDataReader, GPPSection};

#[derive(Debug, Eq, PartialEq, GPPSection)]
#[non_exhaustive]
#[gpp(with_optional_segments(bits = 2))]
pub struct UsCa {
    pub core: Core,
    #[gpp(optional_segment_type = 1)]
    pub gpc: Option<bool>,
}

impl UsCa {
    /// Checks the consistency of values in the already populated fields.
    ///
    /// This is based on the code found in <https://iabgpp.com/js/3.2.0/cmpapi/encoder/segment/UsCaV1CoreSegment.js>.
    ///
    /// While CMPs shouldn't be able to generate invalid combinations, the binary wire format
    /// does not prevent it.
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = vec![];

        if !is_notice_and_opt_out_combination_ok(
            &self.core.sharing_opt_out_notice,
            &self.core.sharing_opt_out,
        ) {
            errors.push(ValidationError::new(
                "sharing_opt_out_notice",
                &self.core.sharing_opt_out_notice,
                "sharing_opt_out",
                &self.core.sharing_opt_out,
            ));
        }

        if !is_notice_and_opt_out_combination_ok(
            &self.core.sale_opt_out_notice,
            &self.core.sale_opt_out,
        ) {
            errors.push(ValidationError::new(
                "sale_opt_out_notice",
                &self.core.sale_opt_out_notice,
                "sale_opt_out",
                &self.core.sale_opt_out,
            ));
        }

        if self.core.mspa_service_provider_mode == MspaMode::NotApplicable {
            if self.core.sale_opt_out_notice != Notice::NotApplicable {
                errors.push(ValidationError::new(
                    "mspa_service_provider_mode",
                    &self.core.mspa_service_provider_mode,
                    "sale_opt_out_notice",
                    &self.core.sale_opt_out_notice,
                ));
            }
            if self.core.sharing_opt_out_notice != Notice::NotApplicable {
                errors.push(ValidationError::new(
                    "mspa_service_provider_mode",
                    &self.core.mspa_service_provider_mode,
                    "sharing_opt_out_notice",
                    &self.core.sharing_opt_out_notice,
                ));
            }
            if self.core.sensitive_data_limit_use_notice != Notice::NotApplicable {
                errors.push(ValidationError::new(
                    "mspa_service_provider_mode",
                    &self.core.mspa_service_provider_mode,
                    "sensitive_data_limit_use_notice",
                    &self.core.sensitive_data_limit_use_notice,
                ));
            }
        } else if self.core.mspa_service_provider_mode == MspaMode::Yes {
            if self.core.mspa_opt_out_option_mode != MspaMode::No {
                errors.push(ValidationError::new(
                    "mspa_service_provider_mode",
                    &self.core.mspa_service_provider_mode,
                    "mspa_opt_out_option_mode",
                    &self.core.mspa_opt_out_option_mode,
                ));
            }
            if self.core.sale_opt_out_notice != Notice::NotApplicable {
                errors.push(ValidationError::new(
                    "mspa_service_provider_mode",
                    &self.core.mspa_service_provider_mode,
                    "sale_opt_out_notice",
                    &self.core.sale_opt_out_notice,
                ));
            }
            if self.core.sharing_opt_out_notice != Notice::NotApplicable {
                errors.push(ValidationError::new(
                    "mspa_service_provider_mode",
                    &self.core.mspa_service_provider_mode,
                    "sharing_opt_out_notice",
                    &self.core.sharing_opt_out_notice,
                ));
            }
            if self.core.sensitive_data_limit_use_notice != Notice::NotApplicable {
                errors.push(ValidationError::new(
                    "mspa_service_provider_mode",
                    &self.core.mspa_service_provider_mode,
                    "sensitive_data_limit_use_notice",
                    &self.core.sensitive_data_limit_use_notice,
                ));
            }
        } else if self.core.mspa_service_provider_mode == MspaMode::No
            && self.core.mspa_opt_out_option_mode != MspaMode::Yes
        {
            errors.push(ValidationError::new(
                "mspa_service_provider_mode",
                &self.core.mspa_service_provider_mode,
                "mspa_opt_out_option_mode",
                &self.core.mspa_opt_out_option_mode,
            ));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[derive(Debug, Eq, PartialEq, FromDataReader)]
#[non_exhaustive]
#[gpp(section_version = 1)]
/// The core sub-section must always be present. Where terms are capitalized in the ‘description’
/// field they are defined terms in Cal. Civ. Code 1798.140.
pub struct Core {
    pub sale_opt_out_notice: Notice,
    pub sharing_opt_out_notice: Notice,
    pub sensitive_data_limit_use_notice: Notice,
    pub sale_opt_out: OptOut,
    pub sharing_opt_out: OptOut,
    pub sensitive_data_processing: SensitiveDataProcessing,
    pub known_child_sensitive_data_consents: KnownChildSensitiveDataConsents,
    pub personal_data_consent: Consent,
    #[gpp(parse_with = parse_mspa_covered_transaction)]
    pub mspa_covered_transaction: bool,
    pub mspa_opt_out_option_mode: MspaMode,
    pub mspa_service_provider_mode: MspaMode,
}

#[derive(Debug, Eq, PartialEq, FromDataReader)]
#[non_exhaustive]
pub struct SensitiveDataProcessing {
    /// Opt-Out of the Use or Disclosure of the Consumer's Sensitive Personal Information Which
    /// Reveals a Consumer's Social Security, Driver's License, State Identification Card, or
    /// Passport Number.
    pub identification_documents: OptOut,
    /// Opt-Out of the Use or Disclosure of the Consumer's Sensitive Personal Information Which
    /// Reveals a Consumer's Account Log-In, Financial Account, Debit Card, or Credit Card Number in
    /// Combination with Any Required Security or Access Code, Password, or Credentials Allowing
    /// Access to an Account.
    pub financial_data: OptOut,
    pub precise_geolocation: OptOut,
    pub origin_beliefs_or_union: OptOut,
    pub mail_email_or_text_messages: OptOut,
    pub genetic_data: OptOut,
    pub biometric_unique_identification: OptOut,
    pub health_data: OptOut,
    pub sex_life_or_sexual_orientation: OptOut,
}

#[derive(Debug, Eq, PartialEq, FromDataReader)]
#[non_exhaustive]
pub struct KnownChildSensitiveDataConsents {
    pub sell_personal_information: Consent,
    pub share_personal_information: Consent,
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
                "BAAAAACA",
                UsCa {
                    core: Core {
                        sale_opt_out_notice: Notice::NotApplicable,
                        sharing_opt_out_notice: Notice::NotApplicable,
                        sensitive_data_limit_use_notice: Notice::NotApplicable,
                        sale_opt_out: OptOut::NotApplicable,
                        sharing_opt_out: OptOut::NotApplicable,
                        sensitive_data_processing: SensitiveDataProcessing {
                            identification_documents: OptOut::NotApplicable,
                            financial_data: OptOut::NotApplicable,
                            precise_geolocation: OptOut::NotApplicable,
                            origin_beliefs_or_union: OptOut::NotApplicable,
                            mail_email_or_text_messages: OptOut::NotApplicable,
                            genetic_data: OptOut::NotApplicable,
                            biometric_unique_identification: OptOut::NotApplicable,
                            health_data: OptOut::NotApplicable,
                            sex_life_or_sexual_orientation: OptOut::NotApplicable,
                        },
                        known_child_sensitive_data_consents: KnownChildSensitiveDataConsents {
                            sell_personal_information: Consent::NotApplicable,
                            share_personal_information: Consent::NotApplicable,
                        },
                        personal_data_consent: Consent::NotApplicable,
                        mspa_covered_transaction: false,
                        mspa_opt_out_option_mode: MspaMode::NotApplicable,
                        mspa_service_provider_mode: MspaMode::NotApplicable,
                    },
                    gpc: None,
                },
            ),
            (
                "BVVVVVVY",
                UsCa {
                    core: Core {
                        sale_opt_out_notice: Notice::Provided,
                        sharing_opt_out_notice: Notice::Provided,
                        sensitive_data_limit_use_notice: Notice::Provided,
                        sale_opt_out: OptOut::OptedOut,
                        sharing_opt_out: OptOut::OptedOut,
                        sensitive_data_processing: SensitiveDataProcessing {
                            identification_documents: OptOut::OptedOut,
                            financial_data: OptOut::OptedOut,
                            precise_geolocation: OptOut::OptedOut,
                            origin_beliefs_or_union: OptOut::OptedOut,
                            mail_email_or_text_messages: OptOut::OptedOut,
                            genetic_data: OptOut::OptedOut,
                            biometric_unique_identification: OptOut::OptedOut,
                            health_data: OptOut::OptedOut,
                            sex_life_or_sexual_orientation: OptOut::OptedOut,
                        },
                        known_child_sensitive_data_consents: KnownChildSensitiveDataConsents {
                            sell_personal_information: Consent::NoConsent,
                            share_personal_information: Consent::NoConsent,
                        },
                        personal_data_consent: Consent::NoConsent,
                        mspa_covered_transaction: true,
                        mspa_opt_out_option_mode: MspaMode::Yes,
                        mspa_service_provider_mode: MspaMode::No,
                    },
                    gpc: None,
                },
            ),
            (
                "BVqqqqpY.YA",
                UsCa {
                    core: Core {
                        sale_opt_out_notice: Notice::Provided,
                        sharing_opt_out_notice: Notice::Provided,
                        sensitive_data_limit_use_notice: Notice::Provided,
                        sale_opt_out: OptOut::DidNotOptOut,
                        sharing_opt_out: OptOut::DidNotOptOut,
                        sensitive_data_processing: SensitiveDataProcessing {
                            identification_documents: OptOut::DidNotOptOut,
                            financial_data: OptOut::DidNotOptOut,
                            precise_geolocation: OptOut::DidNotOptOut,
                            origin_beliefs_or_union: OptOut::DidNotOptOut,
                            mail_email_or_text_messages: OptOut::DidNotOptOut,
                            genetic_data: OptOut::DidNotOptOut,
                            biometric_unique_identification: OptOut::DidNotOptOut,
                            health_data: OptOut::DidNotOptOut,
                            sex_life_or_sexual_orientation: OptOut::DidNotOptOut,
                        },
                        known_child_sensitive_data_consents: KnownChildSensitiveDataConsents {
                            sell_personal_information: Consent::Consent,
                            share_personal_information: Consent::Consent,
                        },
                        personal_data_consent: Consent::Consent,
                        mspa_covered_transaction: true,
                        mspa_opt_out_option_mode: MspaMode::Yes,
                        mspa_service_provider_mode: MspaMode::No,
                    },
                    gpc: Some(true),
                },
            ),
        ];

        for (s, expected) in test_cases {
            let actual = UsCa::from_str(s).unwrap();
            assert_eq!(actual, expected);
            assert!(actual.validate().is_ok());
        }
    }

    #[test_case("" => matches SectionDecodeError::Read(_) ; "empty string")]
    #[test_case("123" => matches SectionDecodeError::InvalidSectionVersion { .. } ; "decode error")]
    #[test_case("CVVVVVVVVWA.YA" => matches SectionDecodeError::InvalidSectionVersion { .. } ; "invalid section version")]
    #[test_case("BVVVVVVVVWA.AA" => matches SectionDecodeError::UnknownSegmentType { .. } ; "unknown segment version")]
    fn error(s: &str) -> SectionDecodeError {
        UsCa::from_str(s).unwrap_err()
    }
}

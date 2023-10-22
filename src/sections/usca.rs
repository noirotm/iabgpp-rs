use crate::core::{DataReader, FromDataReader};
use crate::sections::{OptionalSegmentParser, SectionDecodeError, SegmentedStr};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use std::str::FromStr;

const US_CA_VERSION: u8 = 1;
const US_CA_GPC_SEGMENT_TYPE: u8 = 1;

#[derive(Debug, Eq, PartialEq)]
pub struct UsCa {
    pub core: Core,
    pub gpc: Option<bool>,
}

impl UsCa {
    /// Checks the consistency of values in the already populated fields.
    ///
    /// This is based on the code found in https://iabgpp.com/js/cmpapi/encoder/segment/UsCaV1CoreSegment.js.
    ///
    /// While CMPs shouldn't be able to generate invalid combinations, the binary wire format
    /// does not prevent it.
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = vec![];

        if !Self::is_notice_and_optout_combination_ok(
            &self.core.sharing_optout_notice,
            &self.core.sharing_optout,
        ) {
            errors.push(ValidationError::new(
                "sharing_optout_notice",
                &self.core.sharing_optout_notice,
                "sharing_optout",
                &self.core.sharing_optout,
            ));
        }

        if !Self::is_notice_and_optout_combination_ok(
            &self.core.sale_optout_notice,
            &self.core.sale_optout,
        ) {
            errors.push(ValidationError::new(
                "sale_optout_notice",
                &self.core.sale_optout_notice,
                "sale_optout",
                &self.core.sale_optout,
            ));
        }

        if self.core.mspa_service_provider_mode == MspaMode::NotApplicable {
            if self.core.sale_optout_notice != Notice::NotApplicable {
                errors.push(ValidationError::new(
                    "mspa_service_provider_mode",
                    &self.core.mspa_service_provider_mode,
                    "sale_optout_notice",
                    &self.core.sale_optout_notice,
                ));
            }
            if self.core.sharing_optout_notice != Notice::NotApplicable {
                errors.push(ValidationError::new(
                    "mspa_service_provider_mode",
                    &self.core.mspa_service_provider_mode,
                    "sharing_optout_notice",
                    &self.core.sharing_optout_notice,
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
            if self.core.sale_optout_notice != Notice::NotApplicable {
                errors.push(ValidationError::new(
                    "mspa_service_provider_mode",
                    &self.core.mspa_service_provider_mode,
                    "sale_optout_notice",
                    &self.core.sale_optout_notice,
                ));
            }
            if self.core.sharing_optout_notice != Notice::NotApplicable {
                errors.push(ValidationError::new(
                    "mspa_service_provider_mode",
                    &self.core.mspa_service_provider_mode,
                    "sharing_optout_notice",
                    &self.core.sharing_optout_notice,
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

    fn is_notice_and_optout_combination_ok(notice: &Notice, optout: &Optout) -> bool {
        *notice == Notice::NotApplicable && *optout == Optout::NotApplicable
            || *notice == Notice::Provided && *optout != Optout::NotApplicable
            || *notice == Notice::NotProvided && *optout == Optout::OptedOut
    }
}

pub struct ValidationError {
    pub field1: (&'static str, u8),
    pub field2: (&'static str, u8),
}

impl ValidationError {
    fn new<T1, T2>(field1: &'static str, val1: &T1, field2: &'static str, val2: &T2) -> Self
    where
        T1: ToPrimitive,
        T2: ToPrimitive,
    {
        Self {
            field1: (field1, val1.to_u8().unwrap_or_default()),
            field2: (field2, val2.to_u8().unwrap_or_default()),
        }
    }
}

impl FromStr for UsCa {
    type Err = SectionDecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse_segmented_str()
    }
}

impl FromDataReader for UsCa {
    type Err = SectionDecodeError;

    fn from_data_reader(r: &mut DataReader) -> Result<Self, Self::Err> {
        Ok(Self {
            core: r.parse()?,
            gpc: None,
        })
    }
}

impl OptionalSegmentParser for UsCa {
    fn read_segment_type(r: &mut DataReader) -> Result<u8, SectionDecodeError> {
        Ok(r.read_fixed_integer::<u8>(2)?)
    }

    fn parse_optional_segment(
        segment_type: u8,
        r: &mut DataReader,
        into: &mut Self,
    ) -> Result<(), SectionDecodeError> {
        match segment_type {
            US_CA_GPC_SEGMENT_TYPE => {
                into.gpc = Some(r.read_bool()?);
            }
            n => {
                return Err(SectionDecodeError::UnknownSegmentType { segment_type: n });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Eq, PartialEq)]
/// The core sub-section must always be present. Where terms are capitalized in the ‘description’
/// field they are defined terms in Cal. Civ. Code 1798.140.
pub struct Core {
    pub sale_optout_notice: Notice,
    pub sharing_optout_notice: Notice,
    pub sensitive_data_limit_use_notice: Notice,
    pub sale_optout: Optout,
    pub sharing_optout: Optout,
    pub sensitive_data_processing: SensitiveDataProcessing,
    pub known_child_sensitive_data_consents: KnownChildSensitiveDataConsents,
    pub personal_data_consent: Consent,
    pub mspa_covered_transaction: bool,
    pub mspa_opt_out_option_mode: MspaMode,
    pub mspa_service_provider_mode: MspaMode,
}

impl FromDataReader for Core {
    type Err = SectionDecodeError;

    fn from_data_reader(r: &mut DataReader) -> Result<Self, Self::Err> {
        let version = r.read_fixed_integer::<u8>(6)?;
        if version != US_CA_VERSION {
            return Err(SectionDecodeError::InvalidSegmentVersion {
                expected: US_CA_VERSION,
                found: version,
            });
        }

        Ok(Self {
            sale_optout_notice: Notice::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Notice::NotApplicable),
            sharing_optout_notice: Notice::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Notice::NotApplicable),
            sensitive_data_limit_use_notice: Notice::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Notice::NotApplicable),
            sale_optout: Optout::from_u8(r.read_fixed_integer(2)?).unwrap_or(Optout::NotApplicable),
            sharing_optout: Optout::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Optout::NotApplicable),
            sensitive_data_processing: r.parse()?,
            known_child_sensitive_data_consents: r.parse()?,
            personal_data_consent: Consent::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Consent::NotApplicable),
            mspa_covered_transaction: mspa_covered_transaction_to_bool(r.read_fixed_integer(2)?)?,
            mspa_opt_out_option_mode: MspaMode::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(MspaMode::NotApplicable),
            mspa_service_provider_mode: MspaMode::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(MspaMode::NotApplicable),
        })
    }
}

fn mspa_covered_transaction_to_bool(val: u8) -> Result<bool, SectionDecodeError> {
    match val {
        1 => Ok(true),
        2 => Ok(false),
        v => Err(SectionDecodeError::InvalidFieldValue {
            expected: "1 or 2".to_string(),
            found: v.to_string(),
        }),
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct SensitiveDataProcessing {
    /// Opt-Out of the Use or Disclosure of the Consumer's Sensitive Personal Information Which
    /// Reveals a Consumer's Social Security, Driver's License, State Identification Card, or
    /// Passport Number.
    pub identification_documents: Optout,
    /// Opt-Out of the Use or Disclosure of the Consumer's Sensitive Personal Information Which
    /// Reveals a Consumer's Account Log-In, Financial Account, Debit Card, or Credit Card Number in
    /// Combination with Any Required Security or Access Code, Password, or Credentials Allowing
    /// Access to an Account.
    pub financial_data: Optout,
    pub precise_geolocation: Optout,
    pub origin_beliefs_or_union: Optout,
    pub mail_email_or_text_messages: Optout,
    pub genetic_data: Optout,
    pub biometric_unique_identification: Optout,
    pub health_data: Optout,
    pub sex_life_or_sexual_orientation: Optout,
}

impl FromDataReader for SensitiveDataProcessing {
    type Err = SectionDecodeError;

    fn from_data_reader(r: &mut DataReader) -> Result<Self, Self::Err> {
        Ok(Self {
            identification_documents: Optout::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or_else(|| unreachable!()),
            financial_data: Optout::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Optout::NotApplicable),
            precise_geolocation: Optout::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Optout::NotApplicable),
            origin_beliefs_or_union: Optout::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Optout::NotApplicable),
            mail_email_or_text_messages: Optout::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Optout::NotApplicable),
            genetic_data: Optout::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Optout::NotApplicable),
            biometric_unique_identification: Optout::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Optout::NotApplicable),
            health_data: Optout::from_u8(r.read_fixed_integer(2)?).unwrap_or(Optout::NotApplicable),
            sex_life_or_sexual_orientation: Optout::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Optout::NotApplicable),
        })
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct KnownChildSensitiveDataConsents {
    pub sell_personal_information: Consent,
    pub share_personal_information: Consent,
}

impl FromDataReader for KnownChildSensitiveDataConsents {
    type Err = SectionDecodeError;

    fn from_data_reader(r: &mut DataReader) -> Result<Self, Self::Err> {
        Ok(Self {
            sell_personal_information: Consent::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Consent::NotApplicable),
            share_personal_information: Consent::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Consent::NotApplicable),
        })
    }
}

#[derive(Debug, Eq, PartialEq, FromPrimitive, ToPrimitive)]
pub enum Notice {
    NotApplicable = 0,
    Provided = 1,
    NotProvided = 2,
}

#[derive(Debug, Eq, PartialEq, FromPrimitive, ToPrimitive)]
pub enum Optout {
    NotApplicable = 0,
    OptedOut = 1,
    DidNotOptOut = 2,
}

#[derive(Debug, Eq, PartialEq, FromPrimitive, ToPrimitive)]
pub enum Consent {
    NotApplicable = 0,
    NoConsent = 1,
    Consent = 2,
}

#[derive(Debug, Eq, PartialEq, FromPrimitive, ToPrimitive)]
pub enum MspaMode {
    NotApplicable = 0,
    Yes = 1,
    No = 2,
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test]
    fn parse() {
        let test_cases = [
            (
                "BAAAAACA",
                UsCa {
                    core: Core {
                        sale_optout_notice: Notice::NotApplicable,
                        sharing_optout_notice: Notice::NotApplicable,
                        sensitive_data_limit_use_notice: Notice::NotApplicable,
                        sale_optout: Optout::NotApplicable,
                        sharing_optout: Optout::NotApplicable,
                        sensitive_data_processing: SensitiveDataProcessing {
                            identification_documents: Optout::NotApplicable,
                            financial_data: Optout::NotApplicable,
                            precise_geolocation: Optout::NotApplicable,
                            origin_beliefs_or_union: Optout::NotApplicable,
                            mail_email_or_text_messages: Optout::NotApplicable,
                            genetic_data: Optout::NotApplicable,
                            biometric_unique_identification: Optout::NotApplicable,
                            health_data: Optout::NotApplicable,
                            sex_life_or_sexual_orientation: Optout::NotApplicable,
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
                        sale_optout_notice: Notice::Provided,
                        sharing_optout_notice: Notice::Provided,
                        sensitive_data_limit_use_notice: Notice::Provided,
                        sale_optout: Optout::OptedOut,
                        sharing_optout: Optout::OptedOut,
                        sensitive_data_processing: SensitiveDataProcessing {
                            identification_documents: Optout::OptedOut,
                            financial_data: Optout::OptedOut,
                            precise_geolocation: Optout::OptedOut,
                            origin_beliefs_or_union: Optout::OptedOut,
                            mail_email_or_text_messages: Optout::OptedOut,
                            genetic_data: Optout::OptedOut,
                            biometric_unique_identification: Optout::OptedOut,
                            health_data: Optout::OptedOut,
                            sex_life_or_sexual_orientation: Optout::OptedOut,
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
                        sale_optout_notice: Notice::Provided,
                        sharing_optout_notice: Notice::Provided,
                        sensitive_data_limit_use_notice: Notice::Provided,
                        sale_optout: Optout::DidNotOptOut,
                        sharing_optout: Optout::DidNotOptOut,
                        sensitive_data_processing: SensitiveDataProcessing {
                            identification_documents: Optout::DidNotOptOut,
                            financial_data: Optout::DidNotOptOut,
                            precise_geolocation: Optout::DidNotOptOut,
                            origin_beliefs_or_union: Optout::DidNotOptOut,
                            mail_email_or_text_messages: Optout::DidNotOptOut,
                            genetic_data: Optout::DidNotOptOut,
                            biometric_unique_identification: Optout::DidNotOptOut,
                            health_data: Optout::DidNotOptOut,
                            sex_life_or_sexual_orientation: Optout::DidNotOptOut,
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
    #[test_case("123" => matches SectionDecodeError::DecodeSegment(_) ; "decode error")]
    #[test_case("CVVVVVVVVWA.YA" => matches SectionDecodeError::InvalidSegmentVersion { .. } ; "invalid segment version")]
    #[test_case("BVVVVVVVVWA.AA" => matches SectionDecodeError::UnknownSegmentType { .. } ; "unknown segment version")]
    fn error(s: &str) -> SectionDecodeError {
        UsCa::from_str(s).unwrap_err()
    }
}

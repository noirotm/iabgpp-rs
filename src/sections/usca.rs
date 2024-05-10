use crate::core::{DataReader, FromDataReader};
use crate::sections::{
    DecodableSection, OptionalSegmentParser, SectionDecodeError, SectionId, SegmentedStr,
};
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
    /// This is based on the code found in https://iabgpp.com/js/3.2.0/cmpapi/encoder/segment/UsCaV1CoreSegment.js.
    ///
    /// While CMPs shouldn't be able to generate invalid combinations, the binary wire format
    /// does not prevent it.
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = vec![];

        if !Self::is_notice_and_opt_out_combination_ok(
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

        if !Self::is_notice_and_opt_out_combination_ok(
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

    fn is_notice_and_opt_out_combination_ok(notice: &Notice, opt_out: &OptOut) -> bool {
        *notice == Notice::NotApplicable && *opt_out == OptOut::NotApplicable
            || *notice == Notice::Provided && *opt_out != OptOut::NotApplicable
            || *notice == Notice::NotProvided && *opt_out == OptOut::OptedOut
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

impl DecodableSection for UsCa {
    const ID: SectionId = SectionId::UsCa;
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
        Ok(r.read_fixed_integer(2)?)
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
    pub sale_opt_out_notice: Notice,
    pub sharing_opt_out_notice: Notice,
    pub sensitive_data_limit_use_notice: Notice,
    pub sale_opt_out: OptOut,
    pub sharing_opt_out: OptOut,
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
        let version = r.read_fixed_integer(6)?;
        if version != US_CA_VERSION {
            return Err(SectionDecodeError::InvalidSegmentVersion {
                expected: US_CA_VERSION,
                found: version,
            });
        }

        Ok(Self {
            sale_opt_out_notice: Notice::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Notice::NotApplicable),
            sharing_opt_out_notice: Notice::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Notice::NotApplicable),
            sensitive_data_limit_use_notice: Notice::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Notice::NotApplicable),
            sale_opt_out: OptOut::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(OptOut::NotApplicable),
            sharing_opt_out: OptOut::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(OptOut::NotApplicable),
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

impl FromDataReader for SensitiveDataProcessing {
    type Err = SectionDecodeError;

    fn from_data_reader(r: &mut DataReader) -> Result<Self, Self::Err> {
        Ok(Self {
            identification_documents: OptOut::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or_else(|| unreachable!()),
            financial_data: OptOut::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(OptOut::NotApplicable),
            precise_geolocation: OptOut::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(OptOut::NotApplicable),
            origin_beliefs_or_union: OptOut::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(OptOut::NotApplicable),
            mail_email_or_text_messages: OptOut::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(OptOut::NotApplicable),
            genetic_data: OptOut::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(OptOut::NotApplicable),
            biometric_unique_identification: OptOut::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(OptOut::NotApplicable),
            health_data: OptOut::from_u8(r.read_fixed_integer(2)?).unwrap_or(OptOut::NotApplicable),
            sex_life_or_sexual_orientation: OptOut::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(OptOut::NotApplicable),
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
pub enum OptOut {
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
    #[test_case("123" => matches SectionDecodeError::InvalidSegmentVersion { .. } ; "decode error")]
    #[test_case("CVVVVVVVVWA.YA" => matches SectionDecodeError::InvalidSegmentVersion { .. } ; "invalid segment version")]
    #[test_case("BVVVVVVVVWA.AA" => matches SectionDecodeError::UnknownSegmentType { .. } ; "unknown segment version")]
    fn error(s: &str) -> SectionDecodeError {
        UsCa::from_str(s).unwrap_err()
    }
}

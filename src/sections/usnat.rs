use crate::core::{DataReader, FromDataReader};
use crate::sections::{OptionalSegmentParser, SectionDecodeError, SegmentedStr};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use std::str::FromStr;

const US_NAT_VERSION: u8 = 1;
const US_NAT_GPC_SEGMENT_TYPE: u8 = 1;

#[derive(Debug, Eq, PartialEq)]
pub struct UsNat {
    pub core: Core,
    pub gpc: Option<bool>,
}

impl UsNat {
    /// Checks the consistency of values in the already populated fields.
    ///
    /// This is based on the code found in https://iabgpp.com/js/cmpapi/encoder/segment/UsNatV1CoreSegment.js.
    ///
    /// While CMPs shouldn't be able to generate invalid combinations, the binary wire format
    /// does not prevent it.
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = vec![];

        if !Self::is_notice_and_optout_combination_ok(
            &self.core.sharing_notice,
            &self.core.sharing_optout,
        ) {
            errors.push(ValidationError::new(
                "sharing_notice",
                &self.core.sharing_notice,
                "sharing_optout",
                &self.core.sharing_optout,
            ));
        }

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

        if !Self::is_notice_and_optout_combination_ok(
            &self.core.targeted_advertising_optout_notice,
            &self.core.targeted_advertising_optout,
        ) {
            errors.push(ValidationError::new(
                "targeted_advertising_optout_notice",
                &self.core.targeted_advertising_optout_notice,
                "targeted_advertising_optout_optout",
                &self.core.targeted_advertising_optout,
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

impl FromStr for UsNat {
    type Err = SectionDecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse_segmented_str()
    }
}

impl FromDataReader for UsNat {
    type Err = SectionDecodeError;

    fn from_data_reader(r: &mut DataReader) -> Result<Self, Self::Err> {
        Ok(Self {
            core: r.parse()?,
            gpc: None,
        })
    }
}

impl OptionalSegmentParser for UsNat {
    fn read_segment_type(r: &mut DataReader) -> Result<u8, SectionDecodeError> {
        Ok(r.read_fixed_integer::<u8>(2)?)
    }

    fn parse_optional_segment(
        segment_type: u8,
        r: &mut DataReader,
        into: &mut Self,
    ) -> Result<(), SectionDecodeError> {
        match segment_type {
            US_NAT_GPC_SEGMENT_TYPE => {
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
pub struct Core {
    pub sharing_notice: Notice,
    pub sale_optout_notice: Notice,
    pub sharing_optout_notice: Notice,
    pub targeted_advertising_optout_notice: Notice,
    pub sensitive_data_processing_optout_notice: Notice,
    pub sensitive_data_limit_use_notice: Notice,
    pub sale_optout: Optout,
    pub sharing_optout: Optout,
    pub targeted_advertising_optout: Optout,
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
        if version != US_NAT_VERSION {
            return Err(SectionDecodeError::InvalidSegmentVersion {
                expected: US_NAT_VERSION,
                found: version,
            });
        }

        Ok(Self {
            sharing_notice: Notice::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Notice::NotApplicable),
            sale_optout_notice: Notice::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Notice::NotApplicable),
            sharing_optout_notice: Notice::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Notice::NotApplicable),
            targeted_advertising_optout_notice: Notice::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Notice::NotApplicable),
            sensitive_data_processing_optout_notice: Notice::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Notice::NotApplicable),
            sensitive_data_limit_use_notice: Notice::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Notice::NotApplicable),
            sale_optout: Optout::from_u8(r.read_fixed_integer(2)?).unwrap_or(Optout::NotApplicable),
            sharing_optout: Optout::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Optout::NotApplicable),
            targeted_advertising_optout: Optout::from_u8(r.read_fixed_integer(2)?)
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
    pub racial_or_ethnic_origin: Consent,
    pub religious_or_philosophical_beliefs: Consent,
    pub consumer_health: Consent,
    pub sex_life_or_sexual_orientation: Consent,
    pub citizenship_or_immigration_status: Consent,
    pub genetic_unique_identification: Consent,
    pub biometric_unique_identification: Consent,
    pub precise_geolocation_data: Consent,
    pub identification_documents: Consent,
    pub financial_data: Consent,
    pub union_membership: Consent,
    pub mail_email_or_text_messages: Consent,
}

impl FromDataReader for SensitiveDataProcessing {
    type Err = SectionDecodeError;

    fn from_data_reader(r: &mut DataReader) -> Result<Self, Self::Err> {
        Ok(Self {
            racial_or_ethnic_origin: Consent::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Consent::NotApplicable),
            religious_or_philosophical_beliefs: Consent::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Consent::NotApplicable),
            consumer_health: Consent::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Consent::NotApplicable),
            sex_life_or_sexual_orientation: Consent::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Consent::NotApplicable),
            citizenship_or_immigration_status: Consent::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Consent::NotApplicable),
            genetic_unique_identification: Consent::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Consent::NotApplicable),
            biometric_unique_identification: Consent::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Consent::NotApplicable),
            precise_geolocation_data: Consent::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Consent::NotApplicable),
            identification_documents: Consent::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Consent::NotApplicable),
            financial_data: Consent::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Consent::NotApplicable),
            union_membership: Consent::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Consent::NotApplicable),
            mail_email_or_text_messages: Consent::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Consent::NotApplicable),
        })
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct KnownChildSensitiveDataConsents {
    pub from_13_to_16: Consent,
    pub under_13: Consent,
}

impl FromDataReader for KnownChildSensitiveDataConsents {
    type Err = SectionDecodeError;

    fn from_data_reader(r: &mut DataReader) -> Result<Self, Self::Err> {
        Ok(Self {
            from_13_to_16: Consent::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Consent::NotApplicable),
            under_13: Consent::from_u8(r.read_fixed_integer(2)?).unwrap_or(Consent::NotApplicable),
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

    #[test]
    fn parse() {
        let test_cases = [
            (
                "BAAAAAAAAQA",
                UsNat {
                    core: Core {
                        sharing_notice: Notice::NotApplicable,
                        sale_optout_notice: Notice::NotApplicable,
                        sharing_optout_notice: Notice::NotApplicable,
                        targeted_advertising_optout_notice: Notice::NotApplicable,
                        sensitive_data_processing_optout_notice: Notice::NotApplicable,
                        sensitive_data_limit_use_notice: Notice::NotApplicable,
                        sale_optout: Optout::NotApplicable,
                        sharing_optout: Optout::NotApplicable,
                        targeted_advertising_optout: Optout::NotApplicable,
                        sensitive_data_processing: SensitiveDataProcessing {
                            racial_or_ethnic_origin: Consent::NotApplicable,
                            religious_or_philosophical_beliefs: Consent::NotApplicable,
                            consumer_health: Consent::NotApplicable,
                            sex_life_or_sexual_orientation: Consent::NotApplicable,
                            citizenship_or_immigration_status: Consent::NotApplicable,
                            genetic_unique_identification: Consent::NotApplicable,
                            biometric_unique_identification: Consent::NotApplicable,
                            precise_geolocation_data: Consent::NotApplicable,
                            identification_documents: Consent::NotApplicable,
                            financial_data: Consent::NotApplicable,
                            union_membership: Consent::NotApplicable,
                            mail_email_or_text_messages: Consent::NotApplicable,
                        },
                        known_child_sensitive_data_consents: KnownChildSensitiveDataConsents {
                            from_13_to_16: Consent::NotApplicable,
                            under_13: Consent::NotApplicable,
                        },
                        personal_data_consent: Consent::NotApplicable,
                        mspa_covered_transaction: true,
                        mspa_opt_out_option_mode: MspaMode::NotApplicable,
                        mspa_service_provider_mode: MspaMode::NotApplicable,
                    },
                    gpc: None,
                },
            ),
            (
                "BVVVVVVVVWA",
                UsNat {
                    core: Core {
                        sharing_notice: Notice::Provided,
                        sale_optout_notice: Notice::Provided,
                        sharing_optout_notice: Notice::Provided,
                        targeted_advertising_optout_notice: Notice::Provided,
                        sensitive_data_processing_optout_notice: Notice::Provided,
                        sensitive_data_limit_use_notice: Notice::Provided,
                        sale_optout: Optout::OptedOut,
                        sharing_optout: Optout::OptedOut,
                        targeted_advertising_optout: Optout::OptedOut,
                        sensitive_data_processing: SensitiveDataProcessing {
                            racial_or_ethnic_origin: Consent::NoConsent,
                            religious_or_philosophical_beliefs: Consent::NoConsent,
                            consumer_health: Consent::NoConsent,
                            sex_life_or_sexual_orientation: Consent::NoConsent,
                            citizenship_or_immigration_status: Consent::NoConsent,
                            genetic_unique_identification: Consent::NoConsent,
                            biometric_unique_identification: Consent::NoConsent,
                            precise_geolocation_data: Consent::NoConsent,
                            identification_documents: Consent::NoConsent,
                            financial_data: Consent::NoConsent,
                            union_membership: Consent::NoConsent,
                            mail_email_or_text_messages: Consent::NoConsent,
                        },
                        known_child_sensitive_data_consents: KnownChildSensitiveDataConsents {
                            from_13_to_16: Consent::NoConsent,
                            under_13: Consent::NoConsent,
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
                "BVVVVVVVVWA.YA",
                UsNat {
                    core: Core {
                        sharing_notice: Notice::Provided,
                        sale_optout_notice: Notice::Provided,
                        sharing_optout_notice: Notice::Provided,
                        targeted_advertising_optout_notice: Notice::Provided,
                        sensitive_data_processing_optout_notice: Notice::Provided,
                        sensitive_data_limit_use_notice: Notice::Provided,
                        sale_optout: Optout::OptedOut,
                        sharing_optout: Optout::OptedOut,
                        targeted_advertising_optout: Optout::OptedOut,
                        sensitive_data_processing: SensitiveDataProcessing {
                            racial_or_ethnic_origin: Consent::NoConsent,
                            religious_or_philosophical_beliefs: Consent::NoConsent,
                            consumer_health: Consent::NoConsent,
                            sex_life_or_sexual_orientation: Consent::NoConsent,
                            citizenship_or_immigration_status: Consent::NoConsent,
                            genetic_unique_identification: Consent::NoConsent,
                            biometric_unique_identification: Consent::NoConsent,
                            precise_geolocation_data: Consent::NoConsent,
                            identification_documents: Consent::NoConsent,
                            financial_data: Consent::NoConsent,
                            union_membership: Consent::NoConsent,
                            mail_email_or_text_messages: Consent::NoConsent,
                        },
                        known_child_sensitive_data_consents: KnownChildSensitiveDataConsents {
                            from_13_to_16: Consent::NoConsent,
                            under_13: Consent::NoConsent,
                        },
                        personal_data_consent: Consent::NoConsent,
                        mspa_covered_transaction: true,
                        mspa_opt_out_option_mode: MspaMode::Yes,
                        mspa_service_provider_mode: MspaMode::No,
                    },
                    gpc: Some(true),
                },
            ),
        ];

        for (s, expected) in test_cases {
            let actual = UsNat::from_str(s).unwrap();
            assert_eq!(actual, expected);
            assert!(actual.validate().is_ok());
        }
    }

    #[test]
    fn error() {
        assert!(matches!(
            UsNat::from_str("").unwrap_err(),
            SectionDecodeError::Read(_)
        ));

        assert!(matches!(
            UsNat::from_str("123").unwrap_err(),
            SectionDecodeError::DecodeSegment(_)
        ));

        assert!(matches!(
            UsNat::from_str("CVVVVVVVVWA.YA").unwrap_err(),
            SectionDecodeError::InvalidSegmentVersion { .. }
        ));

        assert!(matches!(
            dbg!(UsNat::from_str("BVVVVVVVVWA.AA").unwrap_err()),
            SectionDecodeError::UnknownSegmentType { .. }
        ));
    }
}

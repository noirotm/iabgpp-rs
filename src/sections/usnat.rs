use crate::core::{DataReader, FromDataReader};
use crate::sections::us_common::{
    is_notice_and_opt_out_combination_ok, mspa_covered_transaction_to_bool, Consent, MspaMode,
    Notice, OptOut, ValidationError,
};
use crate::sections::{
    DecodableSection, OptionalSegmentParser, SectionDecodeError, SectionId, SegmentedStr,
};
use num_traits::FromPrimitive;
use std::str::FromStr;

const US_NAT_VERSION: u8 = 1;
const US_NAT_GPC_SEGMENT_TYPE: u8 = 1;

#[derive(Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct UsNat {
    pub core: Core,
    pub gpc: Option<bool>,
}

impl UsNat {
    /// Checks the consistency of values in the already populated fields.
    ///
    /// This is based on the code found in <https://iabgpp.com/js/3.2.0/cmpapi/encoder/segment/UsNatV1CoreSegment.js>.
    ///
    /// While CMPs shouldn't be able to generate invalid combinations, the binary wire format
    /// does not prevent it.
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = vec![];

        if !is_notice_and_opt_out_combination_ok(
            &self.core.sharing_notice,
            &self.core.sharing_opt_out,
        ) {
            errors.push(ValidationError::new(
                "sharing_notice",
                &self.core.sharing_notice,
                "sharing_opt_out",
                &self.core.sharing_opt_out,
            ));
        }

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

        if !is_notice_and_opt_out_combination_ok(
            &self.core.targeted_advertising_opt_out_notice,
            &self.core.targeted_advertising_opt_out,
        ) {
            errors.push(ValidationError::new(
                "targeted_advertising_opt_out_notice",
                &self.core.targeted_advertising_opt_out_notice,
                "targeted_advertising_opt_out_opt_out",
                &self.core.targeted_advertising_opt_out,
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

impl DecodableSection for UsNat {
    const ID: SectionId = SectionId::UsNat;
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
        Ok(r.read_fixed_integer(2)?)
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
#[non_exhaustive]
pub struct Core {
    pub sharing_notice: Notice,
    pub sale_opt_out_notice: Notice,
    pub sharing_opt_out_notice: Notice,
    pub targeted_advertising_opt_out_notice: Notice,
    pub sensitive_data_processing_opt_out_notice: Notice,
    pub sensitive_data_limit_use_notice: Notice,
    pub sale_opt_out: OptOut,
    pub sharing_opt_out: OptOut,
    pub targeted_advertising_opt_out: OptOut,
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
        if version != US_NAT_VERSION {
            return Err(SectionDecodeError::InvalidSegmentVersion {
                expected: US_NAT_VERSION,
                found: version,
            });
        }

        Ok(Self {
            sharing_notice: Notice::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Notice::NotApplicable),
            sale_opt_out_notice: Notice::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Notice::NotApplicable),
            sharing_opt_out_notice: Notice::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Notice::NotApplicable),
            targeted_advertising_opt_out_notice: Notice::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Notice::NotApplicable),
            sensitive_data_processing_opt_out_notice: Notice::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Notice::NotApplicable),
            sensitive_data_limit_use_notice: Notice::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Notice::NotApplicable),
            sale_opt_out: OptOut::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(OptOut::NotApplicable),
            sharing_opt_out: OptOut::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(OptOut::NotApplicable),
            targeted_advertising_opt_out: OptOut::from_u8(r.read_fixed_integer(2)?)
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

#[derive(Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct SensitiveDataProcessing {
    pub racial_or_ethnic_origin: Consent,
    pub religious_or_philosophical_beliefs: Consent,
    pub health_data: Consent,
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
            health_data: Consent::from_u8(r.read_fixed_integer(2)?)
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
#[non_exhaustive]
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

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test]
    fn parse() {
        let test_cases = [
            (
                "BAAAAAAAAQA",
                UsNat {
                    core: Core {
                        sharing_notice: Notice::NotApplicable,
                        sale_opt_out_notice: Notice::NotApplicable,
                        sharing_opt_out_notice: Notice::NotApplicable,
                        targeted_advertising_opt_out_notice: Notice::NotApplicable,
                        sensitive_data_processing_opt_out_notice: Notice::NotApplicable,
                        sensitive_data_limit_use_notice: Notice::NotApplicable,
                        sale_opt_out: OptOut::NotApplicable,
                        sharing_opt_out: OptOut::NotApplicable,
                        targeted_advertising_opt_out: OptOut::NotApplicable,
                        sensitive_data_processing: SensitiveDataProcessing {
                            racial_or_ethnic_origin: Consent::NotApplicable,
                            religious_or_philosophical_beliefs: Consent::NotApplicable,
                            health_data: Consent::NotApplicable,
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
                        sale_opt_out_notice: Notice::Provided,
                        sharing_opt_out_notice: Notice::Provided,
                        targeted_advertising_opt_out_notice: Notice::Provided,
                        sensitive_data_processing_opt_out_notice: Notice::Provided,
                        sensitive_data_limit_use_notice: Notice::Provided,
                        sale_opt_out: OptOut::OptedOut,
                        sharing_opt_out: OptOut::OptedOut,
                        targeted_advertising_opt_out: OptOut::OptedOut,
                        sensitive_data_processing: SensitiveDataProcessing {
                            racial_or_ethnic_origin: Consent::NoConsent,
                            religious_or_philosophical_beliefs: Consent::NoConsent,
                            health_data: Consent::NoConsent,
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
                        sale_opt_out_notice: Notice::Provided,
                        sharing_opt_out_notice: Notice::Provided,
                        targeted_advertising_opt_out_notice: Notice::Provided,
                        sensitive_data_processing_opt_out_notice: Notice::Provided,
                        sensitive_data_limit_use_notice: Notice::Provided,
                        sale_opt_out: OptOut::OptedOut,
                        sharing_opt_out: OptOut::OptedOut,
                        targeted_advertising_opt_out: OptOut::OptedOut,
                        sensitive_data_processing: SensitiveDataProcessing {
                            racial_or_ethnic_origin: Consent::NoConsent,
                            religious_or_philosophical_beliefs: Consent::NoConsent,
                            health_data: Consent::NoConsent,
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

    #[test_case("" => matches SectionDecodeError::Read(_) ; "empty string")]
    #[test_case("123" => matches SectionDecodeError::InvalidSegmentVersion { .. } ; "decode error")]
    #[test_case("CVVVVVVVVWA.YA" => matches SectionDecodeError::InvalidSegmentVersion { .. } ; "invalid segment version 1")]
    #[test_case("gqgkgAAAAEA" => matches SectionDecodeError::InvalidSegmentVersion { .. } ; "invalid segment version 2")]
    #[test_case("BVVVVVVVVWA.AA" => matches SectionDecodeError::UnknownSegmentType { .. } ; "unknown segment type")]
    fn error(s: &str) -> SectionDecodeError {
        UsNat::from_str(s).unwrap_err()
    }
}

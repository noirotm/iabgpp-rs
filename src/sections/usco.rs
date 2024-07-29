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

const US_CO_VERSION: u8 = 1;
const US_CO_GPC_SEGMENT_TYPE: u8 = 1;

#[derive(Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct UsCo {
    pub core: Core,
    pub gpc: Option<bool>,
}

impl UsCo {
    /// Checks the consistency of values in the already populated fields.
    ///
    /// This is based on the code found in <https://iabgpp.com/js/3.2.0/cmpapi/encoder/segment/UsCoV1CoreSegment.js>.
    ///
    /// While CMPs shouldn't be able to generate invalid combinations, the binary wire format
    /// does not prevent it.
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = vec![];

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

impl DecodableSection for UsCo {
    const ID: SectionId = SectionId::UsCo;
}

impl FromStr for UsCo {
    type Err = SectionDecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse_segmented_str()
    }
}

impl FromDataReader for UsCo {
    type Err = SectionDecodeError;

    fn from_data_reader(r: &mut DataReader) -> Result<Self, Self::Err> {
        Ok(Self {
            core: r.parse()?,
            gpc: None,
        })
    }
}

impl OptionalSegmentParser for UsCo {
    fn read_segment_type(r: &mut DataReader) -> Result<u8, SectionDecodeError> {
        Ok(r.read_fixed_integer(2)?)
    }

    fn parse_optional_segment(
        segment_type: u8,
        r: &mut DataReader,
        into: &mut Self,
    ) -> Result<(), SectionDecodeError> {
        match segment_type {
            US_CO_GPC_SEGMENT_TYPE => {
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
    pub targeted_advertising_opt_out_notice: Notice,
    pub sale_opt_out: OptOut,
    pub targeted_advertising_opt_out: OptOut,
    pub sensitive_data_processing: SensitiveDataProcessing,
    pub known_child_sensitive_data_consents: Consent,
    pub mspa_covered_transaction: bool,
    pub mspa_opt_out_option_mode: MspaMode,
    pub mspa_service_provider_mode: MspaMode,
}

impl FromDataReader for Core {
    type Err = SectionDecodeError;

    fn from_data_reader(r: &mut DataReader) -> Result<Self, Self::Err> {
        let version = r.read_fixed_integer(6)?;
        if version != US_CO_VERSION {
            return Err(SectionDecodeError::InvalidSegmentVersion {
                expected: US_CO_VERSION,
                found: version,
            });
        }

        Ok(Self {
            sharing_notice: Notice::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Notice::NotApplicable),
            sale_opt_out_notice: Notice::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Notice::NotApplicable),
            targeted_advertising_opt_out_notice: Notice::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Notice::NotApplicable),
            sale_opt_out: OptOut::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(OptOut::NotApplicable),
            targeted_advertising_opt_out: OptOut::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(OptOut::NotApplicable),
            sensitive_data_processing: r.parse()?,
            known_child_sensitive_data_consents: Consent::from_u8(r.read_fixed_integer(2)?)
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
    pub religious_beliefs: Consent,
    pub health_condition_or_diagnosis: Consent,
    pub sex_life_or_sexual_orientation: Consent,
    pub citizenship_data: Consent,
    pub genetic_unique_identification: Consent,
    pub biometric_unique_identification: Consent,
}

impl FromDataReader for SensitiveDataProcessing {
    type Err = SectionDecodeError;

    fn from_data_reader(r: &mut DataReader) -> Result<Self, Self::Err> {
        Ok(Self {
            racial_or_ethnic_origin: Consent::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Consent::NotApplicable),
            religious_beliefs: Consent::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Consent::NotApplicable),
            health_condition_or_diagnosis: Consent::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Consent::NotApplicable),
            sex_life_or_sexual_orientation: Consent::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Consent::NotApplicable),
            citizenship_data: Consent::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Consent::NotApplicable),
            genetic_unique_identification: Consent::from_u8(r.read_fixed_integer(2)?)
                .unwrap_or(Consent::NotApplicable),
            biometric_unique_identification: Consent::from_u8(r.read_fixed_integer(2)?)
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

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test]
    fn parse() {
        let test_cases = [
            (
                "BAAAAEA",
                UsCo {
                    core: Core {
                        sharing_notice: Notice::NotApplicable,
                        sale_opt_out_notice: Notice::NotApplicable,
                        targeted_advertising_opt_out_notice: Notice::NotApplicable,
                        sale_opt_out: OptOut::NotApplicable,
                        targeted_advertising_opt_out: OptOut::NotApplicable,
                        sensitive_data_processing: SensitiveDataProcessing {
                            racial_or_ethnic_origin: Consent::NotApplicable,
                            religious_beliefs: Consent::NotApplicable,
                            health_condition_or_diagnosis: Consent::NotApplicable,
                            sex_life_or_sexual_orientation: Consent::NotApplicable,
                            citizenship_data: Consent::NotApplicable,
                            genetic_unique_identification: Consent::NotApplicable,
                            biometric_unique_identification: Consent::NotApplicable,
                        },
                        known_child_sensitive_data_consents: Consent::NotApplicable,
                        mspa_covered_transaction: true,
                        mspa_opt_out_option_mode: MspaMode::NotApplicable,
                        mspa_service_provider_mode: MspaMode::NotApplicable,
                    },
                    gpc: None,
                },
            ),
            (
                "BVVVVVg",
                UsCo {
                    core: Core {
                        sharing_notice: Notice::Provided,
                        sale_opt_out_notice: Notice::Provided,
                        targeted_advertising_opt_out_notice: Notice::Provided,
                        sale_opt_out: OptOut::OptedOut,
                        targeted_advertising_opt_out: OptOut::OptedOut,
                        sensitive_data_processing: SensitiveDataProcessing {
                            racial_or_ethnic_origin: Consent::NoConsent,
                            religious_beliefs: Consent::NoConsent,
                            health_condition_or_diagnosis: Consent::NoConsent,
                            sex_life_or_sexual_orientation: Consent::NoConsent,
                            citizenship_data: Consent::NoConsent,
                            genetic_unique_identification: Consent::NoConsent,
                            biometric_unique_identification: Consent::NoConsent,
                        },
                        known_child_sensitive_data_consents: Consent::NoConsent,
                        mspa_covered_transaction: true,
                        mspa_opt_out_option_mode: MspaMode::Yes,
                        mspa_service_provider_mode: MspaMode::No,
                    },
                    gpc: None,
                },
            ),
            (
                "BVVVVVg.YA",
                UsCo {
                    core: Core {
                        sharing_notice: Notice::Provided,
                        sale_opt_out_notice: Notice::Provided,
                        targeted_advertising_opt_out_notice: Notice::Provided,
                        sale_opt_out: OptOut::OptedOut,
                        targeted_advertising_opt_out: OptOut::OptedOut,
                        sensitive_data_processing: SensitiveDataProcessing {
                            racial_or_ethnic_origin: Consent::NoConsent,
                            religious_beliefs: Consent::NoConsent,
                            health_condition_or_diagnosis: Consent::NoConsent,
                            sex_life_or_sexual_orientation: Consent::NoConsent,
                            citizenship_data: Consent::NoConsent,
                            genetic_unique_identification: Consent::NoConsent,
                            biometric_unique_identification: Consent::NoConsent,
                        },
                        known_child_sensitive_data_consents: Consent::NoConsent,
                        mspa_covered_transaction: true,
                        mspa_opt_out_option_mode: MspaMode::Yes,
                        mspa_service_provider_mode: MspaMode::No,
                    },
                    gpc: Some(true),
                },
            ),
        ];

        for (s, expected) in test_cases {
            let actual = UsCo::from_str(s).unwrap();
            assert_eq!(actual, expected);
            assert!(actual.validate().is_ok());
        }
    }

    #[test_case("" => matches SectionDecodeError::Read(_) ; "empty string")]
    #[test_case("123" => matches SectionDecodeError::InvalidSegmentVersion { .. } ; "decode error")]
    #[test_case("CVVVVVg.YA" => matches SectionDecodeError::InvalidSegmentVersion { .. } ; "invalid segment version")]
    #[test_case("BVVVVVg.AA" => matches SectionDecodeError::UnknownSegmentType { .. } ; "unknown segment version")]
    fn error(s: &str) -> SectionDecodeError {
        UsCo::from_str(s).unwrap_err()
    }
}

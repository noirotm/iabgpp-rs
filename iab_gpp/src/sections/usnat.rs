use crate::sections::us_common::{
    Consent, MspaSupport, Notice, OptOut, parse_mspa_covered_transaction,
};
use iab_gpp_derive::{FromBitStream, GPPSection};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, GPPSection)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[non_exhaustive]
#[gpp(with_optional_segments(bits = 2))]
pub struct UsNat {
    pub core: Core,
    #[gpp(optional_segment_type = 1)]
    pub gpc: Option<bool>,
}

#[derive(Debug, Eq, PartialEq, FromBitStream)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[non_exhaustive]
pub enum Core {
    #[gpp(version = 1)]
    V1(CoreV1),
    #[gpp(version = 2)]
    V2(CoreV2),
}

#[derive(Debug, Eq, PartialEq, FromBitStream)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[non_exhaustive]
pub struct CoreV1 {
    pub sharing_notice: Notice,
    pub sale_opt_out_notice: Notice,
    pub sharing_opt_out_notice: Notice,
    pub targeted_advertising_opt_out_notice: Notice,
    pub sensitive_data_processing_opt_out_notice: Notice,
    pub sensitive_data_limit_use_notice: Notice,
    pub sale_opt_out: OptOut,
    pub sharing_opt_out: OptOut,
    pub targeted_advertising_opt_out: OptOut,
    pub sensitive_data_processing: SensitiveDataProcessingV1,
    pub known_child_sensitive_data_consents: KnownChildSensitiveDataConsentsV1,
    pub personal_data_consent: Consent,
    #[gpp(parse_with = parse_mspa_covered_transaction)]
    pub mspa_covered_transaction: bool,
    pub mspa_opt_out_option_mode: MspaSupport,
    pub mspa_service_provider_mode: MspaSupport,
}

#[derive(Debug, Eq, PartialEq, FromBitStream)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[non_exhaustive]
pub struct SensitiveDataProcessingV1 {
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

#[derive(Debug, Eq, PartialEq, FromBitStream)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[non_exhaustive]
pub struct KnownChildSensitiveDataConsentsV1 {
    pub from_13_to_16: Consent,
    pub under_13: Consent,
}

#[derive(Debug, Eq, PartialEq, FromBitStream)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[non_exhaustive]
pub struct CoreV2 {
    pub sharing_notice: Notice,
    pub sale_opt_out_notice: Notice,
    pub sharing_opt_out_notice: Notice,
    pub targeted_advertising_opt_out_notice: Notice,
    pub sensitive_data_processing_opt_out_notice: Notice,
    pub sensitive_data_limit_use_notice: Notice,
    pub sale_opt_out: OptOut,
    pub sharing_opt_out: OptOut,
    pub targeted_advertising_opt_out: OptOut,
    pub sensitive_data_processing: SensitiveDataProcessingV2,
    pub known_child_sensitive_data_consents: KnownChildSensitiveDataConsentsV2,
    pub personal_data_consent: Consent,
    #[gpp(parse_with = parse_mspa_covered_transaction)]
    pub mspa_covered_transaction: bool,
    pub mspa_opt_out_option_mode: MspaSupport,
    pub mspa_service_provider_mode: MspaSupport,
}

#[derive(Debug, Eq, PartialEq, FromBitStream)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[non_exhaustive]
pub struct SensitiveDataProcessingV2 {
    pub racial_or_ethnic_origin: Consent,
    pub religious_or_philosophical_beliefs: Consent,
    pub health_data: Consent,
    pub sex_life_or_sexual_orientation: Consent,
    pub citizenship_or_immigration_status: Consent,
    pub genetic_unique_identification: Consent,
    pub biometric_unique_identification: Consent,
    pub precise_geolocation_data: Consent,
    pub identification_documents: Consent,
    pub financial_account_data: Consent,
    pub union_membership: Consent,
    pub mail_email_or_text_messages: Consent,
    pub general_health_data: Consent,
    pub crime_victim_status: Consent,
    pub national_origin: Consent,
    pub transgender_or_nonbinary_status: Consent,
}

#[derive(Debug, Eq, PartialEq, FromBitStream)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[non_exhaustive]
pub struct KnownChildSensitiveDataConsentsV2 {
    pub process_sensitive_data_from_13_to_16: Consent,
    pub process_sensitive_data_under_13: Consent,
    pub process_personal_data_from_16_to_17: Consent,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sections::SectionDecodeError;
    use std::str::FromStr;
    use test_case::test_case;

    #[test_case("" => matches SectionDecodeError::Read { .. } ; "empty string")]
    #[test_case("123" => matches SectionDecodeError::UnknownSegmentVersion { .. } ; "decode error")]
    #[test_case("gqgkgAAAAEA" => matches SectionDecodeError::UnknownSegmentVersion { .. } ; "unknown segment version")]
    #[test_case("BVVVVVVVVWA.AA" => matches SectionDecodeError::UnknownSegmentType { .. } ; "unknown segment type")]
    fn error(s: &str) -> SectionDecodeError {
        UsNat::from_str(s).unwrap_err()
    }
}

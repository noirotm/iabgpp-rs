use crate::sections::us_common::{
    parse_mspa_covered_transaction, Consent, MspaMode, Notice, OptOut,
};
use iab_gpp_derive::{FromDataReader, GPPSection};
#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, Eq, PartialEq, GPPSection)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[non_exhaustive]
#[gpp(with_optional_segments(bits = 2))]
pub struct UsOr {
    pub core: Core,
    #[gpp(optional_segment_type = 1)]
    pub gpc: Option<bool>,
}

#[derive(Debug, Eq, PartialEq, FromDataReader)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[non_exhaustive]
#[gpp(section_version = 1)]
pub struct Core {
    pub processing_notice: Notice,
    pub sale_opt_out_notice: Notice,
    pub targeted_advertising_opt_out_notice: Notice,
    pub sale_opt_out: OptOut,
    pub targeted_advertising_opt_out: OptOut,
    pub sensitive_data_processing: SensitiveDataProcessing,
    pub known_child_sensitive_data_consents: KnownChildSensitiveDataConsents,
    pub additional_data_processing_consent: Consent,
    #[gpp(parse_with = parse_mspa_covered_transaction)]
    pub mspa_covered_transaction: bool,
    pub mspa_opt_out_option_mode: MspaMode,
    pub mspa_service_provider_mode: MspaMode,
}

#[derive(Debug, Eq, PartialEq, FromDataReader)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[non_exhaustive]
pub struct SensitiveDataProcessing {
    pub racial_or_ethnic_origin: Consent,
    pub religious_beliefs: Consent,
    pub health_data: Consent,
    pub sex_life_or_sexual_orientation: Consent,
    pub transgender_or_nonbinary_status: Consent,
    pub citizenship_or_immigration_status: Consent,
    pub national_origin: Consent,
    pub crime_victim_status: Consent,
    pub genetic_data: Consent,
    pub biometric_data: Consent,
    pub precise_geolocation_data: Consent,
}

#[derive(Debug, Eq, PartialEq, FromDataReader)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[non_exhaustive]
pub struct KnownChildSensitiveDataConsents {
    pub process_sensitive_data_from_known_child: Consent,
    pub sell_personal_data_from_13_to_16: Consent,
    pub process_personal_data_from_13_to_16: Consent,
}

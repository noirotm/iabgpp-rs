use crate::sections::us_common::{
    parse_mspa_covered_transaction, Consent, MspaMode, Notice, OptOut,
};
use iab_gpp_derive::{FromDataReader, GPPSection};

#[derive(Debug, Eq, PartialEq, GPPSection)]
#[non_exhaustive]
#[gpp(with_optional_segments(bits = 2))]
pub struct UsIa {
    pub core: Core,
    #[gpp(optional_segment_type = 1)]
    pub gpc: Option<bool>,
}

#[derive(Debug, Eq, PartialEq, FromDataReader)]
#[non_exhaustive]
#[gpp(section_version = 1)]
pub struct Core {
    pub processing_notice: Notice,
    pub sale_opt_out_notice: Notice,
    pub targeted_advertising_opt_out_notice: Notice,
    pub sensitive_data_optout_notice: Notice,
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
    pub religious_beliefs: Consent,
    pub health_data: Consent,
    pub sexual_orientation: Consent,
    pub citizenship_status: Consent,
    pub genetic_unique_identification: Consent,
    pub biometric_unique_identification: Consent,
    pub precise_geolocation_data: Consent,
}

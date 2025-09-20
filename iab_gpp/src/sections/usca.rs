use crate::sections::us_common::{
    Consent, MspaMode, Notice, OptOut, parse_mspa_covered_transaction,
};
use iab_gpp_derive::{FromBitStream, GPPSection};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, GPPSection)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[non_exhaustive]
#[gpp(with_optional_segments(bits = 2))]
pub struct UsCa {
    pub core: Core,
    #[gpp(optional_segment_type = 1)]
    pub gpc: Option<bool>,
}

#[derive(Debug, Eq, PartialEq, FromBitStream)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

#[derive(Debug, Eq, PartialEq, FromBitStream)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

#[derive(Debug, Eq, PartialEq, FromBitStream)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

    #[test_case("" => matches SectionDecodeError::Read { .. } ; "empty string")]
    #[test_case("123" => matches SectionDecodeError::UnknownSegmentVersion { .. } ; "decode error")]
    #[test_case("CVVVVVVVVWA.YA" => matches SectionDecodeError::UnknownSegmentVersion { .. } ; "unknown segment version")]
    #[test_case("BVVVVVVVVWA.AA" => matches SectionDecodeError::UnknownSegmentType { .. } ; "unknown segment type")]
    fn error(s: &str) -> SectionDecodeError {
        UsCa::from_str(s).unwrap_err()
    }
}

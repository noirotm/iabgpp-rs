//! This crate is an implementation of the IAB Global Privacy Platform (GPP)
//! [Consent String Specification](https://github.com/InteractiveAdvertisingBureau/Global-Privacy-Platform).
//!
//! At the moment, it has the ability to parse all sections supported by version 1.0 of
//! the standard.
//!
//! NOTE: This is not an official IAB library.
//!
//! # Parsing GPP strings
//!
//! A GPP Consent String is made of a mandatory header and a list of optional sections.
//!
//! The [`GPPString`](v1/struct.GPPString.html) type is used to parse consent strings and decode
//! sections.
//!
//! ```
//! # use std::error::Error;
//! #
//! # fn main() -> Result<(), Box<dyn Error>> {
//! use std::str::FromStr;
//! use iab_gpp::v1::GPPString;
//!
//! let s = "DBACNY~CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA~1YNN";
//! let gpp_string = GPPString::from_str(s)?;
//!
//! // Individual sections can be then be accessed:
//! for &id in gpp_string.section_ids() {
//!     let section_str = gpp_string.section(id).ok_or("missing section")?;
//!     println!("{section_str}");
//!
//!     let section = gpp_string.decode_section(id)?;
//!     println!("Section: {:?}", &section);
//! }
//!
//! // All sections can be decoded at once as well:
//! let sections = gpp_string.decode_all_sections();
//! # Ok(())
//! # }
//! ```
//!
//! # Accessing section data
//!
//! Depending on the legislation which applies with regard to the data you are handling, you may
//! want to decode and analyze only specific sections.
//!
//! For example, let's assume your users are located in the European Union, where GDPR rules apply.
//! The section that you need to extract and decode would be TCF V2.2.
//!
//! The following example checks that a specific vendor (id 755) has the right to create a
//! personalized ads profile (purpose ID 3) for the user who submitted the provided consent string.
//!
//! ```
//! # use std::error::Error;
//! #
//! # fn main() -> Result<(), Box<dyn Error>> {
//! use std::str::FromStr;
//! use iab_gpp::sections::{Section, SectionId};
//! use iab_gpp::sections::tcfeuv2::TcfEuV2;
//! use iab_gpp::v1::GPPString;
//!
//! let s = "DBABMA~CPXuQIAPXuQIAAfKABENB-CgACAAAAAAAAYgF5wAQF5gAAAA.YAAAAAAAAAAA";
//! let gpp = GPPString::from_str(s)?;
//!
//! let has_user_consent = gpp.decode::<TcfEuV2>().map(|tcf| {
//!     // does the user consent to the vendor creating a personalized ads profile
//!     // based on their data?
//!     let personalized_ads_profile_consent = tcf.core.purpose_consents.contains(&3);
//!
//!     // does the user consent to vendor Google Advertising Products to use their data?
//!     let vendor_consent = tcf.core.vendor_consents.contains(&755);
//!
//!     personalized_ads_profile_consent && vendor_consent
//! }).unwrap_or(false);
//!
//! assert!(has_user_consent);
//! # Ok(())
//! # }
//! ```
//!
//! # Error handling
//!
//! This crate is conservative with regard to how it handles parsing failure. If a string cannot be
//! fully decoded, then it is considered as an error.
//!
//! This is done to avoid obtaining erroneous user consent information from potentially corrupted
//! payloads.
//!
pub(crate) mod core;
pub mod sections;
pub mod v1;

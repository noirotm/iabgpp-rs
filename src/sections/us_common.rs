use crate::core::DataReader;
use crate::sections::SectionDecodeError;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::ToPrimitive;

pub struct ValidationError {
    pub field1: (&'static str, u8),
    pub field2: (&'static str, u8),
}

impl ValidationError {
    pub(crate) fn new<T1, T2>(
        field1: &'static str,
        val1: &T1,
        field2: &'static str,
        val2: &T2,
    ) -> Self
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

pub(crate) fn is_notice_and_opt_out_combination_ok(notice: &Notice, opt_out: &OptOut) -> bool {
    *notice == Notice::NotApplicable && *opt_out == OptOut::NotApplicable
        || *notice == Notice::Provided && *opt_out != OptOut::NotApplicable
        || *notice == Notice::NotProvided && *opt_out == OptOut::OptedOut
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

pub(crate) fn mspa_covered_transaction_to_bool(
    r: &mut DataReader,
) -> Result<bool, SectionDecodeError> {
    let val = r.read_fixed_integer(2)?;
    match val {
        1 => Ok(true),
        2 => Ok(false),
        v => Err(SectionDecodeError::InvalidFieldValue {
            expected: "1 or 2".to_string(),
            found: v.to_string(),
        }),
    }
}

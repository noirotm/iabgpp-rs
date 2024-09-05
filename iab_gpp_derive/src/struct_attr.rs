use crate::find_gpp_attr;
use syn::{parse, Attribute, LitInt};

pub enum GPPStructKind {
    Base64Data,
    WithOptionalSegments,
}

pub struct GPPStructHelperAttribute {
    pub kind: GPPStructKind,
    pub section_version: Option<u8>,
}

impl GPPStructHelperAttribute {
    pub fn new(attrs: &[Attribute]) -> parse::Result<Self> {
        let mut gpp_attr = Self {
            kind: GPPStructKind::Base64Data,
            section_version: None,
        };

        if let Some(attr) = find_gpp_attr(attrs) {
            attr.parse_nested_meta(|meta| {
                // #[gpp(with_optional_segments)]
                if meta.path.is_ident("with_optional_segments") {
                    gpp_attr.kind = GPPStructKind::WithOptionalSegments;
                    return Ok(());
                }

                // #[gpp(section_version = N)]
                if meta.path.is_ident("section_version") {
                    let value = meta.value()?; // parses the `=`
                    let s = value.parse::<LitInt>()?;
                    gpp_attr.section_version = Some(s.base10_parse()?);
                    return Ok(());
                }

                Err(meta.error("unrecognized gpp struct parameter"))
            })?;
        }

        Ok(gpp_attr)
    }
}

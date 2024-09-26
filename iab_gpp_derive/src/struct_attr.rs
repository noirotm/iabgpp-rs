use crate::find_gpp_attr;
use syn::{parse, token, Attribute, LitInt};

pub enum GPPStructKind {
    Base64Data,
    WithOptionalSegments(u32),
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
                // #[gpp(with_optional_segments(bits = N)]
                if meta.path.is_ident("with_optional_segments") {
                    // default value is 3 bits (as seen in TCF EU & CA)
                    let mut bits = 3;

                    if meta.input.peek(token::Paren) {
                        meta.parse_nested_meta(|meta| {
                            if meta.path.is_ident("bits") {
                                let value = meta.value()?; // parses the `=`
                                let s = value.parse::<LitInt>()?;
                                bits = s.base10_parse()?;

                                return Ok(());
                            }

                            Err(meta.error("unrecognized with_optional_segments parameter"))
                        })?;
                    }

                    gpp_attr.kind = GPPStructKind::WithOptionalSegments(bits);

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

use crate::find_gpp_attr;
use syn::{parse, Attribute};

pub enum GPPStructKind {
    Base64Data,
    WithOptionalSegments,
}

pub struct GPPStructHelperAttribute {
    pub kind: GPPStructKind,
}

impl GPPStructHelperAttribute {
    pub fn new(attrs: &[Attribute]) -> parse::Result<Self> {
        let mut gpp_attr = Self {
            kind: GPPStructKind::Base64Data,
        };

        if let Some(attr) = find_gpp_attr(attrs) {
            attr.parse_nested_meta(|meta| {
                // #[gpp(with_optional_segments)]
                if meta.path.is_ident("with_optional_segments") {
                    gpp_attr.kind = GPPStructKind::WithOptionalSegments;
                    return Ok(());
                }

                Err(meta.error("unrecognized gpp parameter"))
            })?;
        }

        Ok(gpp_attr)
    }
}

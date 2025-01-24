use crate::find_gpp_attr;
use syn::{parse, Attribute, LitInt};

pub struct GPPEnumVariantHelperAttribute {
    pub variant_version: Option<u8>,
}

impl GPPEnumVariantHelperAttribute {
    pub fn new(attrs: &[Attribute]) -> parse::Result<Self> {
        let mut gpp_attr = Self {
            variant_version: None,
        };

        if let Some(attr) = find_gpp_attr(attrs) {
            attr.parse_nested_meta(|meta| {
                // #[gpp(version = N)]
                if meta.path.is_ident("version") {
                    let value = meta.value()?; // parses the `=`
                    let s = value.parse::<LitInt>()?;
                    gpp_attr.variant_version = Some(s.base10_parse()?);

                    return Ok(());
                }

                Err(meta.error("unrecognized gpp enum parameter"))
            })?;
        }

        Ok(gpp_attr)
    }
}

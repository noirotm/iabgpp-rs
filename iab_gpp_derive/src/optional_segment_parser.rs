use crate::field_attr::GPPFieldHelperAttribute;
use crate::struct_attr::{GPPStructHelperAttribute, GPPStructKind};
use proc_macro2::Ident;
use quote::quote;
use syn::{DataStruct, Visibility};

pub fn derive_optional_segment_parser(
    input: &DataStruct,
    ident: &Ident,
    struct_attr: &GPPStructHelperAttribute,
) -> proc_macro2::TokenStream {
    let mut parse_match_arms = vec![];

    for field in &input.fields {
        let name = field.ident.clone();

        // ignore nameless fields
        if name.is_none() {
            continue;
        }
        // ignore non-public fields
        if !matches!(field.vis, Visibility::Public(_)) {
            continue;
        }
        let name = name.unwrap();

        let attr = GPPFieldHelperAttribute::new(&field.attrs, &field.ty)
            .expect("attribute parsing failed");

        if let Some(segment_type) = attr.optional_segment_type {
            let expr = attr.parser.to_token_stream();
            parse_match_arms.push(quote! {
                #segment_type => {
                    into.#name = Some(#expr?);
                }
            });
        }
    }

    let read_segment_type_override = match struct_attr.kind {
        GPPStructKind::WithOptionalSegments(3) => None,
        GPPStructKind::WithOptionalSegments(n) => Some(quote! {
            fn read_segment_type<R: bitstream_io::read::BitRead>(r: &mut R) -> Result<u8, crate::sections::SectionDecodeError> {
                Ok(r.read_unsigned_var(#n)?)
            }
        }),
        _ => None,
    };

    quote! {
        impl crate::sections::OptionalSegmentParser for #ident {
            #read_segment_type_override

            fn parse_optional_segment<R: bitstream_io::read::BitRead>(
                segment_type: u8,
                r: &mut R,
                into: &mut Self,
            ) -> Result<(), crate::sections::SectionDecodeError> {
                match segment_type {
                    #(#parse_match_arms)*
                    n => {
                        return Err(crate::sections::SectionDecodeError::UnknownSegmentType { segment_type: n });
                    }
                }
                Ok(())
            }
        }
    }
}

use crate::field_attr::GPPFieldHelperAttribute;
use proc_macro2::Ident;
use quote::quote;
use syn::{DataStruct, Visibility};

pub fn derive_struct_from_data_reader(
    input: &DataStruct,
    ident: &Ident,
) -> proc_macro2::TokenStream {
    // generate FromReader impl block
    // # loop over all fields
    // - by default call a FromReader implementation
    // - use more
    let mut parse_statements = vec![];
    let mut field_names = vec![];

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
        field_names.push(name.clone());

        let attr = GPPFieldHelperAttribute::new(&field.attrs).expect("attribute parsing failed");

        if attr.optional_segment_type.is_some() {
            parse_statements.push(quote! {
                let #name = None;
            });
        } else {
            let expr = attr.parser_expr();
            parse_statements.push(quote! {
                let #name = #expr?;
            });
        }

        if let Some(version) = attr.expected_section_version {
            parse_statements.push(quote! {
                if version != #version {
                    return Err(crate::sections::SectionDecodeError::InvalidSectionVersion {
                        expected: #version,
                        found: version,
                    });
                }
            });
        }
    }

    quote! {
        impl crate::core::FromDataReader for #ident {
            type Err = crate::sections::SectionDecodeError;

            fn from_data_reader(r: &mut crate::core::DataReader) -> Result<Self, Self::Err> {
                #(#parse_statements)*

                Ok(Self{
                    #(#field_names),*
                })
            }
        }
    }
}

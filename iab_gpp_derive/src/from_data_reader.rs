use crate::enum_variant_attr::GPPEnumVariantHelperAttribute;
use crate::field_attr::GPPFieldHelperAttribute;
use crate::struct_attr::GPPStructHelperAttribute;
use proc_macro2::Ident;
use quote::quote;
use syn::{DataEnum, DataStruct, Visibility};

pub fn derive_struct_from_data_reader(
    input: &DataStruct,
    ident: &Ident,
    struct_attr: &GPPStructHelperAttribute,
) -> proc_macro2::TokenStream {
    // generate FromReader impl block
    // - check version first if needed
    // # loop over all fields
    // - by default call a FromReader implementation
    // - use DataReader methods if specified
    let mut parse_statements = vec![];
    let mut field_names = vec![];

    if let Some(version) = struct_attr.section_version {
        parse_statements.push(quote! {
            let version = r.read_fixed_integer(6)?;
            if version != #version {
                return Err(crate::sections::SectionDecodeError::UnknownSegmentVersion {
                    segment_version: version,
                });
            }
        });
    }

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

        // debug next field
        /*parse_statements.push(quote! {
            println!("Parsing field: {}", stringify!(#name));
        });*/

        // Handle where attribute
        if let Some(where_spec) = attr.where_spec {
            let name = where_spec.name;
            let expr = where_spec.parser.to_token_stream();
            parse_statements.push(quote! {
                let #name: u64 = #expr?;
            })
        }

        // Handle optional segments
        if attr.optional_segment_type.is_some() {
            parse_statements.push(quote! {
                let #name = None;
            });
        } else {
            let expr = attr.parser.to_token_stream();
            parse_statements.push(quote! {
                let #name = #expr?;
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

pub fn derive_enum_from_data_reader(input: &DataEnum, ident: &Ident) -> proc_macro2::TokenStream {
    // generate FromReader impl block
    // # loop over all variants
    // - read version attribute
    // - call a FromReader implementation
    let mut versions = vec![];

    for variant in &input.variants {
        let name = variant.ident.clone();

        let attr =
            GPPEnumVariantHelperAttribute::new(&variant.attrs).expect("attribute parsing failed");

        // Handle each versioned variant
        if let Some(v) = attr.variant_version {
            versions.push(quote! {
                #v => Ok(Self::#name(r.parse()?)),
            });
        }
    }

    quote! {
        impl crate::core::FromDataReader for #ident {
            type Err = crate::sections::SectionDecodeError;

            fn from_data_reader(r: &mut crate::core::DataReader) -> Result<Self, Self::Err> {
                let version = r.read_fixed_integer(6)?;
                match version {
                    #(#versions)*
                    v => Err(crate::sections::SectionDecodeError::UnknownSegmentVersion { segment_version: v }),
                }
            }
        }
    }
}

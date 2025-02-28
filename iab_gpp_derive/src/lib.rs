use crate::from_data_reader::{derive_enum_from_data_reader, derive_struct_from_data_reader};
use crate::optional_segment_parser::derive_optional_segment_parser;
use crate::struct_attr::{GPPStructHelperAttribute, GPPStructKind};
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{quote, TokenStreamExt};
use syn::{parse_macro_input, Attribute, Data, DataStruct, DeriveInput};

mod enum_variant_attr;
mod field_attr;
mod from_data_reader;
mod optional_segment_parser;
mod struct_attr;

/// Derive the FromDataReader trait
#[proc_macro_derive(FromDataReader, attributes(gpp))]
pub fn derive_from_data_reader(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match input.data {
        Data::Struct(s) => {
            let attr =
                GPPStructHelperAttribute::new(&input.attrs).expect("attribute parsing failed");
            derive_struct_from_data_reader(&s, &input.ident, &attr).into()
        }
        Data::Enum(e) => {
            // we don't support enum-level attributes
            derive_enum_from_data_reader(&e, &input.ident).into()
        }
        _ => TokenStream::new(),
    }
}

#[proc_macro_derive(GPPSection, attributes(gpp))]
pub fn derive_gpp_section(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;

    if let Data::Struct(s) = input.data {
        // first derive DecodableSection which applies to all sections
        let stream = quote! {
            impl crate::sections::DecodableSection for #ident {
                const ID: crate::sections::SectionId = crate::sections::SectionId::#ident;
            }
        };

        // section deriving depends on what kind of section we're dealing with
        let attr = GPPStructHelperAttribute::new(&input.attrs).expect("attribute parsing failed");
        match attr.kind {
            GPPStructKind::Base64Data => {
                // simple FromDataReader impl that read all fields in sequence
                // it's the default one
                impl_base64_gpp_section(ident, s, &attr, stream)
            }
            GPPStructKind::WithOptionalSegments(_) => {
                // FromDataReader impl is altered, we have a mandatory segment
                // followed by optional ones.
                // The impl reads the first segment, then fills the rest with Nones
                // we then add a OptionalSegmentParser impl which reads the rest.
                impl_segmented_gpp_section(ident, s, &attr, stream)
            }
        }
    } else {
        // just ignore attempts to derive things that are not structs
        TokenStream::new()
    }
}

fn impl_base64_gpp_section(
    ident: Ident,
    s: DataStruct,
    attr: &GPPStructHelperAttribute,
    mut stream: proc_macro2::TokenStream,
) -> TokenStream {
    // FromStr impl which parses the given string using Base64
    stream.append_all(quote! {
        impl ::std::str::FromStr for #ident {
            type Err = crate::sections::SectionDecodeError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                use crate::sections::Base64EncodedStr;
                s.parse_base64_str()
            }
        }
    });

    stream.append_all(derive_struct_from_data_reader(&s, &ident, attr));

    stream.into()
}

fn impl_segmented_gpp_section(
    ident: Ident,
    s: DataStruct,
    attr: &GPPStructHelperAttribute,
    mut stream: proc_macro2::TokenStream,
) -> TokenStream {
    // FromStr impl which parses the given string as a sequence of segments
    stream.append_all(quote! {
        impl ::std::str::FromStr for #ident {
            type Err = crate::sections::SectionDecodeError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                use crate::sections::SegmentedStr;
                s.parse_segmented_str()
            }
        }
    });

    stream.append_all(derive_struct_from_data_reader(&s, &ident, attr));

    // OptionalSegmentParser impl
    stream.append_all(derive_optional_segment_parser(&s, &ident, attr));

    stream.into()
}

fn find_gpp_attr(attrs: &[Attribute]) -> Option<&Attribute> {
    attrs.iter().find(|attr| attr.path().is_ident("gpp"))
}

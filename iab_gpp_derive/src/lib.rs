use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{format_ident, quote, TokenStreamExt};
use syn::{
    parenthesized, parse, parse_macro_input, token, Attribute, Data, DataStruct, DeriveInput, Expr,
    ExprCall, LitInt,
};
// #[proc_macro_derive(FromDataReader, attributes(gpp))]
// pub fn derive_from_data_reader(input: TokenStream) -> TokenStream {
//     let input = parse_macro_input!(input as DeriveInput);
//
//     if let Data::Struct(s) = input.data {
//         derive_struct_from_data_reader(s, input.ident)
//     } else {
//         TokenStream::new()
//     }
// }

#[proc_macro_derive(GPPSection, attributes(gpp))]
pub fn derive_gpp_section(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;

    let mut stream = quote! {
        impl crate::sections::DecodableSection for #ident {
            const ID: crate::sections::SectionId = crate::sections::SectionId::#ident;
        }
    };

    // if using the base64 format, generate a FromStr that parses base64
    // should be the default one
    // let attrs = input.attrs;

    stream.append_all(quote! {
        impl std::str::FromStr for #ident {
            type Err = SectionDecodeError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                use crate::sections::Base64EncodedStr;
                s.parse_base64_str()
            }
        }
    });

    let s = if let Data::Struct(s) = input.data {
        derive_struct_from_data_reader(s, ident)
    } else {
        Default::default()
    };

    stream.append_all(s);
    stream.into()
}

fn derive_struct_from_data_reader(input: DataStruct, ident: Ident) -> proc_macro2::TokenStream {
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
        let name = name.unwrap();
        field_names.push(name.clone());

        let attr = gpp_field_helper_attribute(&field.attrs).expect("attribute parsing failed");

        if let Some(reader_call) = attr.reader_call {
            parse_statements.push(quote! {
                let #name = r.#reader_call?;
            });
        } else if let Some(parse_with) = attr.parse_with {
            parse_statements.push(quote! {
                let #name = #parse_with(r)?;
            });
        }
        if let Some(version) = attr.expected_section_version {
            parse_statements.push(quote! {
                if version != #version {
                    return Err(SectionDecodeError::InvalidSectionVersion {
                        expected: #version,
                        found: version,
                    });
                }
            });
        }
    }

    let block = quote! {
        impl crate::core::FromDataReader for #ident {
            type Err = crate::sections::SectionDecodeError;

            fn from_data_reader(r: &mut DataReader) -> Result<Self, Self::Err> {
                #(#parse_statements)*

                Ok(Self{
                    #(#field_names),*
                })
            }
        }
    };

    block
}

//struct GPPStructHelperAttribute {}

struct GPPFieldHelperAttribute {
    expected_section_version: Option<u8>,
    reader_call: Option<ExprCall>,
    parse_with: Option<Ident>,
}

fn find_gpp_attr(attrs: &[Attribute]) -> Option<&Attribute> {
    attrs.iter().find(|attr| attr.path().is_ident("gpp"))
}

fn gpp_field_helper_attribute(attrs: &[Attribute]) -> parse::Result<GPPFieldHelperAttribute> {
    let mut gpp_attr = GPPFieldHelperAttribute {
        expected_section_version: None,
        reader_call: None,
        parse_with: None,
    };
    if let Some(attr) = find_gpp_attr(attrs) {
        attr.parse_nested_meta(|meta| {
            // #[gpp(section_version = N)]
            if meta.path.is_ident("section_version") {
                let value = meta.value()?; // parses the `=`
                let s = value.parse::<LitInt>()?;
                gpp_attr.expected_section_version = Some(s.base10_parse()?);
                return Ok(());
            }

            // #[gpp(parse_with = fn_name)]
            if meta.path.is_ident("parse_with") {
                let value = meta.value()?; // parses the `=`
                let s = value.parse::<Ident>()?;
                gpp_attr.parse_with = Some(s);
                return Ok(());
            }

            // #[gpp(S)] where S interpreted as a call like r.read_S
            // if no parenthesis, assume call without args
            if let Some(ident) = meta.path.get_ident() {
                gpp_attr.reader_call = if meta.input.peek(token::Paren) {
                    let content;
                    parenthesized!(content in meta.input);
                    let lit: LitInt = content.parse()?;
                    Some(create_read_function_call_with_int(ident, lit))
                } else {
                    Some(create_read_function_call(ident))
                };
                return Ok(());
            }

            Err(meta.error("unrecognized gpp parameter"))
        })?;
    }

    Ok(gpp_attr)
}

fn create_read_function_call_with_int(func_name: &Ident, arg: LitInt) -> ExprCall {
    let mut call = create_read_function_call(func_name);
    call.args.push(Expr::Lit(syn::ExprLit {
        attrs: Vec::new(),
        lit: syn::Lit::Int(arg),
    }));

    call
}

fn create_read_function_call(func_name: &Ident) -> ExprCall {
    let name = format_ident!("read_{func_name}");
    ExprCall {
        attrs: Vec::new(),
        func: Box::new(Expr::Path(syn::ExprPath {
            attrs: Vec::new(),
            qself: None,
            path: name.into(),
        })),
        paren_token: Default::default(),
        args: syn::punctuated::Punctuated::new(),
    }
}

use crate::find_gpp_attr;
use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::{parenthesized, parse, token, Attribute, Expr, ExprCall, LitInt};

enum GPPFieldParser {
    FromDataReader,
    ReaderCall(ExprCall),
    Function(Ident),
}

pub struct GPPFieldHelperAttribute {
    pub expected_section_version: Option<u8>,
    pub optional_segment_type: Option<u8>,
    parser: GPPFieldParser,
}

impl GPPFieldHelperAttribute {
    pub fn new(attrs: &[Attribute]) -> parse::Result<Self> {
        let mut gpp_attr = Self {
            expected_section_version: None,
            optional_segment_type: None,
            parser: GPPFieldParser::FromDataReader,
        };
        if let Some(attr) = find_gpp_attr(attrs) {
            attr.parse_nested_meta(|meta| {
                // #[gpp(expect_section_version = N)]
                if meta.path.is_ident("expect_section_version") {
                    let value = meta.value()?; // parses the `=`
                    let s = value.parse::<LitInt>()?;
                    gpp_attr.expected_section_version = Some(s.base10_parse()?);
                    return Ok(());
                }

                // #[gpp(optional_segment_type = N)]
                if meta.path.is_ident("optional_segment_type") {
                    let value = meta.value()?; // parses the `=`
                    let s = value.parse::<LitInt>()?;
                    gpp_attr.optional_segment_type = Some(s.base10_parse()?);
                    return Ok(());
                }

                // #[gpp(parse_with = fn_name)]
                if meta.path.is_ident("parse_with") {
                    let value = meta.value()?; // parses the `=`
                    let s = value.parse::<Ident>()?;
                    gpp_attr.parser = GPPFieldParser::Function(s);
                    return Ok(());
                }

                // #[gpp(S)] where S interpreted as a call like r.read_S
                // if no parenthesis, assume call without args
                if let Some(ident) = meta.path.get_ident() {
                    gpp_attr.parser = if meta.input.peek(token::Paren) {
                        let content;
                        parenthesized!(content in meta.input);
                        let lit: LitInt = content.parse()?;
                        GPPFieldParser::ReaderCall(Self::create_read_function_call(ident, &[lit]))
                    } else {
                        GPPFieldParser::ReaderCall(Self::create_read_function_call(ident, &[]))
                    };
                    return Ok(());
                }

                Err(meta.error("unrecognized gpp parameter"))
            })?;
        }

        Ok(gpp_attr)
    }

    pub fn parser_expr(&self) -> proc_macro2::TokenStream {
        match &self.parser {
            GPPFieldParser::FromDataReader => quote! {
                r.parse()
            },
            GPPFieldParser::ReaderCall(c) => quote! {
                r.#c
            },
            GPPFieldParser::Function(f) => quote! {
                #f(r)
            },
        }
    }

    fn create_read_function_call(func_name: &Ident, args: &[LitInt]) -> ExprCall {
        let name = format_ident!("read_{func_name}");
        let mut call = ExprCall {
            attrs: Vec::new(),
            func: Box::new(Expr::Path(syn::ExprPath {
                attrs: Vec::new(),
                qself: None,
                path: name.into(),
            })),
            paren_token: Default::default(),
            args: syn::punctuated::Punctuated::new(),
        };

        for arg in args {
            call.args.push(Expr::Lit(syn::ExprLit {
                attrs: Vec::new(),
                lit: syn::Lit::Int(arg.clone()),
            }));
        }

        call
    }
}

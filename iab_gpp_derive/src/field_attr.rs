use crate::find_gpp_attr;
use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::meta::ParseNestedMeta;
use syn::parse::ParseStream;
use syn::punctuated::Punctuated;
use syn::{parenthesized, parse, token, Attribute, Expr, ExprCall, LitInt};

pub enum GPPFieldParser {
    FromDataReader,
    ReaderCall(ExprCall),
    Function(Ident),
}

impl GPPFieldParser {
    pub fn to_token_stream(&self) -> proc_macro2::TokenStream {
        match &self {
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
}

pub struct GPPFieldHelperAttribute {
    pub optional_segment_type: Option<u8>,
    pub where_spec: Option<WhereSpec>,
    pub parser: GPPFieldParser,
}

pub struct WhereSpec {
    pub name: Ident,
    pub parser: GPPFieldParser,
}

impl GPPFieldHelperAttribute {
    pub fn new(attrs: &[Attribute]) -> parse::Result<Self> {
        let mut gpp_attr = Self {
            optional_segment_type: None,
            where_spec: None,
            parser: GPPFieldParser::FromDataReader,
        };
        if let Some(attr) = find_gpp_attr(attrs) {
            attr.parse_nested_meta(|meta| {
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

                // #[gpp(where(n = PARSER))]
                // declares that the current field is preceded by a
                // binding named "n" which is parsed using PARSER as
                // a call of the form r.read_S.
                if meta.path.is_ident("where") {
                    meta.parse_nested_meta(|where_meta| {
                        gpp_attr.where_spec = Self::parse_where_meta(where_meta)?;
                        return Ok(());
                    })?;

                    return Ok(());
                }

                // #[gpp(PARSER)] where PARSER interpreted as a call like r.read_PARSER
                // if no parenthesis, assume call without args
                // if arg is a literal, use as-is
                // if arg is an ident, assume a 6 bit integer to be reused
                // as many times as referenced
                if let Some(ident) = meta.path.get_ident() {
                    if let Some(parser) = Self::get_parser(&meta.input, ident)? {
                        gpp_attr.parser = parser;
                    }

                    return Ok(());
                }

                Err(meta.error("unrecognized gpp field parameter"))
            })?;
        }

        Ok(gpp_attr)
    }

    fn parse_where_meta(meta: ParseNestedMeta) -> Result<Option<WhereSpec>, syn::Error> {
        if let Some(ident) = meta.path.get_ident() {
            let mut where_spec = WhereSpec {
                name: ident.clone(),
                parser: GPPFieldParser::FromDataReader,
            };

            let value = meta.value()?;
            let ident: Ident = value.parse()?;

            if let Some(parser) = Self::get_parser(&value, &ident)? {
                where_spec.parser = parser;
            }

            return Ok(Some(where_spec));
        }

        Err(meta.error("unrecognized where field parameter"))
    }

    fn get_parser(
        input: &ParseStream,
        ident: &Ident,
    ) -> Result<Option<GPPFieldParser>, syn::Error> {
        if input.peek(token::Paren) {
            let content;
            parenthesized!(content in input);
            let arg: Expr = content.parse()?;
            Ok(Some(GPPFieldParser::ReaderCall(
                Self::create_read_function_call(ident, &[arg]),
            )))
        } else {
            Ok(Some(GPPFieldParser::ReaderCall(
                Self::create_read_function_call(ident, &[]),
            )))
        }
    }

    fn create_read_function_call(func_name: &Ident, args: &[Expr]) -> ExprCall {
        let name = format_ident!("read_{func_name}");
        let mut call = ExprCall {
            attrs: Vec::new(),
            func: Box::new(Expr::Path(syn::ExprPath {
                attrs: Vec::new(),
                qself: None,
                path: name.into(),
            })),
            paren_token: Default::default(),
            args: Punctuated::new(),
        };

        for arg in args {
            call.args.push(arg.clone());
        }

        call
    }
}

// Copyright 2022 Philipp Fensch
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::parser::Rule;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use super::constant::Value;

#[derive(PartialEq)]
pub struct Enumdef {
    name: String,
    enum_body: Enum,
}

#[derive(PartialEq)]
pub struct Enum {
    pub cases: std::vec::Vec<(String, Value)>,
}

impl From<&Enumdef> for TokenStream {
    fn from(enum_def: &Enumdef) -> TokenStream {
        let name = quote::format_ident!("{}", enum_def.name);
        let enum_body = TokenStream::from(&enum_def.enum_body);
        quote!(enum #name #enum_body)
    }
}

impl From<&Enum> for TokenStream {
    fn from(en: &Enum) -> TokenStream {
        let mut code = quote!();
        for (case_ident, case_value) in &en.cases {
            let case_name = format_ident!("{}", case_ident);
            match case_value {
                Value::Numeric { val } => {
                    code = quote!(#code #case_name = #val as isize,);
                }
                Value::Named { name } => {
                    let value_name = format_ident!("{}", name);
                    code = quote!(#code #case_name = #value_name,);
                }
            }
        }
        quote!( { #code } )
    }
}

pub fn parse_enum_type_spec(enum_type_spec: pest::iterators::Pair<'_, Rule>) -> Enum {
    Enum::from(enum_type_spec.into_inner().next().unwrap())
}

impl From<pest::iterators::Pair<'_, Rule>> for Enumdef {
    fn from(enum_def: pest::iterators::Pair<'_, Rule>) -> Enumdef {
        let mut iter = enum_def.into_inner();
        let enum_name = iter.next().unwrap();
        let enum_body = iter.next().unwrap();

        Enumdef {
            name: enum_name.as_str().to_string(),
            enum_body: Enum::from(enum_body),
        }
    }
}

impl From<pest::iterators::Pair<'_, Rule>> for Enum {
    fn from(enum_body: pest::iterators::Pair<'_, Rule>) -> Enum {
        let mut enum_def = Enum {
            cases: std::vec::Vec::new(),
        };
        for enum_case in enum_body.into_inner() {
            let mut iter = enum_case.into_inner();
            let name = iter.next().unwrap().as_str().to_string();
            let value = Value::from(iter.next().unwrap());
            enum_def.cases.push((name, value));
        }
        enum_def
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::RPCLParser;
    use crate::pest::Parser;

    #[test]
    fn parse_enum_1() {
        // Parsing
        let mut parsed =
            RPCLParser::parse(Rule::enum_body, "{CASE1 = 2, CASE_T = 0xa, _CASE = CONST}").unwrap();
        let enum_generated = Enum::from(parsed.next().unwrap());
        let enum_coded = Enum {
            cases: vec![
                ("CASE1".into(), Value::Numeric { val: 2 }),
                ("CASE_T".into(), Value::Numeric { val: 10 }),
                (
                    "_CASE".into(),
                    Value::Named {
                        name: "CONST".into(),
                    },
                ),
            ],
        };
        assert!(enum_generated == enum_coded, "Enum parsing wrong");

        // Code-gen
        let rust_code: TokenStream =
            quote!( { CASE1 = 2i64 as isize, CASE_T = 10i64 as isize, _CASE = CONST, } );
        let generated_code: TokenStream = (&enum_generated).into();
        assert!(
            generated_code.to_string() == rust_code.to_string(),
            "DataType: Generated code wrong:\n{}\n{}",
            generated_code.to_string(),
            rust_code.to_string()
        );
    }

    #[test]
    fn parse_enum_def_1() {
        // Parsing
        let mut parsed = RPCLParser::parse(Rule::enum_def, "enum Name { A = 1, B = 2};").unwrap();
        let enum_generated = Enumdef::from(parsed.next().unwrap());
        let enum_coded = Enumdef {
            name: "Name".to_string(),
            enum_body: Enum {
                cases: vec![
                    ("A".into(), Value::Numeric { val: 1 }),
                    ("B".into(), Value::Numeric { val: 2 }),
                ],
            },
        };
        assert!(enum_generated == enum_coded, "Enum parsing wrong");

        // Code-gen
        let rust_code: TokenStream = quote!(
            enum Name {
                A = 1i64 as isize,
                B = 2i64 as isize,
            }
        );
        let generated_code: TokenStream = (&enum_generated).into();
        assert!(
            generated_code.to_string() == rust_code.to_string(),
            "DataType: Generated code wrong:\n{}\n{}",
            generated_code.to_string(),
            rust_code.to_string()
        );
    }

    #[test]
    fn parse_enum_type_spec_1() {
        // Parsing
        let mut parsed = RPCLParser::parse(Rule::enum_type_spec, "enum { A = 1, B = 2}").unwrap();
        let enum_generated = parse_enum_type_spec(parsed.next().unwrap());
        let enum_coded = Enum {
            cases: vec![
                ("A".into(), Value::Numeric { val: 1 }),
                ("B".into(), Value::Numeric { val: 2 }),
            ],
        };
        assert!(enum_generated == enum_coded, "Enum parsing wrong");
    }
}

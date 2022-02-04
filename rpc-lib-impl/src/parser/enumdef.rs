use crate::parser::parser::Rule;

use proc_macro2::TokenStream;
use quote::quote;

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
        let enum_body: TokenStream = (&enum_def.enum_body).into();
        quote!(enum #name #enum_body)
    }
}

impl From<&Enum> for TokenStream {
    fn from(en: &Enum) -> TokenStream {
        let mut code = quote!();
        for (case_ident, case_value) in &en.cases {
            match case_value {
                Value::Numeric { val } => {
                    code = quote!(#code #case_ident = #val as isize,);
                }
                Value::Named { name } => {
                    code = quote!(#code #case_ident = #name,);
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
        let name = iter.next().unwrap();
        let enum_body = iter.next().unwrap();

        Enumdef {
            name: name.as_str().to_string(),
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
    use crate::parser::parser::RPCLParser;
    use crate::pest::Parser;

    #[test]
    fn parse_enum_1() {
        let mut parsed =
            RPCLParser::parse(Rule::enum_body, "{CASE1 = 2, CASE_T = 0xa, _CASE = CONST}").unwrap();
        let enum_body = Enum::from(parsed.next().unwrap());

        let en = Enum {
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
        assert!(en == enum_body, "Enum Body wrong");
    }

    #[test]
    fn parse_enum_def_1() {
        let mut parsed = RPCLParser::parse(Rule::enum_def, "enum Name { A = 1, B = 2};").unwrap();
        let parsed_def = Enumdef::from(parsed.next().unwrap());

        let enum_def = Enumdef {
            name: "Name".into(),
            enum_body: Enum {
                cases: vec![
                    ("A".into(), Value::Numeric { val: 1 }),
                    ("B".into(), Value::Numeric { val: 2 }),
                ],
            },
        };
        assert!(enum_def == parsed_def, "Enum Def wrong");
    }

    #[test]
    fn parse_enum_type_spec_1() {
        let mut parsed = RPCLParser::parse(Rule::enum_type_spec, "enum { A = 1, B = 2}").unwrap();
        let enum_body = parse_enum_type_spec(parsed.next().unwrap());

        let en = Enum {
            cases: vec![
                ("A".into(), Value::Numeric { val: 1 }),
                ("B".into(), Value::Numeric { val: 2 }),
            ],
        };
        assert!(en == enum_body, "Enum Type Spec wrong");
    }
}

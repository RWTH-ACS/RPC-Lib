// Copyright 2022 Philipp Fensch
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::parser::Rule;
use proc_macro2::TokenStream;
use quote::quote;

use super::enumdef::{parse_enum_type_spec, Enum};
use super::structdef::{parse_struct_type_spec, Struct};
use super::uniondef::{parse_union_type_spec, Union};

#[derive(PartialEq)]
pub enum DataType {
    Integer { length: u32, signed: bool },
    Float { length: u32 },
    String,
    Boolean,
    TypeDef { name: String },
    Struct { def: Struct },
    Union { def: Union },
    Enum { def: Enum },
    Void,
}

impl From<&DataType> for TokenStream {
    fn from(data_type: &DataType) -> TokenStream {
        match data_type {
            DataType::Integer { length, signed } => match signed {
                true => match length {
                    32 => quote!(i32),
                    64 => quote!(i64),
                    _ => panic!(""),
                },
                false => match length {
                    32 => quote!(u32),
                    64 => quote!(u64),
                    _ => panic!(""),
                },
            },
            DataType::Float { length } => match length {
                32 => quote!(f32),
                64 => quote!(f64),
                _ => panic!(""),
            },
            DataType::String => {
                quote!(String)
            }
            DataType::Boolean => {
                quote!(bool)
            }
            DataType::TypeDef { name } => {
                let ident = quote::format_ident!("{}", name);
                quote!(#ident)
            }
            DataType::Struct { def: _ } => {
                panic!("Anonymous struct as Datatype not implemented");
            }
            DataType::Union { def: _ } => {
                panic!("Anonymous union as Datatype not implemented");
            }
            DataType::Enum { def: _ } => {
                panic!("Anonymous enum as Datatype not implemented");
            }
            DataType::Void => {
                quote!()
            }
        }
    }
}

fn parse_primitive(primitive_type: pest::iterators::Pair<'_, Rule>) -> DataType {
    match primitive_type.as_str() {
        "unsigned int" => DataType::Integer {
            length: 32,
            signed: false,
        },
        "int" => DataType::Integer {
            length: 32,
            signed: true,
        },
        "unsigned hyper" => DataType::Integer {
            length: 64,
            signed: false,
        },
        "hyper" => DataType::Integer {
            length: 64,
            signed: true,
        },
        "float" => DataType::Float { length: 32 },
        "double" => DataType::Float { length: 64 },
        "quadruple" => DataType::Float { length: 128 },
        "bool" => DataType::Boolean,
        "string" | "string<>" => DataType::String,
        _ => panic!("Syntax Error"),
    }
}

impl From<pest::iterators::Pair<'_, Rule>> for DataType {
    fn from(type_specifier: pest::iterators::Pair<'_, Rule>) -> DataType {
        // type_specifier > inner_rule (e.g. primitve_type, enum_type_spec, ...)
        let inner_rule = type_specifier.into_inner().next().unwrap();
        match inner_rule.as_rule() {
            Rule::primitive_type => parse_primitive(inner_rule),
            Rule::void => DataType::Void,
            Rule::enum_type_spec => DataType::Enum {
                def: parse_enum_type_spec(inner_rule),
            },
            Rule::union_type_spec => DataType::Union {
                def: parse_union_type_spec(inner_rule),
            },
            Rule::struct_type_spec => DataType::Struct {
                def: parse_struct_type_spec(inner_rule),
            },
            Rule::identifier => DataType::TypeDef {
                name: inner_rule.as_str().to_string(),
            },
            _ => panic!("Syntax Error"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::constant::Value;
    use super::*;
    use crate::parser::RPCLParser;
    use pest::Parser;

    #[test]
    fn parse_type_spec_primitive_1() {
        // Parsing
        let mut parsed = RPCLParser::parse(Rule::type_specifier, "unsigned int").unwrap();
        let data_generated = DataType::from(parsed.next().unwrap());
        let data_coded = DataType::Integer {
            length: 32,
            signed: false,
        };
        assert!(data_generated == data_coded, "Datatype parsed wrong");

        // Code-gen
        let rust_code: TokenStream = quote!(u32);
        let generated_code: TokenStream = (&data_generated).into();
        assert!(
            generated_code.to_string() == rust_code.to_string(),
            "DataType: Generated code wrong:\n{}\n{}",
            generated_code.to_string(),
            rust_code.to_string()
        );
    }

    #[test]
    fn parse_type_spec_primitive_2() {
        // Parsing
        let mut parsed = RPCLParser::parse(Rule::type_specifier, "float").unwrap();
        let data_generated = DataType::from(parsed.next().unwrap());
        let data_coded = DataType::Float { length: 32 };
        assert!(data_generated == data_coded, "Datatype parsed wrong");

        // Code-gen
        let rust_code: TokenStream = quote!(f32);
        let generated_code: TokenStream = (&data_generated).into();
        assert!(
            generated_code.to_string() == rust_code.to_string(),
            "DataType: Generated code wrong:\n{}\n{}",
            generated_code.to_string(),
            rust_code.to_string()
        );
    }

    #[test]
    fn parse_type_spec_primitive_3() {
        // Parsing
        let mut parsed = RPCLParser::parse(Rule::type_specifier, "bool").unwrap();
        let data_generated = DataType::from(parsed.next().unwrap());
        assert!(data_generated == DataType::Boolean, "Datatype parsed wrong");

        // Code-gen
        let rust_code: TokenStream = quote!(bool);
        let generated_code: TokenStream = (&data_generated).into();
        assert!(
            generated_code.to_string() == rust_code.to_string(),
            "DataType: Generated code wrong:\n{}\n{}",
            generated_code.to_string(),
            rust_code.to_string()
        );
    }

    #[test]
    fn parse_type_spec_custom_type_1() {
        // Parsing
        let mut parsed = RPCLParser::parse(Rule::type_specifier, "MyCustom23Type").unwrap();
        let data_generated = DataType::from(parsed.next().unwrap());
        let data_coded = DataType::TypeDef {
            name: "MyCustom23Type".to_string(),
        };
        assert!(data_generated == data_coded, "Datatype parsed wrong");

        // Code-gen
        let rust_code: TokenStream = quote!(MyCustom23Type);
        let generated_code: TokenStream = (&data_generated).into();
        assert!(
            generated_code.to_string() == rust_code.to_string(),
            "DataType: Generated code wrong:\n{}\n{}",
            generated_code.to_string(),
            rust_code.to_string()
        );
    }

    #[test]
    #[should_panic(expected = "Anonymous enum as Datatype not implemented")]
    fn parse_enum_type_spec_1() {
        let mut parsed = RPCLParser::parse(Rule::type_specifier, "enum { A = 1 }").unwrap();
        let data_generated = DataType::from(parsed.next().unwrap());
        let data_coded = DataType::Enum {
            def: Enum {
                cases: vec![("A".into(), Value::Numeric { val: 1 })],
            },
        };
        assert!(data_generated == data_coded, "Datatype parsed wrong");

        // Code-gen
        let rust_code: TokenStream = quote!();
        let generated_code: TokenStream = (&data_generated).into();
        assert!(
            generated_code.to_string() == rust_code.to_string(),
            "DataType: Generated code wrong:\n{}\n{}",
            generated_code.to_string(),
            rust_code.to_string()
        );
    }
}

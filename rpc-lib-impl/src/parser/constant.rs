// Copyright 2022 Philipp Fensch
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::parser::parser::Rule;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

#[derive(PartialEq)]
pub struct ConstantDeclaration {
    name: String,
    value: Value,
}

#[derive(PartialEq)]
pub enum Value {
    Numeric { val: i64 },
    Named { name: String },
}

impl From<&ConstantDeclaration> for TokenStream {
    fn from(constant: &ConstantDeclaration) -> TokenStream {
        let name = format_ident!("{}", &constant.name);
        let value = TokenStream::from(&constant.value);
        quote!(const #name: i64 = #value;)
    }
}

impl From<&Value> for TokenStream {
    fn from(value: &Value) -> TokenStream {
        match value {
            Value::Numeric { val } => {
                quote!(#val)
            }
            Value::Named { name } => {
                quote!(#name)
            }
        }
        .into()
    }
}

fn parse_num(constant: pest::iterators::Pair<'_, Rule>) -> i64 {
    let rule_str = constant.as_str();
    if rule_str.len() >= 3 && &rule_str[0..2] == "0x" {
        // Hex
        i64::from_str_radix(&rule_str[2..], 16)
    } else if rule_str.len() >= 2 && &rule_str[0..1] == "0" {
        // Oct
        i64::from_str_radix(&rule_str[1..], 8)
    } else {
        // Dec
        rule_str.parse::<i64>()
    }
    .unwrap()
}

impl From<pest::iterators::Pair<'_, Rule>> for Value {
    fn from(value: pest::iterators::Pair<'_, Rule>) -> Value {
        let token = value.into_inner().next().unwrap();
        match token.as_rule() {
            Rule::constant => Value::Numeric {
                val: parse_num(token),
            },
            Rule::identifier => Value::Named {
                name: token.as_str().to_string(),
            },
            _ => panic!("Syntax Error"),
        }
    }
}

impl From<pest::iterators::Pair<'_, Rule>> for ConstantDeclaration {
    fn from(constant_def: pest::iterators::Pair<'_, Rule>) -> ConstantDeclaration {
        let mut it = constant_def.into_inner();
        let name = it.next().unwrap();
        let value = it.next().unwrap();
        ConstantDeclaration {
            name: name.as_str().to_string(),
            value: Value::Numeric {
                val: parse_num(value),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parser::RPCLParser;
    use crate::pest::Parser;

    #[test]
    fn parse_constant_decimal() {
        // Parsing
        let mut parsed = RPCLParser::parse(Rule::constant_def, "const CON = 23;").unwrap();
        let const_generated = ConstantDeclaration::from(parsed.next().unwrap());
        let const_coded = ConstantDeclaration {
            name: "CON".to_string(),
            value: Value::Numeric { val: 23 },
        };
        assert!(const_generated == const_coded, "Constant parsing wrong");

        // Code-gen
        let rust_code: TokenStream = quote!(
            const CON: i64 = 23i64;
        );
        let generated_code: TokenStream = (&const_generated).into();
        assert!(
            generated_code.to_string() == rust_code.to_string(),
            "DataType: Generated code wrong:\n{}\n{}",
            generated_code.to_string(),
            rust_code.to_string()
        );
    }

    #[test]
    fn parse_constant_hexadecimal() {
        // Parsing
        let mut parsed = RPCLParser::parse(Rule::constant_def, "const CON2 = 0x2889;").unwrap();
        let const_generated = ConstantDeclaration::from(parsed.next().unwrap());
        let const_coded = ConstantDeclaration {
            name: "CON2".to_string(),
            value: Value::Numeric { val: 0x2889 },
        };
        assert!(const_generated == const_coded, "Constant parsing wrong");

        // Code-gen
        let rust_code: TokenStream = quote!(
            const CON2: i64 = 10377i64;
        );
        let generated_code: TokenStream = (&const_generated).into();
        assert!(
            generated_code.to_string() == rust_code.to_string(),
            "DataType: Generated code wrong:\n{}\n{}",
            generated_code.to_string(),
            rust_code.to_string()
        );
    }

    #[test]
    fn parse_constant_negative_decimal() {
        // Parsing
        let mut parsed = RPCLParser::parse(Rule::constant_def, "const CON = -68;").unwrap();
        let const_generated = ConstantDeclaration::from(parsed.next().unwrap());
        let const_coded = ConstantDeclaration {
            name: "CON".to_string(),
            value: Value::Numeric { val: -68 },
        };
        assert!(const_generated == const_coded, "Constant parsing wrong");

        // Code-gen
        let rust_code: TokenStream = quote!(
            const CON: i64 = -68i64;
        );
        let generated_code: TokenStream = (&const_generated).into();
        assert!(
            generated_code.to_string() == rust_code.to_string(),
            "DataType: Generated code wrong:\n{}\n{}",
            generated_code.to_string(),
            rust_code.to_string()
        );
    }

    #[test]
    fn parse_constant_octal() {
        // Parsing
        let mut parsed = RPCLParser::parse(Rule::constant_def, "const CON = 047;").unwrap();
        let const_generated = ConstantDeclaration::from(parsed.next().unwrap());
        let const_coded = ConstantDeclaration {
            name: "CON".to_string(),
            value: Value::Numeric { val: 39 },
        };
        assert!(const_generated == const_coded, "Constant parsing wrong");

        // Code-gen
        let rust_code: TokenStream = quote!(
            const CON: i64 = 39i64;
        );
        let generated_code: TokenStream = (&const_generated).into();
        assert!(
            generated_code.to_string() == rust_code.to_string(),
            "DataType: Generated code wrong:\n{}\n{}",
            generated_code.to_string(),
            rust_code.to_string()
        );
    }
}

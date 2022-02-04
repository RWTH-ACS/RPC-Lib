use crate::parser::parser::Rule;

use proc_macro2::TokenStream;
use quote::quote;

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
        let name = &constant.name;
        let value: TokenStream = (&constant.value).into();
        quote!(const #name = #value)
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
        }.into()
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
            Rule::constant => {
                return Value::Numeric {
                    val: parse_num(token),
                }
            }
            Rule::identifier => {
                return Value::Named {
                    name: token.as_str().to_string(),
                }
            }
            _ => panic!("Syntax Error"),
        }
    }
}

impl From<pest::iterators::Pair<'_, Rule>> for ConstantDeclaration {
    fn from(constant_def: pest::iterators::Pair<'_, Rule>) -> ConstantDeclaration {
        let mut constant = ConstantDeclaration {
            name: "".to_string(),
            value: Value::Numeric { val: 0 },
        };
        for rule in constant_def.into_inner() {
            match rule.as_rule() {
                Rule::identifier => {
                    constant.name = rule.as_str().to_string();
                }
                Rule::constant => {
                    constant.value = Value::Numeric {
                        val: parse_num(rule),
                    }
                }
                _ => println!("Syntax Error"),
            }
        }
        constant
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parser::RPCLParser;
    use crate::pest::Parser;

    #[test]
    fn parse_constant_decimal() {
        let mut parsed = RPCLParser::parse(Rule::constant_def, "const CON = 23;").unwrap();
        let constant = ConstantDeclaration::from(parsed.next().unwrap());

        assert!(
            constant.value == Value::Numeric { val: 23 },
            "Value of constant wrong"
        );
        assert!(constant.name == "CON", "Name of constant wrong");
    }

    #[test]
    fn parse_constant_hexadecimal() {
        let mut parsed = RPCLParser::parse(Rule::constant_def, "const CON2 = 0x2889;").unwrap();
        let constant = ConstantDeclaration::from(parsed.next().unwrap());

        assert!(
            constant.value == Value::Numeric { val: 0x2889 },
            "Value of constant wrong"
        );
        assert!(constant.name == "CON2", "Name of constant wrong");
    }

    #[test]
    fn parse_constant_negative_decimal() {
        let mut parsed = RPCLParser::parse(Rule::constant_def, "const CON = -68;").unwrap();
        let constant = ConstantDeclaration::from(parsed.next().unwrap());

        assert!(
            constant.value == Value::Numeric { val: -68 },
            "Value of constant wrong"
        );
        assert!(constant.name == "CON", "Name of constant wrong");
    }

    #[test]
    fn parse_constant_octal() {
        let mut parsed = RPCLParser::parse(Rule::constant_def, "const CON = 047;").unwrap();
        let constant = ConstantDeclaration::from(parsed.next().unwrap());

        assert!(
            constant.value == Value::Numeric { val: 39 },
            "Value of constant wrong"
        );
        assert!(constant.name == "CON", "Name of constant wrong");
    }
}

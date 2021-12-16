use crate::parser::parser::Rule;
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

impl From<DataType> for TokenStream {
    fn from(data_type: DataType) -> TokenStream {
        match data_type {
            DataType::Integer { length, signed } => {
                match signed {
                    true => {
                        match length {
                            32 => quote!(i32),
                            64 => quote!(i64),
                            _ => panic!(""),
                        }
                    }
                    false => {
                        match length {
                            32 => quote!(u32),
                            64 => quote!(u64),
                            _ => panic!(""),
                        }
                    }
                }
            }
            DataType::Float { length } => {
                match length {
                    32 => quote!(f32),
                    64 => quote!(f64),
                    _ => panic!(""),
                }
            }
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
            DataType::Struct { def } => {
                panic!();
            }
            DataType::Union { def } => {
                panic!();
            }
            DataType::Enum { def } => {
                panic!();
            }
            DataType::Void => {
                quote!()
            }
        }.into()
    }
}

fn parse_primitive(primitive_type: pest::iterators::Pair<'_, Rule>) -> DataType {
    match primitive_type.as_str() {
        "unsigned int" => {
            return DataType::Integer {
                length: 32,
                signed: false,
            }
        }
        "int" => {
            return DataType::Integer {
                length: 32,
                signed: true,
            }
        }
        "unsigned hyper" => {
            return DataType::Integer {
                length: 64,
                signed: false,
            }
        }
        "hyper" => {
            return DataType::Integer {
                length: 64,
                signed: true,
            }
        }
        "float" => return DataType::Float { length: 32 },
        "double" => return DataType::Float { length: 64 },
        "quadruple" => return DataType::Float { length: 128 },
        "bool" => return DataType::Boolean,
        "string" | "string<>" => return DataType::String,
        _ => panic!("Syntax Error"),
    }
}

impl From<pest::iterators::Pair<'_, Rule>> for DataType {
    fn from(type_specifier: pest::iterators::Pair<'_, Rule>) -> DataType {
        // type_specifier > inner_rule (e.g. primitve_type, enum_type_spec, ...)
        let inner_rule = type_specifier.into_inner().next().unwrap();
        match inner_rule.as_rule() {
            Rule::primitive_type => {
                return parse_primitive(inner_rule);
            }
            Rule::void => {
                return DataType::Void;
            }
            Rule::enum_type_spec => {
                return DataType::Enum {
                    def: parse_enum_type_spec(inner_rule),
                };
            }
            Rule::union_type_spec => {
                return DataType::Union {
                    def: parse_union_type_spec(inner_rule),
                }
            }
            Rule::struct_type_spec => {
                return DataType::Struct {
                    def: parse_struct_type_spec(inner_rule),
                }
            }
            Rule::identifier => {
                return DataType::TypeDef {
                    name: inner_rule.as_str().to_string(),
                }
            }
            _ => panic!("Syntax Error"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::constant::Value;
    use super::*;
    use crate::parser::parser::RPCLParser;
    use crate::pest::Parser;

    #[test]
    fn parse_type_spec_primitive_1() {
        let mut parsed = RPCLParser::parse(Rule::type_specifier, "unsigned int").unwrap();
        let data = DataType::from(parsed.next().unwrap());

        assert!(
            data == DataType::Integer {
                length: 32,
                signed: false
            },
            "Datatype parsed wrong"
        );
    }

    #[test]
    fn parse_type_spec_primitive_2() {
        let mut parsed = RPCLParser::parse(Rule::type_specifier, "float").unwrap();
        let data = DataType::from(parsed.next().unwrap());

        assert!(
            data == DataType::Float { length: 32 },
            "Datatype parsed wrong"
        );
    }

    #[test]
    fn parse_type_spec_primitive_3() {
        let mut parsed = RPCLParser::parse(Rule::type_specifier, "bool").unwrap();
        let data = DataType::from(parsed.next().unwrap());

        assert!(data == DataType::Boolean, "Datatype parsed wrong");
    }

    #[test]
    fn parse_type_spec_custom_type_1() {
        let mut parsed = RPCLParser::parse(Rule::type_specifier, "MyCustom23Type").unwrap();
        let data = DataType::from(parsed.next().unwrap());

        assert!(
            data == DataType::TypeDef {
                name: "MyCustom23Type".to_string()
            },
            "Datatype parsed wrong"
        );
    }

    #[test]
    fn parse_enum_type_spec_1() {
        let mut parsed = RPCLParser::parse(Rule::type_specifier, "enum { A = 1 }").unwrap();
        let data = DataType::from(parsed.next().unwrap());

        assert!(
            data == DataType::Enum {
                def: Enum {
                    cases: vec![("A".into(), Value::Numeric { val: 1 })]
                }
            },
            "Datatype parsed wrong"
        );
    }
}

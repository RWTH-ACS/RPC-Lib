use crate::parser::parser::Rule;
use proc_macro2::TokenStream;
use quote::quote;

use super::constant::Value;
use super::datatype::DataType;

#[derive(PartialEq)]
pub enum DeclarationType {
    Optional,
    VarlenArray,
    FixedlenArray { length: Value },
    TypeNameDecl,
    VoidDecl,
}

#[derive(PartialEq)]
pub struct Declaration {
    pub decl_type: DeclarationType,
    pub data_type: DataType, // e.g. int(-array), (optional)char, (varlen-)double
    pub name: String,
}

impl From<Declaration> for TokenStream {
    fn from(decl: Declaration) -> TokenStream {
        let data_type: TokenStream = decl.data_type.into();
        let name = quote::format_ident!("{}", decl.name);
        match decl.decl_type {
            DeclarationType::Optional => {
                quote!(#name: std::Result<#data_type, i32>)
            }
            DeclarationType::VarlenArray => {
                quote!(#name: std::vec::Vec<#data_type>)
            }
            DeclarationType::FixedlenArray { length } => {
                let len: TokenStream = length.into();
                quote!(#name: [#data_type; #len])
            }
            DeclarationType::TypeNameDecl => {
                quote!(#name: #data_type)
            }
            DeclarationType::VoidDecl => {
                quote!()
            }
        }.into()
    }
}

fn parse_optional(pointer: pest::iterators::Pair<'_, Rule>) -> Declaration {
    // Optional Data (== union with TRUE && FALSE)
    let mut decl = Declaration {
        decl_type: DeclarationType::Optional,
        data_type: DataType::Void,
        name: "".to_string(),
    };
    for optional_token in pointer.into_inner() {
        match optional_token.as_rule() {
            Rule::type_specifier => {
                decl.data_type = DataType::from(optional_token);
            }
            Rule::identifier => {
                decl.name = optional_token.as_str().to_string();
            }
            _ => panic!("Syntax Error"),
        }
    }
    decl
}

fn parse_varlen_array(varlen_array: pest::iterators::Pair<'_, Rule>) -> Declaration {
    let mut decl = Declaration {
        decl_type: DeclarationType::VarlenArray,
        data_type: DataType::Void,
        name: "".to_string(),
    };
    for token in varlen_array.into_inner() {
        match token.as_rule() {
            Rule::type_specifier => {
                decl.data_type = DataType::from(token);
            }
            Rule::identifier => decl.name = token.as_str().to_string(),
            // May match rule "value", but as std::Vec is used, minimum length is not important
            Rule::value => {}
            _ => panic!("Syntax Error"),
        }
    }
    decl
}

fn parse_fixedlen_array(fixedlen_array: pest::iterators::Pair<'_, Rule>) -> Declaration {
    let mut decl = Declaration {
        decl_type: DeclarationType::VarlenArray,
        data_type: DataType::Void,
        name: "".to_string(),
    };
    for token in fixedlen_array.into_inner() {
        match token.as_rule() {
            Rule::type_specifier => {
                decl.data_type = DataType::from(token);
            }
            Rule::identifier => decl.name = token.as_str().to_string(),
            Rule::value => {
                decl.decl_type = DeclarationType::FixedlenArray {
                    length: Value::from(token),
                };
            }
            _ => panic!("Syntax Error"),
        }
    }
    decl
}

impl From<pest::iterators::Pair<'_, Rule>> for Declaration {
    fn from(declaration: pest::iterators::Pair<'_, Rule>) -> Declaration {
        let mut decl = Declaration {
            decl_type: DeclarationType::VoidDecl,
            data_type: DataType::Void,
            name: "".to_string(),
        };
        // declaration > inner_rule (e.g. pointer, string_decl, varlen_array)
        let inner_token = declaration.into_inner().next().unwrap();

        match inner_token.as_rule() {
            Rule::pointer => {
                return parse_optional(inner_token);
            }
            Rule::string_decl => {
                // String: name: string_decl > identifier
                let name = inner_token.into_inner().next().unwrap().as_str();
                return Declaration {
                    decl_type: DeclarationType::TypeNameDecl,
                    data_type: DataType::String,
                    name: name.to_string(),
                };
            }
            Rule::varlen_array => {
                return parse_varlen_array(inner_token);
            }
            Rule::fixedlen_array => {
                return parse_fixedlen_array(inner_token);
            }
            Rule::normal_type_name_decl => {
                let norm_decl_rule = inner_token.into_inner();
                decl.decl_type = DeclarationType::TypeNameDecl;
                for token in norm_decl_rule {
                    match token.as_rule() {
                        Rule::type_specifier => {
                            decl.data_type = DataType::from(token);
                        }
                        Rule::identifier => {
                            decl.name = token.as_str().to_string();
                        }
                        _ => println!("Syntax error"),
                    }
                }
            }
            Rule::void => {
                return Declaration {
                    decl_type: DeclarationType::VoidDecl,
                    data_type: DataType::Void,
                    name: "".to_string(),
                };
            }
            _ => println!("Syntax error"),
        }
        decl
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parser::RPCLParser;
    use crate::pest::Parser;

    #[test]
    fn decl_test_varlen_1() {
        let mut parsed = RPCLParser::parse(Rule::varlen_array, "unsigned int array<>").unwrap();
        let varlen_array = parse_varlen_array(parsed.next().unwrap());

        assert!(
            varlen_array.decl_type == DeclarationType::VarlenArray,
            "Varlen Array decl_type wrong"
        );
        assert!(
            varlen_array.data_type
                == DataType::Integer {
                    length: 32,
                    signed: false
                },
            "Varlen Array data_type wrong"
        );
        assert!(varlen_array.name == "array", "Varlen Array name wrong");
    }

    #[test]
    fn decl_test_varlen_2() {
        let mut parsed = RPCLParser::parse(Rule::varlen_array, "hyper array2_<>").unwrap();
        let varlen_array = parse_varlen_array(parsed.next().unwrap());

        assert!(
            varlen_array.decl_type == DeclarationType::VarlenArray,
            "Varlen Array decl_type wrong"
        );
        assert!(
            varlen_array.data_type
                == DataType::Integer {
                    length: 64,
                    signed: true
                },
            "Varlen Array data_type wrong"
        );
        assert!(varlen_array.name == "array2_", "Varlen Array name wrong");
    }

    #[test]
    fn decl_test_varlen_3() {
        let mut parsed = RPCLParser::parse(Rule::varlen_array, "CustomType _XR234z<2>").unwrap();
        let varlen_array = parse_varlen_array(parsed.next().unwrap());

        assert!(
            varlen_array.decl_type == DeclarationType::VarlenArray,
            "Varlen Array decl_type wrong"
        );
        assert!(
            varlen_array.data_type
                == DataType::TypeDef {
                    name: "CustomType".to_string()
                },
            "Varlen Array data_type wrong"
        );
        assert!(varlen_array.name == "_XR234z", "Varlen Array name wrong");
    }

    #[test]
    fn decl_test_fixedlen_1() {
        let mut parsed = RPCLParser::parse(Rule::fixedlen_array, "int arr[0x23]").unwrap();
        let fixedlen_array = parse_fixedlen_array(parsed.next().unwrap());

        assert!(
            fixedlen_array.decl_type
                == DeclarationType::FixedlenArray {
                    length: Value::Numeric { val: 0x23 }
                },
            "Fixedlen Array decl_type wrong"
        );
        assert!(
            fixedlen_array.data_type
                == DataType::Integer {
                    length: 32,
                    signed: true
                },
            "Fixedlen Array data_type wrong"
        );
        assert!(fixedlen_array.name == "arr", "Fixedlen Array name wrong");
    }

    #[test]
    fn decl_test_fixedlen_2() {
        let mut parsed = RPCLParser::parse(Rule::fixedlen_array, "CustomType _XR234z[25]").unwrap();
        let fixedlen_array = parse_fixedlen_array(parsed.next().unwrap());

        assert!(
            fixedlen_array.decl_type
                == DeclarationType::FixedlenArray {
                    length: Value::Numeric { val: 25 }
                },
            "Fixedlen Array decl_type wrong"
        );
        assert!(
            fixedlen_array.data_type
                == DataType::TypeDef {
                    name: "CustomType".to_string()
                },
            "Fixedlen Array data_type wrong"
        );
        assert!(
            fixedlen_array.name == "_XR234z",
            "Fixedlen Array name wrong"
        );
    }

    #[test]
    fn decl_test_type_name_decl() {
        let mut parsed = RPCLParser::parse(Rule::declaration, "CustomType name_23Z").unwrap();
        let declaration = Declaration::from(parsed.next().unwrap());

        assert!(
            declaration.decl_type == DeclarationType::TypeNameDecl,
            "Fixedlen Array decl_type wrong"
        );
        assert!(
            declaration.data_type
                == DataType::TypeDef {
                    name: "CustomType".to_string()
                },
            "Fixedlen Array data_type wrong"
        );
        assert!(declaration.name == "name_23Z", "Fixedlen Array name wrong");
    }

    #[test]
    fn decl_test_string_1() {
        let mut parsed = RPCLParser::parse(Rule::declaration, "string x<>").unwrap();
        let declaration = Declaration::from(parsed.next().unwrap());

        assert!(
            declaration.decl_type == DeclarationType::TypeNameDecl,
            "Fixedlen Array decl_type wrong"
        );
        assert!(
            declaration.data_type == DataType::String,
            "Fixedlen Array data_type wrong"
        );
        assert!(declaration.name == "x", "String-Declaration name wrong");
    }

    #[test]
    fn decl_test_string_2() {
        let mut parsed = RPCLParser::parse(Rule::declaration, "string _2x<24>").unwrap();
        let declaration = Declaration::from(parsed.next().unwrap());

        assert!(
            declaration.decl_type == DeclarationType::TypeNameDecl,
            "Fixedlen Array decl_type wrong"
        );
        assert!(
            declaration.data_type == DataType::String,
            "Fixedlen Array data_type wrong"
        );
        assert!(declaration.name == "_2x", "String-Declaration name wrong");
    }

    #[test]
    fn decl_test_optional_1() {
        let mut parsed = RPCLParser::parse(Rule::declaration, "CustomType *name_23Z").unwrap();
        let declaration = Declaration::from(parsed.next().unwrap());

        assert!(
            declaration.decl_type == DeclarationType::Optional,
            "Fixedlen Array decl_type wrong"
        );
        assert!(
            declaration.data_type
                == DataType::TypeDef {
                    name: "CustomType".to_string()
                },
            "Fixedlen Array data_type wrong"
        );
        assert!(declaration.name == "name_23Z", "Fixedlen Array name wrong");
    }

    #[test]
    fn decl_test_optional_2() {
        let mut parsed =
            RPCLParser::parse(Rule::declaration, "unsigned hyper *Optional_2_Int").unwrap();
        let declaration = Declaration::from(parsed.next().unwrap());

        assert!(
            declaration.decl_type == DeclarationType::Optional,
            "Fixedlen Array decl_type wrong"
        );
        assert!(
            declaration.data_type
                == DataType::Integer {
                    length: 64,
                    signed: false,
                },
            "Fixedlen Array data_type wrong"
        );
        assert!(
            declaration.name == "Optional_2_Int",
            "Fixedlen Array name wrong"
        );
    }
}

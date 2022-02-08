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

pub fn decl_type_to_rust(decl_type: &DeclarationType, data_type: &DataType) -> TokenStream {
        let data_type: TokenStream = data_type.into();
        match &decl_type {
            DeclarationType::Optional => {
                quote!(std::Option<#data_type>)
            }
            DeclarationType::VarlenArray => {
                quote!(std::vec::Vec<#data_type>)
            }
            DeclarationType::FixedlenArray { length } => {
                let len: TokenStream = length.into();
                quote!([#data_type; #len])
            }
            DeclarationType::TypeNameDecl => {
                quote!(#data_type)
            }
            DeclarationType::VoidDecl => {
                quote!()
            }
        }.into()
}

impl From<&Declaration> for TokenStream {
    fn from(decl: &Declaration) -> TokenStream {
        let name = quote::format_ident!("{}", decl.name);
        let decl_type_code = decl_type_to_rust(&decl.decl_type, &decl.data_type);
        if decl.decl_type != DeclarationType::VoidDecl {
            quote!( #name: #decl_type_code )
        }
        else {
            quote!()
        }
    }
}

fn parse_optional(pointer: pest::iterators::Pair<'_, Rule>) -> Declaration {
    // Optional Data (== union with TRUE && FALSE)
    let mut it = pointer.into_inner();
    let optional_type = it.next().unwrap();
    let optional_name = it.next().unwrap();
    Declaration {
        decl_type: DeclarationType::Optional,
        data_type: DataType::from(optional_type),
        name: optional_name.as_str().to_string(),
    }
}

fn parse_varlen_array(varlen_array: pest::iterators::Pair<'_, Rule>) -> Declaration {
    let mut it = varlen_array.into_inner();
    let varlen_type = it.next().unwrap();
    let varlen_name = it.next().unwrap();
    // Next may be match rule "value", but because std::Vec is used, minimum length of varlen-array is not important
    Declaration {
        decl_type: DeclarationType::VarlenArray,
        data_type: DataType::from(varlen_type),
        name: varlen_name.as_str().to_string(),
    }
}

fn parse_fixedlen_array(fixedlen_array: pest::iterators::Pair<'_, Rule>) -> Declaration {
    let mut it = fixedlen_array.into_inner();
    let fixedlen_type = it.next().unwrap();
    let fixedlen_name = it.next().unwrap();
    let fixedlen_len = it.next().unwrap();
    Declaration {
        decl_type: DeclarationType::FixedlenArray { length: Value::from(fixedlen_len) },
        data_type: DataType::from(fixedlen_type),
        name: fixedlen_name.as_str().to_string(),
    }
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
        // Parser
        let mut parsed = RPCLParser::parse(Rule::varlen_array, "unsigned int array<>").unwrap();
        let decl_generated = parse_varlen_array(parsed.next().unwrap());
        let decl_coded = Declaration {
            decl_type: DeclarationType::VarlenArray,
            data_type: DataType::Integer { length: 32, signed: false },
            name: "array".to_string(),
        };
        assert!(decl_generated == decl_coded, "Declaration parsing wrong");

        // Code-gen
        let rust_code: TokenStream = quote!{ array: std::vec::Vec<u32> };
        let generated_code: TokenStream = (&decl_generated).into();
        assert!(generated_code.to_string() == rust_code.to_string(), "Declaration: Generated code wrong:\n{}\n{}", generated_code.to_string() , rust_code.to_string());
    }

    #[test]
    fn decl_test_varlen_2() {
        // Parser
        let mut parsed = RPCLParser::parse(Rule::varlen_array, "hyper array2_<>").unwrap();
        let decl_generated = parse_varlen_array(parsed.next().unwrap());
        let decl_coded = Declaration {
            decl_type: DeclarationType::VarlenArray,
            data_type: DataType::Integer { length: 64, signed: true },
            name: "array2_".to_string(),
        };
        assert!(decl_generated == decl_coded, "Declaration parsing wrong");

        // Code-gen
        let rust_code: TokenStream = quote!{ array2_: std::vec::Vec<i64> };
        let generated_code: TokenStream = (&decl_generated).into();
        assert!(generated_code.to_string() == rust_code.to_string(), "Declaration: Generated code wrong:\n{}\n{}", generated_code.to_string() , rust_code.to_string());
    }

    #[test]
    fn decl_test_varlen_3() {
        // Parser
        let mut parsed = RPCLParser::parse(Rule::varlen_array, "CustomType _XR234z<2>").unwrap();
        let decl_generated = parse_varlen_array(parsed.next().unwrap());
        let decl_coded = Declaration {
            decl_type: DeclarationType::VarlenArray,
            data_type: DataType::TypeDef { name: "CustomType".to_string() },
            name: "_XR234z".to_string(),
        };
        assert!(decl_generated == decl_coded, "Declaration parsing wrong");

        // Code-gen
        let rust_code: TokenStream = quote!{ _XR234z: std::vec::Vec<CustomType> };
        let generated_code: TokenStream = (&decl_generated).into();
        assert!(generated_code.to_string() == rust_code.to_string(), "Declaration: Generated code wrong:\n{}\n{}", generated_code.to_string() , rust_code.to_string());
    }

    #[test]
    fn decl_test_fixedlen_1() {
        // Parser
        let mut parsed = RPCLParser::parse(Rule::fixedlen_array, "int arr[0x23]").unwrap();
        let decl_generated = parse_fixedlen_array(parsed.next().unwrap());
        let decl_coded = Declaration {
            decl_type: DeclarationType::FixedlenArray { length: Value::Numeric { val: 0x23 } },
            data_type: DataType::Integer { length: 32, signed: true },
            name: "arr".to_string(),
        };
        assert!(decl_generated == decl_coded, "Declaration parsing wrong");

        // Code-gen
        let rust_code: TokenStream = quote!{ arr: [i32; 35i64] };
        let generated_code: TokenStream = (&decl_generated).into();
        assert!(generated_code.to_string() == rust_code.to_string(), "Declaration: Generated code wrong:\n{}\n{}", generated_code.to_string() , rust_code.to_string());
    }

    #[test]
    fn decl_test_fixedlen_2() {
        // Parser
        let mut parsed = RPCLParser::parse(Rule::fixedlen_array, "CustomType _XR234z[25]").unwrap();
        let decl_generated = parse_fixedlen_array(parsed.next().unwrap());
        let decl_coded = Declaration {
            decl_type: DeclarationType::FixedlenArray { length: Value::Numeric { val: 25 } },
            data_type: DataType::TypeDef { name: "CustomType".to_string() },
            name: "_XR234z".to_string(),
        };
        assert!(decl_generated == decl_coded, "Declaration parsing wrong");

        // Code-gen
        let rust_code: TokenStream = quote!{ _XR234z: [CustomType; 25i64] };
        let generated_code: TokenStream = (&decl_generated).into();
        assert!(generated_code.to_string() == rust_code.to_string(), "Declaration: Generated code wrong:\n{}\n{}", generated_code.to_string() , rust_code.to_string());
    }

    #[test]
    fn decl_test_type_name_decl() {
        // Parser
        let mut parsed = RPCLParser::parse(Rule::declaration, "CustomType name_23Z").unwrap();
        let decl_generated = Declaration::from(parsed.next().unwrap());
        let decl_coded = Declaration {
            decl_type: DeclarationType::TypeNameDecl,
            data_type: DataType::TypeDef { name: "CustomType".to_string() },
            name: "name_23Z".to_string(),
        };
        assert!(decl_generated == decl_coded, "Declaration parsing wrong");

        // Code-gen
        let rust_code: TokenStream = quote!{ name_23Z: CustomType };
        let generated_code: TokenStream = (&decl_generated).into();
        assert!(generated_code.to_string() == rust_code.to_string(), "Declaration: Generated code wrong:\n{}\n{}", generated_code.to_string() , rust_code.to_string());
    }

    #[test]
    fn decl_test_string_1() {
        // Parser
        let mut parsed = RPCLParser::parse(Rule::declaration, "string x<>").unwrap();
        let decl_generated = Declaration::from(parsed.next().unwrap());
        let decl_coded = Declaration {
            decl_type: DeclarationType::TypeNameDecl,
            data_type: DataType::String,
            name: "x".to_string(),
        };
        assert!(decl_generated == decl_coded, "Declaration parsing wrong");

        // Code-gen
        let rust_code: TokenStream = quote!{ x: String };
        let generated_code: TokenStream = (&decl_generated).into();
        assert!(generated_code.to_string() == rust_code.to_string(), "Declaration: Generated code wrong:\n{}\n{}", generated_code.to_string() , rust_code.to_string());
    }

    #[test]
    fn decl_test_string_2() {
        // Parser
        let mut parsed = RPCLParser::parse(Rule::declaration, "string _2x<24>").unwrap();
        let decl_generated = Declaration::from(parsed.next().unwrap());
        let decl_coded = Declaration {
            decl_type: DeclarationType::TypeNameDecl,
            data_type: DataType::String,
            name: "_2x".to_string(),
        };
        assert!(decl_generated == decl_coded, "Declaration parsing wrong");

        // Code-gen
        let rust_code: TokenStream = quote!{ _2x: String };
        let generated_code: TokenStream = (&decl_generated).into();
        assert!(generated_code.to_string() == rust_code.to_string(), "Declaration: Generated code wrong:\n{}\n{}", generated_code.to_string() , rust_code.to_string());
    }

    #[test]
    fn decl_test_optional_1() {
        // Parser
        let mut parsed = RPCLParser::parse(Rule::declaration, "CustomType *name_23Z").unwrap();
        let decl_generated = Declaration::from(parsed.next().unwrap());
        let decl_coded = Declaration {
            decl_type: DeclarationType::Optional,
            data_type: DataType::TypeDef { name: "CustomType".to_string() },
            name: "name_23Z".to_string(),
        };
        assert!(decl_generated == decl_coded, "Declaration parsing wrong");

        // Code-gen
        let rust_code: TokenStream = quote!( name_23Z: std::Option<CustomType>);
        let generated_code: TokenStream = (&decl_generated).into();
        assert!(generated_code.to_string() == rust_code.to_string(), "Declaration: Generated code wrong:\n{}\n{}", generated_code.to_string() , rust_code.to_string());
    }

    #[test]
    fn decl_test_optional_2() {
        let mut parsed = RPCLParser::parse(Rule::declaration, "unsigned hyper *Optional_2_Int").unwrap();
        let decl_generated = Declaration::from(parsed.next().unwrap());
        let decl_coded = Declaration {
            decl_type: DeclarationType::Optional,
            data_type: DataType::Integer{ length: 64, signed: false },
            name: "Optional_2_Int".to_string(),
        };
        assert!(decl_generated == decl_coded, "Declaration parsing wrong");

        // Code-gen
        let rust_code: TokenStream = quote!( Optional_2_Int: std::Option<u64>);
        let generated_code: TokenStream = (&decl_generated).into();
        assert!(generated_code.to_string() == rust_code.to_string(), "Declaration: Generated code wrong:\n{}\n{}", generated_code.to_string() , rust_code.to_string());
    }
}

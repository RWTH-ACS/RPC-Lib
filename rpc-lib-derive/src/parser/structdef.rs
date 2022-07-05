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

use super::declaration::Declaration;

#[derive(PartialEq)]
pub struct Structdef {
    name: String,
    struct_body: Struct,
}

#[derive(PartialEq)]
pub struct Struct {
    pub fields: std::vec::Vec<Declaration>,
}

impl From<&Structdef> for TokenStream {
    fn from(struct_def: &Structdef) -> TokenStream {
        // Name
        let name = format_ident!("{}", struct_def.name);
        let struct_body = &struct_def.struct_body;

        // Struct Body
        let mut struct_code = quote!();
        for field in &struct_body.fields {
            let field_name = format_ident!("{}", &field.name);
            let field_type = TokenStream::from(&field.data_type);
            struct_code = quote!( #struct_code #field_name: #field_type, );
        }
        quote! {
            #[derive(Debug)]
            #[derive(::rpc_lib::XdrDeserialize, ::rpc_lib::XdrSerialize)]
            struct #name {
                #struct_code
            }
        }
    }
}

impl From<&Struct> for TokenStream {
    fn from(st: &Struct) -> TokenStream {
        let mut code = quote!();
        for decl in &st.fields {
            let decl: TokenStream = decl.into();
            code = quote!(#code #decl,);
        }
        quote!( { #code } )
    }
}

pub fn parse_struct_type_spec(struct_type_spec: pest::iterators::Pair<'_, Rule>) -> Struct {
    Struct::from(struct_type_spec.into_inner().next().unwrap())
}

impl From<pest::iterators::Pair<'_, Rule>> for Structdef {
    fn from(struct_def: pest::iterators::Pair<'_, Rule>) -> Structdef {
        let mut iter = struct_def.into_inner();
        let name = iter.next().unwrap();
        let struct_body = iter.next().unwrap();

        Structdef {
            name: name.as_str().to_string(),
            struct_body: Struct::from(struct_body),
        }
    }
}

impl From<pest::iterators::Pair<'_, Rule>> for Struct {
    fn from(struct_body: pest::iterators::Pair<'_, Rule>) -> Struct {
        let mut st = Struct {
            fields: std::vec::Vec::new(),
        };
        for token in struct_body.into_inner() {
            match token.as_rule() {
                Rule::declaration => {
                    st.fields.push(Declaration::from(token));
                }
                _ => panic!("Syntax Error"),
            }
        }
        st
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::RPCLParser;
    use pest::Parser;

    use super::super::datatype::*;
    use super::super::declaration::*;

    #[test]
    fn parse_struct_1() {
        // Parser
        let mut parsed = RPCLParser::parse(Rule::struct_body, "{ int x; double f; }").unwrap();
        let struct_body = Struct::from(parsed.next().unwrap());

        let st = Struct {
            fields: vec![
                Declaration {
                    decl_type: DeclarationType::TypeNameDecl,
                    data_type: DataType::Integer {
                        length: 32,
                        signed: true,
                    },
                    name: "x".into(),
                },
                Declaration {
                    decl_type: DeclarationType::TypeNameDecl,
                    data_type: DataType::Float { length: 64 },
                    name: "f".into(),
                },
            ],
        };
        assert!(st == struct_body, "Struct Body wrong");

        // Code-gen
        let rust_code: TokenStream = quote! {
            { x: i32, f: f64, }
        };
        let generated_code: TokenStream = (&struct_body).into();
        assert!(
            generated_code.to_string() == rust_code.to_string(),
            "Struct: Generated code wrong:\n{}\n{}",
            generated_code.to_string(),
            rust_code.to_string()
        );
    }

    #[test]
    fn parse_struct_2() {
        // Parser
        let mut parsed = RPCLParser::parse(
            Rule::struct_body,
            "{ unsigned hyper x1_; MyCustomType_2 f; }",
        )
        .unwrap();
        let struct_body = Struct::from(parsed.next().unwrap());

        let st = Struct {
            fields: vec![
                Declaration {
                    decl_type: DeclarationType::TypeNameDecl,
                    data_type: DataType::Integer {
                        length: 64,
                        signed: false,
                    },
                    name: "x1_".into(),
                },
                Declaration {
                    decl_type: DeclarationType::TypeNameDecl,
                    data_type: DataType::TypeDef {
                        name: "MyCustomType_2".into(),
                    },
                    name: "f".into(),
                },
            ],
        };
        assert!(st == struct_body, "Struct Body wrong");

        // Code-gen
        let rust_code: TokenStream = quote! {
            { x1_: u64, f: MyCustomType_2, }
        };
        let generated_code: TokenStream = (&struct_body).into();
        assert!(
            generated_code.to_string() == rust_code.to_string(),
            "Struct: Generated code wrong:\n{}\n{}",
            generated_code.to_string(),
            rust_code.to_string()
        );
    }

    #[test]
    fn parse_struct_def() {
        // Parser
        let mut parsed = RPCLParser::parse(
            Rule::struct_def,
            "struct MyStruct_ { int x; double f; MyType t; };",
        )
        .unwrap();
        let struct_def = Structdef::from(parsed.next().unwrap());

        let st = Structdef {
            name: "MyStruct_".to_string(),
            struct_body: Struct {
                fields: vec![
                    Declaration {
                        decl_type: DeclarationType::TypeNameDecl,
                        data_type: DataType::Integer {
                            length: 32,
                            signed: true,
                        },
                        name: "x".into(),
                    },
                    Declaration {
                        decl_type: DeclarationType::TypeNameDecl,
                        data_type: DataType::Float { length: 64 },
                        name: "f".into(),
                    },
                    Declaration {
                        decl_type: DeclarationType::TypeNameDecl,
                        data_type: DataType::TypeDef {
                            name: "MyType".into(),
                        },
                        name: "t".into(),
                    },
                ],
            },
        };
        assert!(struct_def == st, "Struct Def wrong");

        // Code-gen
        let rust_code: TokenStream = quote! {
            #[derive(Debug)]
            #[derive(::rpc_lib::XdrDeserialize, ::rpc_lib::XdrSerialize)]
            struct MyStruct_ {
                x: i32,
                f: f64,
                t: MyType,
            }
        }
        .into();
        let generated_code: TokenStream = (&struct_def).into();
        assert!(
            generated_code.to_string() == rust_code.to_string(),
            "Struct: Generated code wrong:\n{}\n{}",
            generated_code.to_string(),
            rust_code.to_string()
        );
    }

    #[test]
    fn parse_struct_type_spec_1() {
        // Parser
        let mut parsed = RPCLParser::parse(Rule::struct_type_spec, "struct { int x; }").unwrap();
        let struct_body = parse_struct_type_spec(parsed.next().unwrap());

        let st = Struct {
            fields: vec![Declaration {
                decl_type: DeclarationType::TypeNameDecl,
                data_type: DataType::Integer {
                    length: 32,
                    signed: true,
                },
                name: "x".into(),
            }],
        };
        assert!(struct_body == st, "Struct Type Spec wrong");

        // Code-gen
        let rust_code: TokenStream = quote! {
            { x: i32, }
        };
        let generated_code: TokenStream = (&struct_body).into();
        assert!(
            generated_code.to_string() == rust_code.to_string(),
            "Struct: Generated code wrong:\n{}\n{}",
            generated_code.to_string(),
            rust_code.to_string()
        );
    }
}

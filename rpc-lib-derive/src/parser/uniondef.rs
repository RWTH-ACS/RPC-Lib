// Copyright 2022 Philipp Fensch
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::parser::declaration::decl_type_to_rust;
use crate::parser::Rule;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use super::constant::Value;
use super::datatype::DataType;
use super::declaration::{Declaration, DeclarationType};

#[derive(PartialEq)]
enum DiscriminantType {
    Int,
    UnsignedInt,
    Boolean,
    Enum { name: String },
}

#[derive(PartialEq)]
pub struct Uniondef {
    name: String,
    union_body: Union,
}

#[derive(PartialEq)]
pub struct Union {
    discriminant: DiscriminantType,
    cases: std::vec::Vec<(Value, Declaration)>,
    default: std::boxed::Box<Declaration>,
}

fn make_deserialize_function_code(union: &Union) -> TokenStream {
    let mut match_code = quote!();
    match &union.discriminant {
        DiscriminantType::Int => {
            // Cases:
            for (case_val, data_decl) in &union.cases {
                let number = *match case_val {
                    Value::Numeric { val } => val,
                    _ => panic!("Union: Case has to be integer when discriminanttype is int!"),
                } as i32;
                let case_ident = format_ident!("Case{}", number as u32);
                if data_decl.decl_type != DeclarationType::VoidDecl {
                    let name = quote::format_ident!("{}", data_decl.name);
                    let decl_type = decl_type_to_rust(&data_decl.decl_type, &data_decl.data_type);
                    match_code = quote!( #match_code #number => Self :: #case_ident { #name: <#decl_type>::deserialize(&mut reader)? }, );
                } else {
                    match_code = quote!( #match_code #number => Self :: #case_ident, );
                }
            }

            // Default-Case:
            match_code = quote!( #match_code i => Self :: CaseDefault(i), );
        }
        DiscriminantType::UnsignedInt => panic!("Unsigned int as discriminant not implemented yet"),
        DiscriminantType::Boolean => panic!("Boolean as discriminant not implemented yet"),
        DiscriminantType::Enum { name: _ } => panic!("Enum as discriminant not implemented yet"),
    }

    // Construct Function:
    quote! {
        fn deserialize(mut reader: impl ::std::io::Read) -> ::std::io::Result<Self> {
            let err_code = i32::deserialize(&mut reader)?;
            Ok(match err_code {
                #match_code
                _ => panic!("Unknown field of discriminated union with Field-Value {}", err_code),
            })
        }
    }
}

fn make_len_function_code(union: &Union) -> TokenStream {
    let mut match_arms = quote!();
    match &union.discriminant {
        DiscriminantType::Int => {
            // Cases:
            for (case_val, data_decl) in &union.cases {
                let number = *match case_val {
                    Value::Numeric { val } => val,
                    _ => panic!("Union: Case has to be integer when discriminanttype is int!"),
                } as i32;
                let case_ident = format_ident!("Case{}", number as u32);
                let decl_name = format_ident!("{}", data_decl.name);
                match_arms = quote! { #match_arms
                    Self :: #case_ident { #decl_name } => {
                        XdrSerialize::len(&#number) + XdrSerialize::len(&#decl_name)
                    }
                };
            }
            // Default-Case:
            match_arms = quote! { #match_arms
                Self::CaseDefault(i) =>  XdrSerialize::len(i),
            };
        }
        DiscriminantType::UnsignedInt => panic!("Unsigned int as discriminant not implemented yet"),
        DiscriminantType::Boolean => panic!("Boolean as discriminant not implemented yet"),
        DiscriminantType::Enum { name: _ } => panic!("Enum as discriminant not implemented yet"),
    }
    quote! {
        fn len(&self) -> usize {
            match self {
                #match_arms
            }
        }
    }
}

fn make_serialization_function_code(union: &Union) -> TokenStream {
    let mut match_arms = quote!();
    match &union.discriminant {
        DiscriminantType::Int => {
            // Cases:
            for (case_val, data_decl) in &union.cases {
                let number = *match case_val {
                    Value::Numeric { val } => val,
                    _ => panic!("Union: Case has to be integer when discriminanttype is int!"),
                } as i32;
                let case_ident = format_ident!("Case{}", number as u32);
                let decl_name = format_ident!("{}", data_decl.name);
                let decl_type = decl_type_to_rust(&data_decl.decl_type, &data_decl.data_type);
                match_arms = quote! { #match_arms
                    Self :: #case_ident { #decl_name } => {
                        i32::serialize(&#number, &mut writer)?;
                        <#decl_type>::serialize(&#decl_name, &mut writer)?;
                    }
                };
            }
            // Default-Case:
            match_arms = quote! { #match_arms
                Self::CaseDefault(i) => i32::serialize(&i, &mut writer)?,
            };
        }
        DiscriminantType::UnsignedInt => panic!("Unsigned int as discriminant not implemented yet"),
        DiscriminantType::Boolean => panic!("Boolean as discriminant not implemented yet"),
        DiscriminantType::Enum { name: _ } => panic!("Enum as discriminant not implemented yet"),
    }
    quote! {
        fn serialize(&self, mut writer: impl std::io::Write) -> std::io::Result<()> {
            match self {
                #match_arms
            }
            Ok(())
        }
    }
}

impl From<&Uniondef> for TokenStream {
    fn from(union_def: &Uniondef) -> TokenStream {
        let name = quote::format_ident!("{}", union_def.name);

        // Deserialize
        let mut union_body = quote!();
        for (val, decl) in &union_def.union_body.cases {
            let case_ident = match val {
                Value::Numeric { val } => val.to_string(),
                Value::Named { name } => name.to_string(),
            };
            let case_name = quote::format_ident!("Case{}", case_ident);
            match decl.data_type {
                DataType::Void => {
                    union_body = quote!( #union_body #case_name,);
                }
                _ => {
                    let decl_code = TokenStream::from(decl);
                    union_body = quote!( #union_body #case_name { #decl_code},);
                }
            }
        }

        let deserialization_func = make_deserialize_function_code(&union_def.union_body);
        let serialization_func = make_serialization_function_code(&union_def.union_body);
        let len_func = make_len_function_code(&union_def.union_body);

        let default_case = match union_def.union_body.discriminant {
            DiscriminantType::Int => {
                quote! {CaseDefault(i32)}
            }
            DiscriminantType::UnsignedInt => {
                panic!("Unsigned int as discriminant not implemented yet")
            }
            DiscriminantType::Boolean => panic!("Boolean as discriminant not implemented yet"),
            DiscriminantType::Enum { name: _ } => {
                panic!("Enum as discriminant not implemented yet")
            }
        };

        // Paste together
        quote! {
            #[derive(Debug)]
            enum #name {
                #union_body
                #default_case
            }

            impl XdrDeserialize for #name {
                #deserialization_func
            }

            impl XdrSerialize for #name {
                #len_func

                #serialization_func
            }
        }
    }
}

pub fn parse_union_type_spec(union_type_spec: pest::iterators::Pair<'_, Rule>) -> Union {
    Union::from(union_type_spec.into_inner().next().unwrap())
}

impl From<pest::iterators::Pair<'_, Rule>> for Uniondef {
    fn from(union_def: pest::iterators::Pair<'_, Rule>) -> Uniondef {
        let mut iter = union_def.into_inner();
        let name = iter.next().unwrap();
        let union_body = iter.next().unwrap();
        Uniondef {
            name: name.as_str().to_string(),
            union_body: Union::from(union_body),
        }
    }
}

fn parse_case_spec(case_spec: pest::iterators::Pair<'_, Rule>) -> (Value, Declaration) {
    let mut iter = case_spec.into_inner();
    let val = iter.next().unwrap();
    let decl = iter.next().unwrap();
    (Value::from(val), Declaration::from(decl))
}

impl From<pest::iterators::Pair<'_, Rule>> for Union {
    fn from(union_body: pest::iterators::Pair<'_, Rule>) -> Union {
        let mut union_def = Union {
            discriminant: DiscriminantType::Int,
            cases: std::vec::Vec::new(),
            default: std::boxed::Box::new(Declaration {
                decl_type: DeclarationType::VoidDecl,
                data_type: DataType::Void,
                name: "".into(),
            }),
        };
        for token in union_body.into_inner() {
            match token.as_rule() {
                Rule::discriminant_decl => {
                    let decl = Declaration::from(token.into_inner().next().unwrap());
                    union_def.discriminant = match decl.data_type {
                        DataType::Integer { length: _, signed } => {
                            if signed {
                                DiscriminantType::Int
                            } else {
                                DiscriminantType::UnsignedInt
                            }
                        }
                        DataType::Boolean => DiscriminantType::Boolean,
                        DataType::TypeDef { name } => DiscriminantType::Enum { name },
                        _ => panic!("Invalid Discriminant-Type in Union"),
                    };
                }
                Rule::case_spec => {
                    union_def.cases.push(parse_case_spec(token));
                }
                Rule::declaration => {
                    union_def.default = std::boxed::Box::new(Declaration::from(token));
                }
                _ => panic!("Syntax Error"),
            }
        }
        union_def
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::RPCLParser;
    use pest::Parser;

    #[test]
    fn parse_union_1() {
        // Parser
        let mut parsed = RPCLParser::parse(
            Rule::union_body,
            "switch(int x) {case X: int x; case Y2: unsigned hyper c; default: void; }",
        )
        .unwrap();
        let union_generated = Union::from(parsed.next().unwrap());
        let union_coded = Union {
            discriminant: DiscriminantType::Int,
            cases: vec![
                (
                    Value::Named { name: "X".into() },
                    Declaration::from(
                        RPCLParser::parse(Rule::declaration, "int x")
                            .unwrap()
                            .next()
                            .unwrap(),
                    ),
                ),
                (
                    Value::Named { name: "Y2".into() },
                    Declaration::from(
                        RPCLParser::parse(Rule::declaration, "unsigned hyper c")
                            .unwrap()
                            .next()
                            .unwrap(),
                    ),
                ),
            ],
            default: std::boxed::Box::new(Declaration {
                decl_type: DeclarationType::VoidDecl,
                data_type: DataType::Void,
                name: "".into(),
            }),
        };
        assert!(union_generated == union_coded, "Union parsing wrong");
    }

    #[test]
    #[should_panic(expected = "Unsigned int as discriminant not implemented yet")]
    fn parse_union_2() {
        // Parser
        let mut parsed = RPCLParser::parse(
            Rule::union_def,
            "union MyUnion switch(unsigned int err) {case 1: int y; default: void; };",
        )
        .unwrap();
        let union_generated = Uniondef::from(parsed.next().unwrap());
        let union_coded = Uniondef {
            name: "MyUnion".to_string(),
            union_body: Union {
                discriminant: DiscriminantType::UnsignedInt,
                cases: vec![(
                    Value::Numeric { val: 1 },
                    Declaration::from(
                        RPCLParser::parse(Rule::declaration, "int y")
                            .unwrap()
                            .next()
                            .unwrap(),
                    ),
                )],
                default: std::boxed::Box::new(Declaration {
                    decl_type: DeclarationType::VoidDecl,
                    data_type: DataType::Void,
                    name: "".into(),
                }),
            },
        };
        assert!(union_generated == union_coded, "Union parsing wrong");

        // Code-gen
        let rust_code: TokenStream = quote! {
            { CaseX { x: i32 }, CaseY2 { c: u64 }, CaseDefault, }
        };
        let generated_code: TokenStream = (&union_generated).into();
        assert!(
            generated_code.to_string() == rust_code.to_string(),
            "Union: Generated code wrong:\n{}\n{}",
            generated_code.to_string(),
            rust_code.to_string()
        );
    }

    #[test]
    fn parse_union_3() {
        let mut parsed = RPCLParser::parse(
            Rule::union_type_spec,
            "union switch(unsigned int err) {case 1: int y; default: void; }",
        )
        .unwrap();
        let union_body = parse_union_type_spec(parsed.next().unwrap());

        let un = Union {
            discriminant: DiscriminantType::UnsignedInt,
            cases: vec![(
                Value::Numeric { val: 1 },
                Declaration {
                    decl_type: DeclarationType::TypeNameDecl,
                    data_type: DataType::Integer {
                        length: 32,
                        signed: true,
                    },
                    name: "y".into(),
                },
            )],
            default: std::boxed::Box::new(Declaration {
                decl_type: DeclarationType::VoidDecl,
                data_type: DataType::Void,
                name: "".into(),
            }),
        };
        assert!(un == union_body, "Union Spec wrong");
    }

    #[test]
    fn parse_union_def() {
        // Parser
        let mut parsed = RPCLParser::parse(Rule::union_def, "union MyUnion2 switch(int err) {case 0: int result; case 2: float result; default: void; };").unwrap();
        let union_generated = Uniondef::from(parsed.next().unwrap());
        let union_coded = Uniondef {
            name: "MyUnion2".to_string(),
            union_body: Union {
                discriminant: DiscriminantType::Int,
                cases: vec![
                    (
                        Value::Numeric { val: 0 },
                        Declaration::from(
                            RPCLParser::parse(Rule::declaration, "int result")
                                .unwrap()
                                .next()
                                .unwrap(),
                        ),
                    ),
                    (
                        Value::Numeric { val: 2 },
                        Declaration::from(
                            RPCLParser::parse(Rule::declaration, "float result")
                                .unwrap()
                                .next()
                                .unwrap(),
                        ),
                    ),
                ],
                default: std::boxed::Box::new(Declaration {
                    decl_type: DeclarationType::VoidDecl,
                    data_type: DataType::Void,
                    name: "".into(),
                }),
            },
        };
        assert!(union_generated == union_coded, "Union parsing wrong");

        // Code-gen
        let rust_code: TokenStream = quote! {
            #[derive(Debug)]
            enum MyUnion2 { Case0 { result: i32 }, Case2 { result: f32 }, CaseDefault(i32) }
            impl XdrDeserialize for MyUnion2 {
                fn deserialize(mut reader: impl ::std::io::Read) -> ::std::io::Result<Self> {
                    let err_code = i32::deserialize(&mut reader)?;
                    Ok(match err_code {
                        0i32 => Self::Case0 { result: <i32>::deserialize(&mut reader)? },
                        2i32 => Self::Case2 { result: <f32>::deserialize(&mut reader)? },
                        i => Self::CaseDefault(i),
                        _ => panic!("Unknown field of discriminated union with Field-Value {}", err_code),
                    })
                }
            }
            impl XdrSerialize for MyUnion2 {
                fn len(&self) -> usize {
                    match self {
                        Self::Case0 { result } => {
                            XdrSerialize::len(&0i32) + XdrSerialize::len(&result)
                        }
                        Self::Case2 { result } => {
                            XdrSerialize::len(&2i32) + XdrSerialize::len(&result)
                        }
                        Self::CaseDefault(i) => XdrSerialize::len(i),
                    }
                }

                fn serialize(&self, mut writer: impl std::io::Write) -> std::io::Result<()> {
                    match self {
                        Self::Case0 { result } => {
                            i32::serialize(&0i32, &mut writer)?;
                            <i32>::serialize(&result, &mut writer)?;
                        }
                        Self::Case2 { result } => {
                            i32::serialize(&2i32, &mut writer)?;
                            <f32>::serialize(&result, &mut writer)?;
                        }
                        Self::CaseDefault(i) => i32::serialize(&i, &mut writer)?,
                    }
                    Ok(())
                }
            }
        };
        let generated_code: TokenStream = (&union_generated).into();
        assert!(
            generated_code.to_string() == rust_code.to_string(),
            "Union: Generated code wrong:\n{}\n{}",
            generated_code.to_string(),
            rust_code.to_string()
        );
    }
}

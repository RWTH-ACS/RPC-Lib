use crate::parser::parser::Rule;

use proc_macro2::TokenStream;
use quote::{quote, format_ident};

use super::constant::Value;
use super::datatype::DataType;
use super::declaration::{Declaration, DeclarationType};

#[derive(PartialEq)]
enum DiscriminantType {
    Int,
    UnsignedInt,
    Boolean,
    Enum{ name: String },
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
                    let decl: TokenStream = data_decl.into();
                    match_code = quote!( #match_code #number => Self :: #case_ident { #decl :: deserialize(bytes, parse_index) }, );
                } else {
                    match_code = quote!( #match_code #number => Self :: #case_ident, );
                }
            }

            // Default-Case:
            match_code = quote!( #match_code _ => Self :: CaseDefault, );
        }
        DiscriminantType::UnsignedInt => panic!("Unsigned int as discriminant not implemented yet"),
        DiscriminantType::Boolean => panic!("Boolean as discriminant not implemented yet"),
        DiscriminantType::Enum { name: _ } => panic!("Enum as discriminant not implemented yet"),
    }

    // Construct Function:
    quote! {
        fn deserialize(bytes: &Vec<u8>, parse_index: &mut usize) -> Self {
            let err_code = i32::deserialize(bytes, parse_index);
            match err_code {
                #match_code
                _ => panic!("Unknown field of discriminated union with Field-Value {}", err_code),
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
                let decl_type: TokenStream = (&data_decl.data_type).into();
                match_arms = quote!{ #match_arms
                    Self :: #case_ident { #decl_name } => {
                        vec.extend(i32::serialize(&#number));
                        vec.extend(<#decl_type>::serialize(&#decl_name));
                    }
                };
            }
            // Default-Case:
            match_arms = quote!{ #match_arms
                Self::CaseDefault => {}
            };
        }
        DiscriminantType::UnsignedInt => panic!("Unsigned int as discriminant not implemented yet"),
        DiscriminantType::Boolean => panic!("Boolean as discriminant not implemented yet"),
        DiscriminantType::Enum { name: _ } => panic!("Enum as discriminant not implemented yet"),
    }
    quote!{
        fn serialize(&self) -> std::vec::Vec<u8> {
            let mut vec: std::vec::Vec<u8> = std::vec::Vec::new();
            match self {
                #match_arms
            }
            vec
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
                    let data_type_code: TokenStream = (&decl.data_type).into();
                    let decl_name_code = quote::format_ident!("{}", decl.name);
                    union_body = quote!( #union_body #case_name { #decl_name_code: #data_type_code},);
                }
            }
        }

        // Functions for Trait Xdr:
        let deserialization_func = make_deserialize_function_code(&union_def.union_body);
        let serialization_func = make_serialization_function_code(&union_def.union_body);

        // Paste together
        quote!{
            enum #name {
                #union_body
                CaseDefault, 
            }

            impl Xdr for #name {

                #deserialization_func 

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
                        DataType::Boolean => {
                            DiscriminantType::Boolean
                        }
                        DataType::TypeDef { name } => {
                            DiscriminantType::Enum { name: name}
                        }
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
    use crate::parser::parser::RPCLParser;
    use crate::pest::Parser;

    #[test]
    fn parse_union_1() {
        let mut parsed = RPCLParser::parse(
            Rule::union_body,
            "switch(int x) {case X: int x; case Y2: unsigned hyper c; default: void; }",
        )
        .unwrap();
        let union_body = Union::from(parsed.next().unwrap());

        let un = Union {
            discriminant: DiscriminantType::Int,
            cases: vec![
                (
                    Value::Named { name: "X".into() },
                    Declaration {
                        decl_type: DeclarationType::TypeNameDecl,
                        data_type: DataType::Integer {
                            length: 32,
                            signed: true,
                        },
                        name: "x".into(),
                    },
                ),
                (
                    Value::Named { name: "Y2".into() },
                    Declaration {
                        decl_type: DeclarationType::TypeNameDecl,
                        data_type: DataType::Integer {
                            length: 64,
                            signed: false,
                        },
                        name: "c".into(),
                    },
                ),
            ],
            default: std::boxed::Box::new(Declaration {
                decl_type: DeclarationType::VoidDecl,
                data_type: DataType::Void,
                name: "".into(),
            }),
        };
        assert!(un == union_body, "Union Body wrong");
    }

    #[test]
    fn parse_union_2() {
        let mut parsed = RPCLParser::parse(
            Rule::union_def,
            "union MyUnion switch(unsigned int err) {case 1: int y; default: void; };",
        )
        .unwrap();
        let parsed_def = Uniondef::from(parsed.next().unwrap());

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
        assert!(
            Uniondef {
                name: "MyUnion".into(),
                union_body: un
            } == parsed_def,
            "Union Def wrong"
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
        let mut parsed = RPCLParser::parse(
            Rule::union_def,
            "union MyUnion2 switch(int err) {
                case 0:
                    int result;
                case 2:
                    float result;
                default:
                    void;
            };",
        )
        .unwrap();
        let parsed_def = Uniondef::from(parsed.next().unwrap());

        let un = Union {
            discriminant: DiscriminantType::Int,
            cases: vec![
                (
                    Value::Numeric { val: 0 },
                    Declaration {
                        decl_type: DeclarationType::TypeNameDecl,
                        data_type: DataType::Integer {
                            length: 32,
                            signed: true,
                        },
                        name: "result".into(),
                    },
                ),
                (
                    Value::Numeric { val: 2 },
                    Declaration {
                        decl_type: DeclarationType::TypeNameDecl,
                        data_type: DataType::Float { length: 32 },
                        name: "result".into(),
                    },
                ),
            ],
            default: std::boxed::Box::new(Declaration {
                decl_type: DeclarationType::VoidDecl,
                data_type: DataType::Void,
                name: "".into(),
            }),
        };
        assert!(un == parsed_def.union_body, "Union Def wrong");
    }
}

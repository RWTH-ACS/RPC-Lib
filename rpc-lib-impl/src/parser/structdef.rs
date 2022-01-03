use crate::parser::parser::Rule;

use proc_macro2::TokenStream;
use quote::quote;

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

impl From<Structdef> for TokenStream {
    fn from(struct_def: Structdef) -> TokenStream {
        // Name
        let name = quote::format_ident!("{}", struct_def.name);
        let struct_body = struct_def.struct_body;

        // Serialization
        let mut serialization_code = quote!();
        let mut deserialization_code = quote!();
        let mut struct_body_code = quote!();

        for field in struct_body.fields {
            let field_name = quote::format_ident!("{}", &field.name);
            let field_type: TokenStream = field.data_type.into();
            serialization_code = quote!{
                #serialization_code vec.extend(self.#field_name.serialize());
            };
            struct_body_code = quote!( #struct_body_code #field_name: #field_type, );
            deserialization_code = quote!( #deserialization_code #field_name: <#field_type> :: deserialize(bytes, parse_index), )
        }

        let code = quote!{
            impl Xdr for #name {
                fn serialize(&self) -> std::vec::Vec<u8> {
                    let mut vec: std::vec::Vec<u8> = std::vec::Vec::new();
                    #serialization_code
                    vec
                }

                fn deserialize(bytes: &Vec<u8>, parse_index: &mut usize) -> Self {
                    #name {
                        #deserialization_code
                    }
                }
            }
        };

        // Struct
        quote!{
            struct #name {
                #struct_body_code
            }
            #code
        }
    }
}

impl From<Struct> for TokenStream {
    fn from(st: Struct) -> TokenStream {
        let mut code = quote!();
        for decl in st.fields {
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
    use crate::parser::parser::RPCLParser;
    use crate::pest::Parser;

    use super::super::datatype::*;
    use super::super::declaration::*;

    #[test]
    fn parse_struct_1() {
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
    }

    #[test]
    fn parse_struct_2() {
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
    }

    #[test]
    fn parse_struct_def() {
        let mut parsed = RPCLParser::parse(
            Rule::struct_def,
            "struct MyStruct_ { int x; quadruple f; MyType t; };",
        )
        .unwrap();
        let struct_def = Structdef::from(parsed.next().unwrap());

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
                    data_type: DataType::Float { length: 128 },
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
        };
        assert!(
            struct_def.name == "MyStruct_".to_string(),
            "Struct Def name wrong"
        );
        assert!(struct_def.struct_body == st, "Struct Def Struct Body wrong");
    }

    #[test]
    fn parse_struct_type_spec_1() {
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
    }
}

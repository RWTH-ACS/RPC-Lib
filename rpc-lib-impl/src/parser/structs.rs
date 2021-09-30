use quote::__private::TokenStream as QuoteTokenStream;
use quote::{format_ident, quote};

use super::parser::Rule;
use super::util::*;

struct Field {
    field_type: String,
    field_name: String,
    field_kind: TypeKind,
    fixed_array_size: usize,
}

pub struct StructDef {
    pub identifier: String,
    contained_items: std::vec::Vec<Field>,
}

impl StructDef {
    pub fn from_pest(struct_rule: pest::iterators::Pair<'_, Rule>) -> StructDef {
        let mut def = StructDef {
            identifier: String::new(),
            contained_items: Vec::new(),
        };
        for item in struct_rule.into_inner() {
            match item.as_rule() {
                // Name of struct
                Rule::identifier => {
                    def.identifier = item.as_str().to_string();
                }
                // Fields of struct
                Rule::type_name_pair => {
                    let mut iter = item.into_inner();
                    let field_type_pair = iter.next().unwrap();
                    let field_name_pair = iter.next().unwrap();

                    let field = match field_name_pair.as_rule() {
                        Rule::identifier => Field {
                            field_type: field_type_pair.as_str().to_string(),
                            field_name: field_name_pair.as_str().to_string(),
                            field_kind: TypeKind::SimpleIdent,
                            fixed_array_size: 0,
                        },
                        Rule::array_fixed => {
                            let mut array_fixed_iter = field_name_pair.into_inner();
                            let name = array_fixed_iter.next().unwrap();
                            let size: usize = array_fixed_iter
                                .next()
                                .unwrap()
                                .as_str()
                                .parse::<usize>()
                                .unwrap();
                            Field {
                                field_type: field_type_pair.as_str().to_string(),
                                field_name: name.as_str().to_string(),
                                field_kind: TypeKind::FixedArray,
                                fixed_array_size: size,
                            }
                        }
                        Rule::array_varlen => Field {
                            field_type: field_type_pair.as_str().to_string(),
                            field_name: field_name_pair
                                .into_inner()
                                .next()
                                .unwrap()
                                .as_str()
                                .to_string(),
                            field_kind: TypeKind::VarlenArray,
                            fixed_array_size: 0,
                        },
                        _ => {
                            panic!("Error in Struct-Definition: {}", def.identifier)
                        }
                    };

                    def.contained_items.push(field);
                }
                _ => {}
            }
        }
        def
    }

    /// Generates Rust-Code
    pub fn to_rust_code(&self) -> QuoteTokenStream {
        let name = format_ident!("{}", self.identifier);

        let mut fields: QuoteTokenStream = quote!();
        let mut serialize_code: QuoteTokenStream = quote!();
        let mut deserialize_code: QuoteTokenStream = quote!();

        for field in &self.contained_items {
            // If type is not primitve then it is typedef or struct  (which are being defined somewhere above)
            let field_name = format_ident!("{}", field.field_name);

            // Type: Converting from C to Rust
            let type_code = match field.field_kind {
                TypeKind::SimpleIdent => {
                    let orig_type = format_ident!(
                        "{}",
                        convert_primitve_type(&field.field_type) // Example: unsigned int -> u32
                            .unwrap_or_else(|| &field.field_type) // Example: my_struct -> my_struct
                    );
                    quote!(#orig_type)
                }
                TypeKind::FixedArray => {
                    let orig_type = format_ident!(
                        "{}",
                        convert_primitve_type(&field.field_type) // Example: unsigned int -> u32
                            .unwrap_or_else(|| &field.field_type) // Example: my_struct -> my_struct
                    );
                    let arr_size = field.fixed_array_size;
                    quote!([#orig_type; #arr_size])
                }
                TypeKind::VarlenArray => {
                    let orig_type = format_ident!(
                        "Vec<{}>",
                        convert_primitve_type(&field.field_type)
                            .unwrap_or_else(|| &field.field_type)
                    );
                    quote!(#orig_type)
                }
            };

            serialize_code = quote! {
                #serialize_code
                vec.extend(self.#field_name.serialize());
            };

            deserialize_code = quote! {
                #deserialize_code
                #field_name: <#type_code> ::deserialize(bytes, parse_index),
            };

            fields = quote! {
                #fields
                #field_name: #type_code,
            };
        }

        let code = quote! {
            struct #name {
                #fields
            }

            impl Xdr for #name {
                fn serialize(&self) -> Vec<u8> {
                    let mut vec = Vec::new();
                    #serialize_code
                    vec
                }
            
                fn deserialize(bytes: &Vec<u8>, parse_index: &mut usize) -> #name {
                    #name {
                        #deserialize_code
                    }
                }
            }
        };
        code
    }
}

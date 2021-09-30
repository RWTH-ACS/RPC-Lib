use std::vec::*;

use quote::__private::TokenStream as QuoteTokenStream;
use quote::{format_ident, quote};

use super::parser::Rule;
use super::util::*;

pub struct FunctionDef {
    pub identifier: String,
    pub return_type: String,
    pub parameter: Vec<String>,
    pub value: u32,
}

impl FunctionDef {
    pub fn from_pest(func_rule: pest::iterators::Pair<'_, Rule>) -> FunctionDef {
        let mut def = FunctionDef {
            identifier: String::new(),
            return_type: String::new(),
            parameter: Vec::new(),
            value: 0,
        };
        for func_item in func_rule.into_inner() {
            match func_item.as_rule() {
                Rule::type_ident => {
                    def.return_type = func_item.as_str().to_string();
                }
                Rule::func_ident => {
                    def.identifier = func_item.as_str().to_string();
                }
                Rule::parameter_list => {
                    for parameter in func_item.into_inner() {
                        match parameter.as_rule() {
                            Rule::built_in_type | Rule::identifier => {
                                def.parameter.push(parameter.as_str().to_string());
                            }
                            _ => {
                                panic!("Invalid Parameter in Procedure {}", def.identifier);
                            }
                        }
                    }
                }
                Rule::integer => {
                    def.value = func_item.as_str().parse::<u32>().unwrap();
                }
                _ => panic!("Invalid Syntax in Function"),
            }
        }
        def
    }

    pub fn to_rust_code(&self) -> QuoteTokenStream {
        // Name of function
        let name = format_ident!("{}", self.identifier.to_lowercase().to_string());

        // Return-Type (Can be primtive or defined somewhere above)
        let has_ret_value = if self.return_type != "void" {
            true
        } else {
            false
        };
        let ret_type =
            convert_primitve_type(&self.return_type).unwrap_or_else(|| &self.return_type);
        // Format
        let return_type = format_ident!("{}", ret_type);

        // Function Body
        let mut function_body = quote!{
            let mut send_data = Vec::new();
        };


        // Parameter
        let mut parameter = quote!();
        let mut arg = 0;
        for param in &self.parameter {
            // Naming of parameter is not important!
            // Type can be primitve or already defined
            let param_type = convert_primitve_type(param).unwrap_or_else(|| param);
            let parameter_type = format_ident!("{}", param_type);

            // Parameter-Name ()
            let param_name_signature = format_ident!("x{}", arg.to_string());

            arg = arg + 1;
            parameter = quote! {
                #parameter #param_name_signature: #parameter_type,
            };

            // Serialization in Function Body
            function_body = quote!{
                #function_body
                send_data.extend(#param_name_signature.serialize());
            };
        }

        // Function Body: Send Request
        let num = self.value;
        function_body = quote!{
            #function_body

            let recv = rpc_lib::rpc_call(&self.client, #num, &send_data);

            // Parse ReplyHeader
            let mut parse_index = 0;
            let _response = rpc_lib::RpcReply::deserialize(&recv, &mut parse_index);
        };

        // Paste everything together
        let rust_signature = if has_ret_value {
            let rust_signature = quote! {
                pub fn #name(& self, #parameter) -> #return_type {
                    #function_body

                    #return_type ::deserialize(&recv, &mut parse_index)
                }
            };

            rust_signature
        } else {
            let rust_signature = quote! {
                pub fn #name(& self, #parameter) {
                    #function_body
                }
            };

            rust_signature
        };

        rust_signature
    }
}

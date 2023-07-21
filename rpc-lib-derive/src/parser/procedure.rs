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

use super::constant::Value;
use super::datatype::DataType;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum RawCallType {
    UnionI32,
    Struct,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Procedure {
    pub name: String,
    pub return_type: DataType,
    // bool stands for mutability
    pub args: std::vec::Vec<DataType>,
    pub num: Value,
    pub slice_call_target_type: Option<RawCallType>,
}

impl From<&Procedure> for TokenStream {
    fn from(proc: &Procedure) -> TokenStream {
        let proc_name = format_ident!("{}", proc.name);

        let arg_defs = proc
            .args
            .iter()
            .enumerate()
            .map(|(i, ty)| {
                let ty = TokenStream::from(ty);
                let ident = format_ident!("x{}", i);
                quote! {
                    #ident: &#ty,
                }
            })
            .collect::<TokenStream>();

        let arg = if !proc.args.is_empty() {
            let field_defs = proc
                .args
                .iter()
                .enumerate()
                .map(|(i, ty)| {
                    let ty = TokenStream::from(ty);
                    let ident = format_ident!("x{}", i);
                    quote! {
                        #ident: &'a #ty,
                    }
                })
                .collect::<TokenStream>();

            let field_idents = proc
                .args
                .iter()
                .enumerate()
                .map(|(i, _ty)| {
                    let ident = format_ident!("x{}", i);
                    quote! {
                        #ident,
                    }
                })
                .collect::<TokenStream>();

            quote! {
                {
                    #[derive(::rpc_lib::XdrSerialize)]
                    struct Args<'a> {
                        #field_defs
                    }

                    &Args {
                        #field_idents
                    }
                }
            }
        } else {
            quote!(())
        };

        let proc_num = TokenStream::from(&proc.num);
        if let Some(slice_target) = &proc.slice_call_target_type {
            match slice_target {
                RawCallType::UnionI32 => {
                    quote! { fn #proc_name <'a> (&mut self, target: &'a mut rpc_lib::RawResponseUnion<'a, i32>, #arg_defs ) -> std::io::Result<()> {
                        self.client.call_with_raw_union_response(#proc_num as u32, #arg, target)
                    }}
                }
                _ => unimplemented!("raw_return values are only supported for unions and typedefs"),
            }
        } else {
            if proc.return_type == DataType::Void {
                quote! { fn #proc_name(&self, #arg_defs) {}}
            } else {
                let return_type = TokenStream::from(&proc.return_type);
                quote! { fn #proc_name(&mut self, #arg_defs) -> std::io::Result<#return_type> {
                    self.client.call(#proc_num as u32, #arg)
                }}
            }
        }
    }
}

impl From<pest::iterators::Pair<'_, Rule>> for Procedure {
    fn from(procedure_def: pest::iterators::Pair<'_, Rule>) -> Procedure {
        let mut iter = procedure_def.into_inner();
        let proc_return = iter.next().unwrap();
        let proc_name = iter.next().unwrap();
        let proc_args = iter.next().unwrap();
        let proc_num = iter.next().unwrap();

        let mut arg_vec = std::vec::Vec::new();
        for arg in proc_args.into_inner() {
            if arg.as_rule() == Rule::type_specifier {
                arg_vec.push(DataType::from(arg));
            }
        }

        Procedure {
            name: proc_name.as_str().to_string(),
            return_type: DataType::from(proc_return.into_inner().next().unwrap()),
            args: arg_vec,
            num: Value::from(proc_num),
            slice_call_target_type: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::RPCLParser;
    use pest::Parser;

    #[test]
    fn parse_procedure_1() {
        // Parsing
        let mut parsed =
            RPCLParser::parse(Rule::procedure_def, "float PROC_NAME(int, float) = 1;").unwrap();
        let proc_generated = Procedure::from(parsed.next().unwrap());
        let proc_coded = Procedure {
            name: "PROC_NAME".to_string(),
            return_type: DataType::Float { length: 32 },
            args: vec![
                DataType::Integer {
                    length: 32,
                    signed: true,
                },
                DataType::Float { length: 32 },
            ],
            num: Value::Numeric { val: 1 },
            slice_call_target_type: None,
        };
        assert!(proc_generated == proc_coded, "Procedure parsing wrong");

        // Code-gen
        let rust_code: TokenStream = quote! {
            fn PROC_NAME(&mut self, x0: &i32, x1: &f32, ) -> std::io::Result<f32> {
                self.client.call(1i64 as u32, {
                    #[derive(::rpc_lib::XdrSerialize)]
                    struct Args<'a> {
                        x0: &'a i32,
                        x1: &'a f32,
                    }

                    &Args {
                        x0,
                        x1,
                    }
                })
            }
        };
        let generated_code: TokenStream = (&proc_generated).into();
        assert!(
            generated_code.to_string() == rust_code.to_string(),
            "Procedure: Generated code wrong:\n{}\n{}",
            generated_code.to_string(),
            rust_code.to_string()
        );
    }

    #[test]
    fn parse_procedure_2() {
        let mut parsed =
            RPCLParser::parse(Rule::procedure_def, "void PROC_NAME(void) = 0x24;").unwrap();
        let proc_generated = Procedure::from(parsed.next().unwrap());
        let proc_coded = Procedure {
            name: "PROC_NAME".to_string(),
            return_type: DataType::Void,
            args: vec![],
            num: Value::Numeric { val: 36 },
            slice_call_target_type: None,
        };
        assert!(proc_generated == proc_coded, "Procedure parsing wrong");

        // Code-gen
        let rust_code: TokenStream = quote! {
            fn PROC_NAME(&self, ) { }
        };
        let generated_code: TokenStream = (&proc_generated).into();
        assert!(
            generated_code.to_string() == rust_code.to_string(),
            "Procedure: Generated code wrong:\n{}\n{}",
            generated_code.to_string(),
            rust_code.to_string()
        );
    }
}

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

#[derive(PartialEq)]
pub struct Procedure {
    name: String,
    return_type: DataType,
    args: std::vec::Vec<DataType>,
    num: Value,
}

impl From<&Procedure> for TokenStream {
    fn from(proc: &Procedure) -> TokenStream {
        let proc_name = format_ident!("{}", proc.name);
        let mut args = quote!();
        let mut serialization_code = quote!( let mut send_data = std::vec::Vec::new(); );
        for (i, arg) in proc.args.iter().enumerate() {
            let arg_name = format_ident!("x{}", i);
            let arg_type: TokenStream = arg.into();
            args = quote!( #args #arg_name: &#arg_type,);
            serialization_code =
                quote!( #serialization_code #arg_name.serialize(&mut send_data)?; );
        }
        let proc_num = TokenStream::from(&proc.num);
        if proc.return_type == DataType::Void {
            quote! { fn #proc_name (&self, #args) { }}
        } else {
            let return_type = TokenStream::from(&proc.return_type);
            quote! { fn #proc_name (&mut self, #args) -> std::io::Result<#return_type> {
                // Parameter-Seralization
                #serialization_code

                // Call
                let recv = rpc_lib::rpc_call(&mut self.client, #proc_num as u32, &send_data)?;

                // Parse ReplyHeader
                <#return_type>::deserialize(&recv[..])
            }}
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
        };
        assert!(proc_generated == proc_coded, "Procedure parsing wrong");

        // Code-gen
        let rust_code: TokenStream = quote! {
            fn PROC_NAME(&mut self, x0: &i32, x1: &f32, ) -> std::io::Result<f32> {
                let mut send_data = std::vec::Vec::new();
                x0.serialize(&mut send_data)?;
                x1.serialize(&mut send_data)?;
                let recv = rpc_lib::rpc_call(&mut self.client, 1i64 as u32, &send_data)?;
                <f32>::deserialize(&recv[..])
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

// Copyright 2022 Philipp Fensch
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

mod constant;
mod datatype;
mod declaration;
mod enumdef;
mod procedure;
mod program;
mod structdef;
mod typedef;
mod uniondef;
mod xdr_spec;

use pest::Parser;
use proc_macro2::TokenStream;
use quote::quote;

use program::Program;
use xdr_spec::Specification;

#[derive(pest_derive::Parser)]
#[grammar = "rpcl.pest"]
pub struct RPCLParser;

pub fn parse(x_file: &str, struct_name: &str) -> (TokenStream, u32, u32) {
    let parsed = RPCLParser::parse(Rule::file, x_file).expect("Syntax Error in .x-File");
    let s_name = quote::format_ident!("{}", struct_name);

    let mut code = quote!();

    let mut spec = None;
    let mut program = None;
    for token in parsed {
        match token.as_rule() {
            Rule::specification => {
                if spec.is_some() {
                    unimplemented!("Separate spec sections are unimplemented. One would have to merge the two datastructs here...");
                }
                spec = Some(Specification::from(token));
            }
            Rule::program_def => {
                program = Some(Program::from(token));
            }
            _ => {}
        }
    }

    let mut program = program.expect("rpcl file without program is invalid");
    if let Some(spec) = &mut spec {
        spec.update_contains_vararray();
        program
            .versions
            .iter_mut()
            .for_each(|v| v.create_sliced_variants(&spec));
    }
    let program_number = program.program_number;
    let version_number = program.versions[0].version_number;

    let spec_code = if let Some(spec) = spec {
        TokenStream::from(&spec)
    } else {
        quote!()
    };
    let proc_code = TokenStream::from(&program);
    code = quote! {
        #code
        #spec_code
        use rpc_lib::{XdrDeserialize, XdrSerialize};
        impl #s_name {
            #proc_code
        }
    };
    (code, program_number, version_number)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_rule() {
        let file_str = "program PROG {
            version VERS {
                int FUNC(void) = 1;
            } = 1;
        } = 10;";
        let _parsed = RPCLParser::parse(Rule::file, file_str).expect("Syntax Error in .x-File");
    }

    #[test]
    fn test_file_rule_2() {
        let file_str = "struct X {
            int x;
            int y;
        };
        
        program PROG {
            version VERS {
                int FUNC(void) = 1;
            } = 1;
        } = 10;";
        let _parsed = RPCLParser::parse(Rule::file, file_str).expect("Syntax Error in .x-File");
    }
}

use pest::Parser;
use quote::__private::TokenStream as QuoteTokenStream;
use quote::{format_ident, quote};

use std::vec::*;

use super::functions::*;
use super::structs::*;
use super::typedefs::*;
use super::unions::*;

#[derive(Parser)]
#[grammar = "rpcl.pest"]
pub struct XFileParser;

pub fn parse(x_file: &String, struct_name: &String) -> (QuoteTokenStream, u32, u32) {
    let parsed = XFileParser::parse(Rule::file, x_file).expect("Syntax Error in .x-File");

    let mut struct_definitions: Vec<StructDef> = Vec::new();
    let mut union_definitions: Vec<UnionDef> = Vec::new();
    let mut type_definitions: Vec<TypeDef> = Vec::new();
    let mut function_definitions: Vec<FunctionDef> = Vec::new();

    let mut program_number = 0;
    let mut version_number = 0;

    for x in parsed {
        match x.as_rule() {
            Rule::struct_rule => {
                let def = StructDef::from_pest(x);
                struct_definitions.push(def);
            }
            Rule::union_rule => {
                let def = UnionDef::from_pest(x);
                union_definitions.push(def);
            }
            Rule::typedef_rule => {
                let def = TypeDef::from_pest(x);
                type_definitions.push(def);
            }
            Rule::program_rule => {
                let (defs, prog, vers) = program_rule(x);
                function_definitions = defs;
                program_number = prog;
                version_number = vers;
            }
            Rule::EOI => {}
            _ => {
                panic!("Not implemented: {:?}", x.as_rule())
            }
        }
    }

    let mut code = quote!();

    // Typedefs
    for def in &type_definitions {
        let typedef_code = def.to_rust_code();
        code = quote! {
            #code
            #typedef_code
        };
    }

    // Unions
    for def in &union_definitions {
        let union_code = def.to_rust_code();
        code = quote! {
            #code
            #union_code
        };
    }

    // Structs
    for def in &struct_definitions {
        let struct_code = def.to_rust_code();
        code = quote! {
            #code
            #struct_code
        }
    }

    // C-Bindings & Functions
    let mut function_block = quote!();
    for def in function_definitions {
        let function_code = def.to_rust_code();
        function_block = quote! {
            #function_block
            #function_code
        };
    }

    // pasting everything together
    let name = format_ident!("{}", struct_name);
    code = quote! {
        #code

        use crate::rpc_lib::Xdr;

        impl #name {
            #function_block
        }
    };

    (code, program_number, version_number)
}

fn program_rule(x: pest::iterators::Pair<'_, Rule>) -> (Vec<FunctionDef>, u32, u32) {
    // program_rule -> version_rule
    let mut program_iter = x.into_inner();
    let _program_name = program_iter.next().unwrap();
    let version_pair = program_iter.next().unwrap();

    let mut version_number = 0;
    let mut function_definitions: Vec<FunctionDef> = Vec::new();
    for item in version_pair.into_inner() {
        match item.as_rule() {
            Rule::identifier => {
                let _version_name = item.as_str();
            }
            Rule::integer => {
                version_number = item.as_str().parse::<u32>().unwrap();
            }
            Rule::function_rule => {
                let def = FunctionDef::from_pest(item);
                function_definitions.push(def);
            }
            _ => {}
        }
    }

    // Program Number
    let program_number = program_iter
        .next()
        .unwrap()
        .as_str()
        .parse::<u32>()
        .unwrap();

    (function_definitions, program_number, version_number)
}

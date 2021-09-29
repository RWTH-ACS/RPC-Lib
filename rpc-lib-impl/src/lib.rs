extern crate pest;
extern crate proc_macro;
extern crate quote;
#[macro_use]
extern crate pest_derive;

use proc_macro::TokenStream;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use quote::{format_ident, quote};

mod parser;

#[proc_macro_attribute]
pub fn include_rpcl(meta: TokenStream, item: TokenStream) -> TokenStream {
    // Get Name of .x-File
    let name_x_file: String = meta
        .into_iter()
        .next()
        .expect("Invalid use of Macro: include_rpcl(<Filename>)")
        .to_string();
    let len = name_x_file.len();
    let path = Path::new(&name_x_file[1..len - 1]);

    //Read .x-File
    let mut file = File::open(&path).expect("Couldn't open .x-File");
    let mut s = String::new();
    file.read_to_string(&mut s)
        .expect(&std::format!("Couldn't read {}", path.display()));
    println!("Parsing {}", path.display());

    //Extract Structname (struct <Name>;)
    let struct_name: String = item
        .into_iter()
        .skip(1)
        .next()
        .expect("Invalid Syntax: Must be: struct <Name>;")
        .to_string();

    //Parsing
    let (type_code, prog_num, ver_num) = parser::parse(&s, &struct_name);

    let name = format_ident!("{}", struct_name);
    let common_code = quote!{

        struct #name {
            client: rpc_lib::RpcClient
        }

        impl #name {
            fn new(address: &str) -> #name {
                #name {
                    client: rpc_lib::clnt_create(address, #prog_num, #ver_num)
                }
            }
        }
    };

    let code = quote!{
        #type_code
        #common_code
    };

    code.into()
}

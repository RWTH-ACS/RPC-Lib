// Copyright 2022 Philipp Fensch
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

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
    let (generated_code, prog_num, ver_num) = parser::parse(&s, &struct_name);

    let name = format_ident!("{}", struct_name);
    let doc_macro_call = std::format!("#[include_rpcl({})]", &name_x_file);
    let common_code = quote! {

        /// Contains connection to Rpc-Service and associated functions as defined in
        #[doc = #name_x_file]
        /// .
        ///
        /// # Examples
        /// 
        /// Creates a connection to 127.0.0.1, makes an Rpc-Call and prints the result.
        /// ```
        /// extern crate rpc_lib;
        /// use rpc_lib::include_rpcl;
        /// 
        #[doc = #doc_macro_call]
        /// struct RPCStruct;
        /// 
        /// fn main() {
        ///     let mut rpc = RPCStruct::new("127.0.0.1").expect("Server not available");
        ///     let result = rpc.MY_RPC_PROCEDURE(&1, &2).expect("Rpc call failed");
        ///     println!("MY_RPC_PROCEDURE returned: {}", result);
        /// }
        /// ```
        struct #name {
            client: rpc_lib::RpcClient
        }

        impl #name {
            /// Creates Connection to requested Rpc-Service.
            ///
            /// Connects to Portmapper-Service, gets Port-Number of requested Rpc-Service and
            /// connects to it.
            fn new(address: &str) -> std::io::Result<#name> {
                Ok(#name {
                    client: rpc_lib::clnt_create(address, #prog_num, #ver_num)?
                })
            }
        }
    };

    let code = quote! {
        #common_code
        #generated_code
    };

    code.into()
}

// Copyright 2022 Philipp Fensch
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::parser::parser::Rule;

use proc_macro2::TokenStream;
use quote::quote;

use super::datatype::DataType;
use super::declaration::{Declaration, DeclarationType, decl_type_to_rust};

#[derive(PartialEq)]
pub struct Typedef {
    name: String,
    orig_type: DataType,
    decl_type: DeclarationType,
}

impl From<&Typedef> for TokenStream {
    fn from(type_def: &Typedef) -> TokenStream {
        let type_code = decl_type_to_rust(&type_def.decl_type, &type_def.orig_type);
        let name = quote::format_ident!("{}", type_def.name);
        quote!(type #name = #type_code;)
    }
}

impl From<pest::iterators::Pair<'_, Rule>> for Typedef {
    fn from(type_def: pest::iterators::Pair<'_, Rule>) -> Typedef {
        let decl_token = type_def.into_inner().next().unwrap();
        let decl = Declaration::from(decl_token);
        Typedef {
            orig_type: decl.data_type,  
            decl_type: decl.decl_type,
            name: decl.name,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parser::RPCLParser;
    use crate::pest::Parser;

    #[test]
    fn parse_typedef_1() {
        // Parser
        let mut parsed = RPCLParser::parse(Rule::type_def, "typedef unsigned int uint_32_t;").unwrap();
        let typedef_generated = Typedef::from(parsed.next().unwrap());
        let typedef_coded = Typedef {
            name: "uint_32_t".to_string(),
            orig_type: DataType::Integer{ length: 32, signed: false },
            decl_type: DeclarationType::TypeNameDecl,
        };
        assert!(typedef_generated == typedef_coded, "Typedef parsing wrong");

        // Code-gen
        let rust_code: TokenStream = quote!{
            type uint_32_t = u32;
        };
        let generated_code: TokenStream = (&typedef_generated).into();
        assert!(generated_code.to_string() == rust_code.to_string(), "Typedef: Generated code wrong:\n{}\n{}", generated_code.to_string() , rust_code.to_string());
    }

    #[test]
    fn parse_typedef_2() {
        // Parser
        let mut parsed = RPCLParser::parse(Rule::type_def, "typedef char rpc_uuid<16>;").unwrap();
        let typedef_generated = Typedef::from(parsed.next().unwrap());
        let typedef_coded = Typedef {
            name: "rpc_uuid".to_string(),
            orig_type: DataType::TypeDef{ name: "char".to_string() },
            decl_type: DeclarationType::VarlenArray,
        };
        assert!(typedef_generated == typedef_coded, "Typedef parsing wrong");

        // Code-gen
        let rust_code: TokenStream = quote!{
            type rpc_uuid = std::vec::Vec<char>;
        };
        let generated_code: TokenStream = (&typedef_generated).into();
        assert!(generated_code.to_string() == rust_code.to_string(), "Typedef: Generated code wrong:\n{}\n{}", generated_code.to_string() , rust_code.to_string());
    }

    #[test]
    fn parse_typedef_3() {
        // Parser
        let mut parsed = RPCLParser::parse(Rule::type_def, "typedef opaque mem_data<>;").unwrap();
        let typedef_generated = Typedef::from(parsed.next().unwrap());
        let typedef_coded = Typedef {
            name: "mem_data".to_string(),
            orig_type: DataType::TypeDef{ name: "opaque".to_string() },
            decl_type: DeclarationType::VarlenArray,
        };
        assert!(typedef_generated == typedef_coded, "Typedef parsing wrong");

        // Code-gen
        let rust_code: TokenStream = quote!{
            type mem_data = std::vec::Vec<opaque>;
        };
        let generated_code: TokenStream = (&typedef_generated).into();
        assert!(generated_code.to_string() == rust_code.to_string(), "Typedef: Generated code wrong:\n{}\n{}", generated_code.to_string() , rust_code.to_string());
    }
}

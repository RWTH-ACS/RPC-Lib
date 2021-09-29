use quote::__private::TokenStream as QuoteTokenStream;
use quote::{format_ident, quote};

use super::parser::Rule;
use super::util::*;

use std::collections::HashSet;

pub struct TypeDef {
    kind: TypeKind,
    identifier: String,
    original_type: String,
    array_size: usize,
}

impl TypeDef {
    pub fn from_pest(typedef_rule: pest::iterators::Pair<'_, Rule>) -> TypeDef {
        let mut def = TypeDef {
            kind: TypeKind::SimpleIdent,
            identifier: String::new(),
            original_type: String::new(),
            array_size: 0,
        };
        let mut rule_iter = typedef_rule.into_inner();

        // Original Type
        let original_type = rule_iter.next().unwrap();
        def.original_type = original_type.as_str().to_string();

        // New Type
        let new_type = rule_iter.next().unwrap();
        match new_type.as_rule() {
            Rule::identifier => {
                def.identifier = new_type.as_str().to_string();
                def.kind = TypeKind::SimpleIdent;
            }
            Rule::array_fixed => {
                def.kind = TypeKind::FixedArray;
                let mut pair = new_type.into_inner();
                def.identifier = pair.next().unwrap().as_str().to_string();
                let integer = pair.next().unwrap().as_str();
                def.array_size = integer.parse::<usize>().unwrap();
            }
            Rule::array_varlen => {
                def.kind = TypeKind::VarlenArray;
                let mut pair = new_type.into_inner();
                def.identifier = pair.next().unwrap().as_str().to_string();
            }
            _ => {}
        }
        def
    }

    pub fn to_rust_code(&self, varlen_arrays: &mut HashSet<String>) -> QuoteTokenStream {
        // Name of new Type, that is being defined
        let new_ident = self.identifier.to_string();
        let formatted_ident = format_ident!("{}", new_ident);

        // Original Type: Converting from C to Rust
        let type_code = match self.kind {
            TypeKind::SimpleIdent => {
                let orig_type = format_ident!(
                    "{}",
                    convert_primitve_type(&self.original_type) // Example: unsigned int -> u32
                        .unwrap_or_else(|| &self.original_type) // Example: my_struct -> my_struct
                );
                quote!(#orig_type)
            }
            TypeKind::FixedArray => {
                let orig_type = format_ident!(
                    "{}",
                    convert_primitve_type(&self.original_type) // Example: unsigned int -> u32
                        .unwrap_or_else(|| &self.original_type) // Example: my_struct -> my_struct
                );
                let arr_size = self.array_size;
                quote!([#orig_type; #arr_size])
            }
            TypeKind::VarlenArray => {
                varlen_arrays.insert(self.original_type.clone());
                let orig_type = format_ident!(
                    "{}_var_arr",
                    convert_primitve_type(&self.original_type)
                        .unwrap_or_else(|| &self.original_type)
                );
                quote!(#orig_type)
            }
        };
        let code = quote! {
            type #formatted_ident = #type_code;
        };
        code
    }
}

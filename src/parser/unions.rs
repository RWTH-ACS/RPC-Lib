use quote::__private::TokenStream as QuoteTokenStream;
use quote::{format_ident, quote};

use super::parser::Rule;
use super::util::*;

use std::collections::HashSet;

pub struct UnionDef {
    pub identifier: String,
    pub optional_data_type: String,
    pub kind: TypeKind,
    fixed_array_size: usize, // If kind == FixedArray
}

impl UnionDef {
    pub fn from_pest(union_rule: pest::iterators::Pair<'_, Rule>) -> UnionDef {
        // Get every Token produced by union_rule
        let mut union_iter = union_rule.into_inner();
        let union_ident = union_iter.next().unwrap();
        let _union_switch_var = union_iter.next().unwrap();
        let _case_num = union_iter.next().unwrap();
        let optional_data = union_iter.next().unwrap();
        let _default_type = union_iter.next().unwrap();

        // Identifier
        let union_name = union_ident.as_str().to_string();

        // Optional Data
        let mut opt = optional_data.into_inner();
        let opt_type = opt.next().unwrap().into_inner().next().unwrap();
        let opt_name = opt.next().unwrap();

        let (type_name, kind, array_size) = match opt_name.as_rule() {
            Rule::identifier => (opt_type.as_str().to_string(), TypeKind::SimpleIdent, 0),
            Rule::array_fixed => {
                let mut array_fixed_iter = opt_name.into_inner();
                let _name = array_fixed_iter.next().unwrap();
                let size: usize = array_fixed_iter
                    .next()
                    .unwrap()
                    .as_str()
                    .parse::<usize>()
                    .unwrap();
                (opt_type.as_str().to_string(), TypeKind::FixedArray, size)
            }
            Rule::array_varlen => (opt_type.as_str().to_string(), TypeKind::VarlenArray, 0),
            _ => {
                panic!("Error in Union-Definition: {}", union_name)
            }
        };
        UnionDef {
            identifier: union_name,
            optional_data_type: type_name,
            kind: kind,
            fixed_array_size: array_size,
        }
    }

    /// Generates Rust-Code
    pub fn to_rust_code(&self, varlen_arrays: &mut HashSet<String>) -> QuoteTokenStream {
        let union_name = format_ident!("{}", self.identifier);
        let inner_union_ident = format_ident!("{}_u", self.identifier);

        // Type: Converting from C to Rust
        let inner_union_type = match self.kind {
            TypeKind::SimpleIdent => {
                let orig_type = format_ident!(
                    "{}",
                    convert_primitve_type(&self.optional_data_type) // Example: unsigned int -> u32
                        .unwrap_or_else(|| &self.optional_data_type) // Example: my_struct -> my_struct
                );
                quote!(#orig_type)
            }
            TypeKind::FixedArray => {
                let orig_type = format_ident!(
                    "{}",
                    convert_primitve_type(&self.optional_data_type) // Example: unsigned int -> u32
                        .unwrap_or_else(|| &self.optional_data_type) // Example: my_struct -> my_struct
                );
                let arr_size = self.fixed_array_size;
                quote!([#orig_type; #arr_size])
            }
            TypeKind::VarlenArray => {
                if self.optional_data_type == "string" {
                    quote!(*mut c_char)
                } else {
                    varlen_arrays.insert(self.optional_data_type.clone());
                    let orig_type = format_ident!(
                        "{}_var_arr",
                        convert_primitve_type(&self.optional_data_type)
                            .unwrap_or_else(|| &self.optional_data_type)
                    );
                    quote!(#orig_type)
                }
            }
        };
        let code = quote! {
            // Contained type may be defined somewhere above (If Array-Type)
            #[repr(C)]
            struct #union_name {
                err: i32,
                #inner_union_ident: #inner_union_type
            }
        };
        code
    }
}

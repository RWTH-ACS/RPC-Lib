// Copyright 2022 Philipp Fensch
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::collections::HashSet;

use crate::parser::declaration::DeclarationType;
use crate::parser::Rule;

use proc_macro2::TokenStream;
use quote::quote;

use super::constant::ConstantDeclaration;
use super::enumdef::Enumdef;
use super::structdef::Structdef;
use super::typedef::Typedef;
use super::uniondef::Uniondef;

pub enum ResolvedType<'a> {
    Struct(&'a Structdef),
    Union(&'a Uniondef),
    Enum(&'a Enumdef),
    // TODO: Consts as well?
}

#[derive(Debug)]
pub struct Specification {
    typedefs: std::vec::Vec<Typedef>,
    enums: std::vec::Vec<Enumdef>,
    structs: std::vec::Vec<Structdef>,
    unions: std::vec::Vec<Uniondef>,
    constants: std::vec::Vec<ConstantDeclaration>,
    pub union_typedefs_with_vararray: HashSet<String>,
}
impl Specification {
    /// Creates a copy of all datatypes that are of type [`DeclarationType::VarlenArray`]. These
    /// can be used for zero-copy operation variants of functions.
    pub fn update_contains_vararray(&mut self) {
        // TODO: Nested typedefs are not handled
        let mut vararray_typedefs: HashSet<String> = HashSet::new();
        let sliced_typedefs: Vec<Typedef> = self
            .typedefs
            .iter()
            .filter(|td| td.decl_type == DeclarationType::VarlenArray)
            .map(|td| {
                vararray_typedefs.insert(td.name.clone());
                let mut sliced_td = (*td).clone();
                sliced_td.decl_type = DeclarationType::ArraySlice;
                sliced_td.name.push_str("_sliced");
                sliced_td.needs_lifetime = true;
                sliced_td
            })
            .collect();
        self.typedefs.extend_from_slice(sliced_typedefs.as_slice());

        self.structs
            .iter_mut()
            .for_each(|s| s.update_contains_vararray(&vararray_typedefs));

        let sliced_structs: Vec<Structdef> = self
            .structs
            .iter()
            .filter(|s| s.contains_vararray)
            .map(|s| s.sliced_copy(&vararray_typedefs))
            .collect();
        self.structs.extend(sliced_structs);

        self.unions
            .iter_mut()
            .for_each(|u| u.update_contains_vararray(&vararray_typedefs));
        let sliced_unions: Vec<Uniondef> = self
            .unions
            .iter()
            .filter(|u| u.contains_vararray)
            .map(|u| {
                self.union_typedefs_with_vararray.insert(u.name.clone());
                u.sliced_copy(&vararray_typedefs)
            })
            .collect();
        self.unions.extend(sliced_unions);
    }

    pub fn get_type_specification<'a>(&'a self, name: &str) -> Option<ResolvedType<'a>> {
        for s in &self.structs {
            if &s.name == name {
                return Some(ResolvedType::Struct(s));
            }
        }
        for u in &self.unions {
            if &u.name == name {
                return Some(ResolvedType::Union(u));
            }
        }
        for e in &self.enums {
            if &e.name == name {
                return Some(ResolvedType::Enum(e));
            }
        }
        None
    }
}

impl From<&Specification> for TokenStream {
    fn from(spec: &Specification) -> TokenStream {
        let mut code = quote!();
        for typedef in &spec.typedefs {
            let def: TokenStream = typedef.into();
            code = quote!( #code #def );
        }
        for enumdef in &spec.enums {
            let def: TokenStream = enumdef.into();
            code = quote!( #code #def );
        }
        for structdef in &spec.structs {
            let def: TokenStream = structdef.into();
            code = quote!( #code #def );
        }
        for uniondef in &spec.unions {
            let def: TokenStream = uniondef.into();
            code = quote!( #code #def );
        }
        for constant in &spec.constants {
            let def: TokenStream = constant.into();
            code = quote!( #code #def );
        }
        quote!(type opaque = u8; #code)
    }
}

impl From<pest::iterators::Pair<'_, Rule>> for Specification {
    fn from(specification: pest::iterators::Pair<'_, Rule>) -> Specification {
        let mut spec = Specification {
            typedefs: std::vec::Vec::new(),
            enums: std::vec::Vec::new(),
            structs: std::vec::Vec::new(),
            unions: std::vec::Vec::new(),
            constants: std::vec::Vec::new(),
            union_typedefs_with_vararray: HashSet::new(),
        };
        for definition in specification.into_inner() {
            match definition.as_rule() {
                Rule::type_def => {
                    spec.typedefs.push(Typedef::from(definition));
                }
                Rule::enum_def => {
                    spec.enums.push(Enumdef::from(definition));
                }
                Rule::struct_def => {
                    spec.structs.push(Structdef::from(definition));
                }
                Rule::union_def => {
                    spec.unions.push(Uniondef::from(definition));
                }
                Rule::constant_def => {
                    spec.constants.push(ConstantDeclaration::from(definition));
                }
                _ => eprintln!("Unknown Definition"),
            }
        }
        spec
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::RPCLParser;
    use pest::Parser;

    #[test]
    fn parse_specification_1() {
        let s = "const CON = 1;
        struct X {
            int x;
            int y;
        };
        const CON2 = 2;
        enum X {
            X1 = 1,
            X2 = 2
        };
        ";
        let mut parsed = RPCLParser::parse(Rule::specification, s).unwrap();
        let spec = Specification::from(parsed.next().unwrap());

        assert!(
            spec.constants.len() == 2,
            "Number of parsed constants wrong"
        );
        assert!(spec.structs.len() == 1, "Number of parsed structs wrong");
        assert!(spec.enums.len() == 1, "Number of parsed enums wrong");
    }

    #[test]
    fn parse_specification_2() {
        let s = "union MyUnion switch(int err) {
            case 0:
                int result;
            default:
                void;
        };
        union MyUnion2 switch(int err) {
            case 0:
                int result;
            case 2:
                float result;
            default:
                void;
        };
        typedef unsigned int u_int_32;
        typedef Type1 Type2;
        ";
        let mut parsed = RPCLParser::parse(Rule::specification, s).unwrap();
        let spec = Specification::from(parsed.next().unwrap());

        assert!(spec.unions.len() == 2, "Number of parsed unions wrong");
        assert!(spec.typedefs.len() == 2, "Number of parsed typedefs wrong");
    }
}

// Copyright 2022 Philipp Fensch
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::parser::Rule;

use proc_macro2::TokenStream;
use quote::quote;

use super::constant::ConstantDeclaration;
use super::enumdef::Enumdef;
use super::structdef::Structdef;
use super::typedef::Typedef;
use super::uniondef::Uniondef;

pub struct Specification {
    typedefs: std::vec::Vec<Typedef>,
    enums: std::vec::Vec<Enumdef>,
    structs: std::vec::Vec<Structdef>,
    unions: std::vec::Vec<Uniondef>,
    constants: std::vec::Vec<ConstantDeclaration>,
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
                _ => println!("Unknown Definition"),
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

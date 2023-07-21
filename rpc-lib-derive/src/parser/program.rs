// Copyright 2022 Philipp Fensch
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::parser::xdr_spec::ResolvedType;
use crate::parser::Rule;
use proc_macro2::TokenStream;
use quote::quote;

use super::datatype::DataType;
use super::procedure::{Procedure, RawCallType};
use super::uniondef::DiscriminantType;
use super::xdr_spec::Specification;

#[derive(Debug)]
pub struct Program {
    pub program_number: u32,
    pub versions: std::vec::Vec<Version>,
}

impl From<&Program> for TokenStream {
    fn from(program: &Program) -> TokenStream {
        assert!(
            program.versions.len() == 1,
            "Multiple Versions not supported!"
        );
        let mut version_code = quote!();
        for version in &program.versions {
            let code: TokenStream = version.into();
            version_code = quote!( #version_code #code )
        }
        version_code
    }
}

impl From<pest::iterators::Pair<'_, Rule>> for Program {
    fn from(program_def: pest::iterators::Pair<'_, Rule>) -> Program {
        let mut prog = Program {
            program_number: 0,
            versions: std::vec::Vec::new(),
        };
        let iter = program_def.into_inner();
        for x in iter {
            match x.as_rule() {
                Rule::version_def => {
                    // Inner Version Rule
                    prog.versions.push(Version::from(x));
                }
                Rule::identifier => {
                    // Name of program
                }
                Rule::constant => {
                    // Number of program
                    prog.program_number = x.as_str().parse::<u32>().unwrap();
                }
                _ => panic!("Invalid Syntax in Function"),
            }
        }
        prog
    }
}

#[derive(Debug)]
pub struct Version {
    pub version_number: u32,
    procedures: std::vec::Vec<Procedure>,
}
impl Version {
    pub fn create_sliced_variants(&mut self, spec: &Specification) {
        let mut sliced_procedures = Vec::new();
        for p in &self.procedures {
            match &p.return_type {
                DataType::TypeDef { name } => match spec.get_type_specification(name) {
                    Some(ResolvedType::Union(u)) => {
                        if u.contains_vararray && u.union_body.discriminant == DiscriminantType::Int
                        {
                            let mut sliced_proc = p.clone();
                            sliced_proc.name.push_str("_raw");
                            sliced_proc.slice_call_target_type = Some(RawCallType::UnionI32);
                            sliced_proc.return_type = DataType::Void;
                            sliced_procedures.push(sliced_proc);
                        }
                    }
                    // sliced variants for structs or enums are not yet supported
                    _ => {}
                },
                // sliced variants for Anonymous structs or unions are not yet supported
                _ => {}
            }
        }
        self.procedures.extend(sliced_procedures);
    }
}

impl From<&Version> for TokenStream {
    fn from(version: &Version) -> TokenStream {
        let mut code = quote!();
        for proc in &version.procedures {
            let proc_code: TokenStream = proc.into();
            code = quote!( #code #proc_code );
        }
        code
    }
}

impl From<pest::iterators::Pair<'_, Rule>> for Version {
    fn from(version_def: pest::iterators::Pair<'_, Rule>) -> Version {
        let mut vers = Version {
            version_number: 0,
            procedures: std::vec::Vec::new(),
        };
        let iter = version_def.into_inner();
        for x in iter {
            match x.as_rule() {
                Rule::procedure_def => {
                    // Inner Version Rule
                    vers.procedures.push(Procedure::from(x));
                }
                Rule::identifier => {
                    // Name of program
                }
                Rule::constant => {
                    // Number of program
                    vers.version_number = x.as_str().parse::<u32>().unwrap();
                }
                _ => panic!("Invalid Syntax in Function"),
            }
        }
        vers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::RPCLParser;
    use pest::Parser;

    // Tests program_def
    #[test]
    fn parse_program_def() {
        let s = "program PROG {
            version VERS {
                void FUNC(void) = 1;
            } = 1;
        } = 1;";
        let mut parsed = RPCLParser::parse(Rule::program_def, s).unwrap();
        let prog = Program::from(parsed.next().unwrap());

        assert!(prog.program_number == 1, "Program Number wrong");
        assert!(prog.versions.len() == 1, "Number of parsed Versions wrong!");
    }

    #[test]
    fn parse_program_multiple_versions() {
        let s = "program PROG {
            version VERS {
                void FUNC(void) = 1;
            } = 1;
            version VERS {
                void FUNC(int) = 1;
            } = 2;
        } = 10;";
        let mut parsed = RPCLParser::parse(Rule::program_def, s).unwrap();
        let prog = Program::from(parsed.next().unwrap());

        assert!(prog.program_number == 10, "Program Number wrong");
        assert!(prog.versions.len() == 2, "Number of parsed Versions wrong!");
    }

    // Tests version_def
    #[test]
    fn parse_version_def() {
        let s = "version VERS {
            void FUNC(void) = 1;
            void FUNC_3(int) = 2;
            void FUNC_7(void) = 3;
         } = 5;";
        let mut parsed = RPCLParser::parse(Rule::version_def, s).unwrap();
        let vers = Version::from(parsed.next().unwrap());

        assert!(vers.version_number == 5, "Version Number wrong");
        assert!(
            vers.procedures.len() == 3,
            "Number of parsed procedures wrong!"
        );
    }
}

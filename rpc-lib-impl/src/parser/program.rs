use crate::parser::parser::Rule;

struct Program {
    program_number: u32,
    versions: std::vec::Vec<Version>,
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

struct Version {
    version_number: u32,
    procedures: std::vec::Vec<u32>,
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
                    vers.procedures.push(0);
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
    use crate::parser::parser::RPCLParser;
    use crate::pest::Parser;

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

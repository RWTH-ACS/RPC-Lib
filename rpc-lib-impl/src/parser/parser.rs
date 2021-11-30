use pest::Parser;
use proc_macro::TokenStream;
use quote::quote;

#[derive(Parser)]
#[grammar = "rpcl.pest"]
pub struct RPCLParser;

pub fn parse(x_file: &String, _struct_name: &String) -> (TokenStream, u32, u32) {
    let parsed = RPCLParser::parse(Rule::file, x_file).expect("Syntax Error in .x-File");

    let program_number = 0;
    let version_number = 0;

    let code = quote!();

    (code.into(), program_number, version_number)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_rule() {
        let x_file = "program PROG {
            version VERS {
                int FUNC(void) = 1;
            } = 1;
        } = 10;";
        let _parsed = RPCLParser::parse(Rule::file, x_file).expect("Syntax Error in .x-File");

        let x_file = "struct X {
            int x;
            int y;
        };
        
        program PROG {
            version VERS {
                int FUNC(void) = 1;
            } = 1;
        } = 10;";
        let _parsed = RPCLParser::parse(Rule::file, x_file).expect("Syntax Error in .x-File");
    }
}

use crate::parser::parser::Rule;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use super::constant::Value;
use super::datatype::DataType;

pub struct Procedure {
    name: String,
    return_type: DataType,
    args: std::vec::Vec<DataType>,
    num: Value,
}

impl From<&Procedure> for TokenStream {
    fn from(proc: &Procedure) -> TokenStream {
        let proc_name = format_ident!("{}", proc.name);
        let mut args = quote!();
        let mut serialization_code = quote!( let mut send_data = std::vec::Vec::new(); );
        let mut i: u32 = 0;
        for arg in &proc.args {
            let arg_name = format_ident!("x{}", i);
            let arg_type: TokenStream = arg.into();
            args = quote!( #args #arg_name: #arg_type,);
            serialization_code = quote!( #serialization_code send_data.extend(#arg_name.serialize()); );
            i = i + 1;
        }
        let proc_num: TokenStream = (&proc.num).into();
        if proc.return_type == DataType::Void {
            quote!{ fn #proc_name (&self, #args) { }}
        }
        else {
            let return_type: TokenStream = (&proc.return_type).into();
            quote!{ fn #proc_name (&mut self, #args) -> #return_type {
                // Parameter-Seralization
                #serialization_code

                // Call
                let recv = rpc_lib::rpc_call(&mut self.client, #proc_num as u32, &send_data);

                // Parse ReplyHeader
                let mut parse_index = 0;
                <#return_type>::deserialize(&recv, &mut parse_index)
            }}
        }
    }
}

impl From<pest::iterators::Pair<'_, Rule>> for Procedure {
    fn from(procedure_def: pest::iterators::Pair<'_, Rule>) -> Procedure {
        let mut iter = procedure_def.into_inner();
        let proc_return = iter.next().unwrap();
        let proc_name = iter.next().unwrap();
        let proc_args = iter.next().unwrap();
        let proc_num = iter.next().unwrap();

        let mut arg_vec = std::vec::Vec::new();
        for arg in proc_args.into_inner() {
            if arg.as_rule() == Rule::type_specifier {
                arg_vec.push(DataType::from(arg));
            }
        }

        Procedure {
            name: proc_name.as_str().to_string(),
            return_type: DataType::from(proc_return.into_inner().next().unwrap()),
            args: arg_vec,
            num: Value::from(proc_num),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parser::RPCLParser;
    use crate::pest::Parser;

    #[test]
    fn parse_procedure_1() {
        let mut parsed = RPCLParser::parse(
            Rule::procedure_def,
            "float PROC_NAME(int, float) = 1;"
        ).unwrap();

        let proc = Procedure::from(parsed.next().unwrap());

        assert!(proc.name == "PROC_NAME".to_string(), "Procedure name wrong");
        assert!(proc.return_type == DataType::Float { length: 32 }, "Procedure return type wrong");
        assert!(proc.args == vec![
            DataType::Integer { length: 32, signed: true },
            DataType::Float { length: 32 },
        ], "Arguments wrong");
        assert!(proc.num == Value::Numeric { val: 1 }, "Procedure Number wrong");
    }

    #[test]
    fn parse_procedure_2() {
        let mut parsed = RPCLParser::parse(
            Rule::procedure_def,
            "void PROC_NAME(void) = 0x24;"
        ).unwrap();

        let proc = Procedure::from(parsed.next().unwrap());

        assert!(proc.name == "PROC_NAME".to_string(), "Procedure name wrong");
        assert!(proc.return_type == DataType::Void, "Procedure return type wrong");
        assert!(proc.args == vec![], "Arguments wrong");
        assert!(proc.num == Value::Numeric { val: 0x24 }, "Procedure Number wrong");
    }
}

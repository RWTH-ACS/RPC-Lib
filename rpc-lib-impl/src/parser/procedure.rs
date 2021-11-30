use crate::parser::parser::Rule;

use super::constant::Value;
use super::datatype::DataType;

struct Procedure {
    name: String,
    return_type: DataType,
    args: std::vec::Vec<DataType>,
    num: Value,
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

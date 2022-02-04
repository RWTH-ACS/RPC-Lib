use crate::parser::parser::Rule;

use proc_macro2::TokenStream;
use quote::quote;

use super::datatype::DataType;
use super::declaration::Declaration;

pub struct Typedef {
    name: String,
    orig_type: DataType,
}

impl From<&Typedef> for TokenStream {
    fn from(type_def: &Typedef) -> TokenStream {
        let orig_type: TokenStream = (&type_def.orig_type).into();
        let name = quote::format_ident!("{}", type_def.name);
        quote!(type #name = #orig_type;)
    }
}

impl From<pest::iterators::Pair<'_, Rule>> for Typedef {
    fn from(type_def: pest::iterators::Pair<'_, Rule>) -> Typedef {
        let mut typedef = Typedef {
            name: "".to_string(),
            orig_type: DataType::Void,
        };
        for token in type_def.into_inner() {
            match token.as_rule() {
                Rule::declaration => {
                    let decl = Declaration::from(token);
                    typedef.orig_type = decl.data_type;
                    typedef.name = decl.name;
                }
                _ => println!("Syntax Error"),
            }
        }
        typedef
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parser::RPCLParser;
    use crate::pest::Parser;

    #[test]
    fn parse_typedef_1() {
        let mut parsed =
            RPCLParser::parse(Rule::type_def, "typedef unsigned int uint_32_t;").unwrap();
        let typedef = Typedef::from(parsed.next().unwrap());

        assert!(typedef.name == "uint_32_t", "Typedef defined type wrong");
        //assert!(typedef.orig_type == DataType::NamedType{ name: "unsigned int".to_string() }, "Typedef original type wrong");
    }

    #[test]
    fn parse_typedef_2() {
        let mut parsed = RPCLParser::parse(Rule::type_def, "typedef char rpc_uuid<16>;").unwrap();
        let typedef = Typedef::from(parsed.next().unwrap());

        assert!(typedef.name == "rpc_uuid", "Typedef defined type wrong");
        //assert!(typedef.orig_type == "char<16>", "Typedef original type wrong");
    }

    #[test]
    fn parse_typedef_3() {
        let mut parsed = RPCLParser::parse(Rule::type_def, "typedef opaque mem_data<>;").unwrap();
        let typedef = Typedef::from(parsed.next().unwrap());

        assert!(typedef.name == "mem_data", "Typedef defined type wrong");
        //assert!(typedef.orig_type == "opaque<>", "Typedef original type wrong");
    }
}

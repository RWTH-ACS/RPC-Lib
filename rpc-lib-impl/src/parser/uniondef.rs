use crate::parser::parser::Rule;

use super::constant::Value;
use super::datatype::DataType;
use super::declaration::{Declaration, DeclarationType};

#[derive(PartialEq)]
pub struct Uniondef {
    name: String,
    union_body: Union,
}

#[derive(PartialEq)]
pub struct Union {
    cases: std::vec::Vec<(Value, Declaration)>,
    default: std::boxed::Box<Declaration>,
}

pub fn parse_union_type_spec(union_type_spec: pest::iterators::Pair<'_, Rule>) -> Union {
    Union::from(union_type_spec.into_inner().next().unwrap())
}

impl From<pest::iterators::Pair<'_, Rule>> for Uniondef {
    fn from(union_def: pest::iterators::Pair<'_, Rule>) -> Uniondef {
        let mut iter = union_def.into_inner();
        let name = iter.next().unwrap();
        let union_body = iter.next().unwrap();
        Uniondef {
            name: name.as_str().to_string(),
            union_body: Union::from(union_body),
        }
    }
}

fn parse_case_spec(case_spec: pest::iterators::Pair<'_, Rule>) -> (Value, Declaration) {
    let mut iter = case_spec.into_inner();
    let val = iter.next().unwrap();
    let decl = iter.next().unwrap();
    (Value::from(val), Declaration::from(decl))
}

impl From<pest::iterators::Pair<'_, Rule>> for Union {
    fn from(union_body: pest::iterators::Pair<'_, Rule>) -> Union {
        let mut union_def = Union {
            cases: std::vec::Vec::new(),
            default: std::boxed::Box::new(Declaration {
                decl_type: DeclarationType::VoidDecl,
                data_type: DataType::Void,
                name: "".into(),
            }),
        };
        for token in union_body.into_inner() {
            match token.as_rule() {
                Rule::discriminant_decl => {}
                Rule::case_spec => {
                    union_def.cases.push(parse_case_spec(token));
                }
                Rule::declaration => {
                    union_def.default = std::boxed::Box::new(Declaration::from(token));
                }
                _ => panic!("Syntax Error"),
            }
        }
        union_def
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parser::RPCLParser;
    use crate::pest::Parser;

    #[test]
    fn parse_union_1() {
        let mut parsed = RPCLParser::parse(
            Rule::union_body,
            "switch(int x) {case X: int x; case Y2: unsigned hyper c; default: void; }",
        )
        .unwrap();
        let union_body = Union::from(parsed.next().unwrap());

        let un = Union {
            cases: vec![
                (
                    Value::Named { name: "X".into() },
                    Declaration {
                        decl_type: DeclarationType::TypeNameDecl,
                        data_type: DataType::Integer {
                            length: 32,
                            signed: true,
                        },
                        name: "x".into(),
                    },
                ),
                (
                    Value::Named { name: "Y2".into() },
                    Declaration {
                        decl_type: DeclarationType::TypeNameDecl,
                        data_type: DataType::Integer {
                            length: 64,
                            signed: false,
                        },
                        name: "c".into(),
                    },
                ),
            ],
            default: std::boxed::Box::new(Declaration {
                decl_type: DeclarationType::VoidDecl,
                data_type: DataType::Void,
                name: "".into(),
            }),
        };
        assert!(un == union_body, "Union Body wrong");
    }

    #[test]
    fn parse_union_2() {
        let mut parsed = RPCLParser::parse(
            Rule::union_def,
            "union MyUnion switch(unsigned int err) {case 1: int y; default: void; };",
        )
        .unwrap();
        let parsed_def = Uniondef::from(parsed.next().unwrap());

        let un = Union {
            cases: vec![(
                Value::Numeric { val: 1 },
                Declaration {
                    decl_type: DeclarationType::TypeNameDecl,
                    data_type: DataType::Integer {
                        length: 32,
                        signed: true,
                    },
                    name: "y".into(),
                },
            )],
            default: std::boxed::Box::new(Declaration {
                decl_type: DeclarationType::VoidDecl,
                data_type: DataType::Void,
                name: "".into(),
            }),
        };
        assert!(
            Uniondef {
                name: "MyUnion".into(),
                union_body: un
            } == parsed_def,
            "Union Def wrong"
        );
    }

    #[test]
    fn parse_union_3() {
        let mut parsed = RPCLParser::parse(
            Rule::union_type_spec,
            "union switch(unsigned int err) {case 1: int y; default: void; }",
        )
        .unwrap();
        let union_body = parse_union_type_spec(parsed.next().unwrap());

        let un = Union {
            cases: vec![(
                Value::Numeric { val: 1 },
                Declaration {
                    decl_type: DeclarationType::TypeNameDecl,
                    data_type: DataType::Integer {
                        length: 32,
                        signed: true,
                    },
                    name: "y".into(),
                },
            )],
            default: std::boxed::Box::new(Declaration {
                decl_type: DeclarationType::VoidDecl,
                data_type: DataType::Void,
                name: "".into(),
            }),
        };
        assert!(un == union_body, "Union Spec wrong");
    }

    #[test]
    fn parse_union_def() {
        let mut parsed = RPCLParser::parse(
            Rule::union_def,
            "union MyUnion2 switch(int err) {
                case 0:
                    int result;
                case 2:
                    float result;
                default:
                    void;
            };",
        )
        .unwrap();
        let parsed_def = Uniondef::from(parsed.next().unwrap());

        let un = Union {
            cases: vec![
                (
                    Value::Numeric { val: 0 },
                    Declaration {
                        decl_type: DeclarationType::TypeNameDecl,
                        data_type: DataType::Integer {
                            length: 32,
                            signed: true,
                        },
                        name: "result".into(),
                    },
                ),
                (
                    Value::Numeric { val: 2 },
                    Declaration {
                        decl_type: DeclarationType::TypeNameDecl,
                        data_type: DataType::Float { length: 32 },
                        name: "result".into(),
                    },
                ),
            ],
            default: std::boxed::Box::new(Declaration {
                decl_type: DeclarationType::VoidDecl,
                data_type: DataType::Void,
                name: "".into(),
            }),
        };
        assert!(un == parsed_def.union_body, "Union Def wrong");
    }
}

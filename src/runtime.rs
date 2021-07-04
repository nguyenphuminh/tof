use crate::lexer::*;
use std::collections::{BTreeMap, VecDeque};

#[derive(Debug, Clone)]
pub enum Variable {
    Lamda { args: Vec<String>, value: Vec<LEX> },
    Rusty(fn(args: Vec<Variable>) -> Variable),
    Int(i32),
    Str(String),
    Void,
}

#[derive(Debug, Clone)]
pub struct Vars {
    vars: VecDeque<BTreeMap<String, Variable>>,
}

impl Vars {
    fn new() -> Vars {
        let mut list = VecDeque::new();
        list.push_back(BTreeMap::new());
        Vars { vars: list }
    }

    fn push(&mut self) {
        self.vars.push_back(BTreeMap::new());
    }

    fn pop(&mut self) {
        self.vars.pop_back();
    }

    fn insert(&mut self, id: String, data: Variable) {
        let m = self.vars.back_mut().unwrap();
        m.insert(id, data);
    }

    fn get(&mut self, id: &str) -> Option<&Variable> {
        for m in self.vars.iter().rev() {
            if let Some(val) = m.get(id) {
                return Some(val);
            }
        }
        None
    }
}

pub fn start(lex: Vec<LEX>) {
    let mut data: Vars = Vars::new();
    prelude(&mut data);
    eval(lex, &mut data);
}

pub fn prelude(data: &mut Vars) {
    data.insert(
        "stdio".to_string(),
        Variable::Rusty(|args| {
            println!("{:?}", args[0]);
            Variable::Void
        }),
    );
}

pub fn eval(lex: Vec<LEX>, data: &mut Vars) -> Variable {
    let t = Variable::Void;
    for line in lex {
        match line {
            LEX::DEF(def) => {
                eval_def(def, data);
            }
            LEX::EXPR(expr) => {
                eval_expr(expr, data);
            }
            LEX::RETURN(expr) => {
                eval_expr(expr, data);
            }
        }
    }
    t
}

pub fn eval_def(def: Def, data: &mut Vars) {
    let val = eval_expr(def.value, data);
    data.insert(def.name, val);
}

pub fn eval_expr(expr: Expr, mut data: &mut Vars) -> Variable {
    let mut v = Variable::Void;

    match *expr.node {
        Node::Int(int) => v = Variable::Int(int),
        Node::Str(string) => v = Variable::Str(string),
        Node::Lamda { args, value } => v = Variable::Lamda { args, value },
        Node::FCCALL { ref args, ref name } => {
            let mut fc = data.clone();
            let fc = fc.get(&name).unwrap();
            let args_t_s = args
                .iter()
                .map(|node| {
                    eval_expr(
                        Expr {
                            line_number: expr.line_number.clone(),
                            node: node.clone(),
                        },
                        data,
                    )
                })
                .collect();
            match &fc {
                Variable::Rusty(fnc) => {
                    fnc(args_t_s);
                }
                Variable::Lamda { args, value } => {
                    for (pos, e) in args.iter().enumerate() {
                        data.insert(e.to_string(), args_t_s[pos].clone());
                    }

                    eval(value.clone(), data);
                }
                _ => {
                    panic!("not callable");
                }
            }
        }
        Node::CALL(name) => {
            v = data.get(&name).unwrap().clone();
        }
        Node::VOID => v = Variable::Void,
        Node::OP { joint, lhs, rhs } => {
            let lhs = eval_expr(
                Expr {
                    line_number: expr.line_number,
                    node: lhs,
                },
                &mut data,
            );
            let rhs = eval_expr(
                Expr {
                    line_number: expr.line_number,
                    node: rhs,
                },
                &mut data,
            );
            match joint {
                JOINT::ADD => match lhs {
                    Variable::Int(int) => match rhs {
                        Variable::Int(int2) => {
                            v = Variable::Int(int + int2);
                        }
                        _ => panic!("you can only add numbers and string"),
                    },
                    Variable::Str(string) => match rhs {
                        Variable::Str(string2) => {
                            v = Variable::Str(string + &string2);
                        }
                        _ => panic!("you can only add numbers and string"),
                    },
                    _ => panic!("you can only add numsbers and string"),
                },
                JOINT::SUB => {}
                JOINT::MULT => {}
                JOINT::DIV => {}
            }
        }
    }
    v
}

use std::collections::HashMap;

use crate::mir::*;

#[derive(Clone, Copy, Debug)]
pub enum Value {
    Int(i64),
    Bool(bool),
    Unit,
}

pub struct Interpreter<'a> {
    values: Vec<HashMap<String, Value>>,
    funcs: HashMap<String, Func<'a>>,
}

impl<'a> Interpreter<'a> {
    pub fn new(funcs: Vec<Func<'a>>) -> Self {
        let mut funcs_hash = HashMap::new();
        for f in funcs {
            funcs_hash.insert(f.name.data.to_owned(), f);
        }
        Self {
            values: Default::default(),
            funcs: funcs_hash
        }
    }
    pub fn run(&mut self) -> Value {
        let main = self.funcs.get("main").unwrap().clone();
        self.translate_block(&main.block)
    }
    pub fn translate_block(&mut self, stmts: &[Stmt]) -> Value {
        let mut ret = Value::Unit;
        for s in stmts {
            ret = self.translate_stmt(s);
        }
        ret
    }
    pub fn translate_stmt(&mut self, stmt: &'_ Stmt<'_>) -> Value {
        match stmt {
            Stmt::Expr(e) => self.translate_expr(e),
            Stmt::Let(name, e) => {
                let value = self.translate_expr(e);
                self.values.last_mut().unwrap().insert(name.data.to_string(), value);
                Value::Unit
            },
            Stmt::Assign(name, e) => {
                *self.values.last_mut().unwrap().get_mut(name.data).unwrap() = self.translate_expr(e);
                Value::Unit
            },
            Stmt::Return(e) => {
                //TODO:
                self.translate_expr(e)
            },
            Stmt::While(cond, block) => {
                let mut ret = Value::Unit;
                while let Value::Bool(true) = self.translate_expr(cond) {
                    ret = self.translate_block(block);
                }
                ret
            },
        }
    }
    pub fn translate_expr(&mut self, expr: &'_ Expression<'_>) -> Value {
        match expr {
            Expression::Int(x) => Value::Int(x.data),
            Expression::Bool(x) => Value::Bool(*x),
            Expression::Name(name) => *self.values.last().unwrap().get(name.data).unwrap(),
            Expression::Add(l, r) => self.int_func(l, r, |a, b| a + b),
            Expression::Sub(l, r) => self.int_func(l, r, |a, b| a - b),
            Expression::Mul(l, r) => self.int_func(l, r, |a, b| a * b),
            Expression::Div(l, r) => self.int_func(l, r, |a, b| a / b),
            Expression::Eq(l, r) => {
                let l = self.translate_expr(l);
                let r = self.translate_expr(r);
                match (l, r) {
                    (Value::Int(l), Value::Int(r)) => Value::Bool(l == r),
                    (Value::Bool(l), Value::Bool(r)) => Value::Bool(l == r),
                    _ => panic!("expect same type"),
                }
            },
            Expression::Neq(l, r) => {
                let l = self.translate_expr(l);
                let r = self.translate_expr(r);
                match (l, r) {
                    (Value::Int(l), Value::Int(r)) => Value::Bool(l != r),
                    (Value::Bool(l), Value::Bool(r)) => Value::Bool(l != r),
                    _ => panic!("expect same type"),
                }
            },
            Expression::Call(name, p) => {
                if name.data == "print" {
                    let value = self.translate_expr(p.get(0).unwrap());
                    println!("{value:?}");
                    Value::Unit
                }else {
                    let func = self.funcs.get(name.data).unwrap().clone();
                    let mut next_value = HashMap::new();
                    for ((name, _), e) in func.params.iter().zip(p.iter()) {
                        let value = self.translate_expr(e);
                        next_value.insert(name.data.to_string(), value);
                    }
                    self.values.push(next_value);
                    let ret = self.translate_block(&func.block);
                    self.values.pop();
                    ret
                }
            },
            Expression::If(cond, then_body, else_body) => {
                if let Value::Bool(true) = self.translate_expr(cond) {
                    self.translate_block(then_body)
                }else if let Some(e) = else_body {
                    self.translate_block(e)
                }else {
                    Value::Unit
                }
            },
        }
    }

    fn int_func<F>(&mut self, l: &'_ Expression<'_>, r: &'_ Expression<'_>, f: F) -> Value
    where
        F: Fn(i64, i64) -> i64
    {
        let l = self.translate_expr(l);
        let r = self.translate_expr(r);
        match (l, r) {
            (Value::Int(l), Value::Int(r)) => Value::Int(f(l, r)),
            _ => panic!("expect Int"),
        }
    }
}

use std::collections::HashMap;

use crate::mir::*;

#[derive(Clone, Copy, Debug)]
pub enum Value {
    Int(i64),
    Bool(bool),
    HeapId(usize),
    Unit,
}

#[derive(Clone, Debug)]
pub enum HeapValue {
    Vec(Vec<Value>),
    String(String),
    Class(Class),
}

#[derive(Clone, Debug)]
pub struct Class {
    values: Vec<Value>,

}

pub struct Interpreter<'a> {
    stack: Vec<Value>,
    heap: HashMap<usize, HeapValue>,
    funcs: HashMap<String, Func<'a>>,
    func_stack_offset: usize,
}

impl<'a> Interpreter<'a> {
    pub fn new(funcs: Vec<Func<'a>>) -> Self {
        let mut funcs_hash = HashMap::new();
        for f in funcs {
            funcs_hash.insert(f.name.data.to_owned(), f);
        }
        Self {
            stack: vec![],
            heap: Default::default(),
            funcs: funcs_hash,
            func_stack_offset: 0,
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
            Stmt::Let(_, e) => {
                let value = self.translate_expr(e);
                self.stack.push(value);
                Value::Unit
            },
            Stmt::Assign(name, e) => {
                *self.stack.get_mut(self.func_stack_offset + name.data).unwrap() = self.translate_expr(e);
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
            Expression::String(idx, data) => {
                self.heap.insert(*idx, HeapValue::String(data.data.to_owned()));
                Value::HeapId(*idx)
            }
            Expression::Bool(x) => Value::Bool(*x),
            Expression::Name(name) => *self.stack.get(self.func_stack_offset + name.data).unwrap(),
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
                    match value {
                        Value::HeapId(idx) => {
                            match self.heap.get(&idx).unwrap() {
                                HeapValue::Vec(x) => {println!("{x:?}")},
                                HeapValue::String(s) => {println!("{s}");},
                                HeapValue::Class(_) => todo!(),
                            }
                        },
                        _ => {println!("{value:?}");},
                    }
                    Value::Unit
                }else if name.data == "Array" {
                    let value = p.iter()
                        .map(|x| self.translate_expr(x))
                        .collect::<Vec<_>>();
                    let idx = self.heap.len();
                    self.heap.insert(idx, HeapValue::Vec(value));
                    Value::HeapId(idx)
                }else {
                    let func = self.funcs.get(name.data).unwrap().clone();
                    let old_offset = self.func_stack_offset;
                    let next_offset = self.stack.len();
                    for ((_, _), e) in func.params.iter().zip(p.iter()) {
                        let value = self.translate_expr(e);
                        self.stack.push(value);
                    }
                    self.func_stack_offset = next_offset;
                    let ret = self.translate_block(&func.block);
                    self.stack.truncate(next_offset);
                    self.func_stack_offset = old_offset;
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

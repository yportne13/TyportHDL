use std::collections::HashMap;

use crate::{mir::*, built_in::{bi_print, bi_array}};

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
    //Class(Class),
}

/*#[derive(Clone, Debug)]
pub struct Class {
    values: Vec<Value>,

}*/

pub struct Interpreter {
    built_in_func: HashMap<String, Box<dyn Fn(&mut Interpreter, Vec<Value>) -> Value>>,
    stack: Vec<Value>,
    pub heap: HashMap<usize, HeapValue>,
    pub classes: HashMap<String, Class>,
    funcs: HashMap<String, Func>,
    func_stack_offset: usize,
}

impl Interpreter {
    pub fn new(classes: Vec<Class>) -> Self {
        let mut funcs_hash = HashMap::new();
        for f in classes {
            funcs_hash.insert(f.name.data.to_owned(), f);
        }
        let mut built_in_func: HashMap<String, Box<dyn Fn(&mut Interpreter, Vec<Value>) -> Value>> = HashMap::new();
        built_in_func.insert("print".to_owned(), Box::new(|vm, args| bi_print(vm, args)));
        built_in_func.insert("Array".to_owned(), Box::new(|vm, args| bi_array(vm, args)));

        Self {                                                                                                                                                                 
            built_in_func,
            stack: vec![],
            heap: Default::default(),
            classes: funcs_hash,
            funcs: Default::default(),
            func_stack_offset: 0,
        }
    }
    /*pub fn run(&'a mut self) -> Value {
        //let main = self.classes.get("main").unwrap().clone();
        //self.translate_block(&main.block)
        self.translate_block(&self.classes.get("main").unwrap().block)
    }*/
    pub fn translate_block(&mut self, stmts: &[Stmt]) -> Value {
        let mut ret = Value::Unit;
        for s in stmts {
            ret = self.translate_stmt(s);
        }
        ret
    }
    pub fn translate_stmt(&mut self, stmt: &Stmt) -> Value {
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
            Stmt::Block(b) => {
                self.translate_block(b)
            },
            Stmt::Func { name, params, return_type, block } => {
                let ret = Func {
                    name: name.clone(),
                    params: params.to_vec(),
                    return_type: return_type.clone(),
                    block: block.clone(),
                };
                self.funcs.insert(name.data.to_owned(), ret);
                Value::Unit
            }
        }
    }
    pub fn translate_expr(&mut self, expr: &Expression) -> Value {
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
                let mut args = vec![];
                for arg in p {
                    args.push(self.translate_expr(arg))
                }
                let mut built_in_func: HashMap<String, Box<dyn Fn(&mut Interpreter, Vec<Value>) -> Value>> = HashMap::new();
                built_in_func.insert("print".to_owned(), Box::new(bi_print));
                built_in_func.insert("Array".to_owned(), Box::new(bi_array));

                if let Some(f) = built_in_func.get(&name.data) {
                    f(self, args)
                } else if let Some(func) = self.funcs.clone().get(&name.data) {
                    let old_offset = self.func_stack_offset;
                    let next_offset = self.stack.len();
                    for (_, e) in func.params.iter().zip(p.iter()) {
                        let value = self.translate_expr(e);
                        self.stack.push(value);
                    }
                    self.func_stack_offset = next_offset;
                    let ret = self.translate_block(&func.block);
                    self.stack.truncate(next_offset);
                    self.func_stack_offset = old_offset;
                    ret
                } else {
                    let func = self.classes.get(&name.data).unwrap().clone();
                    let old_offset = self.func_stack_offset;
                    let next_offset = self.stack.len();
                    for ((_, _), e) in func.args.iter().zip(p.iter()) {
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

    fn int_func<F>(&mut self, l: &Expression, r: &Expression, f: F) -> Value
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

use std::collections::HashMap;

use crate::Span;

#[derive(Debug, Clone)]
pub enum Expression {
    Int(Span<i64>),
    String(usize, Span<String>),
    Bool(bool),
    Name(Span<usize>),
    Add(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Mul(Box<Expression>, Box<Expression>),
    Div(Box<Expression>, Box<Expression>),
    Eq(Box<Expression>, Box<Expression>),
    Neq(Box<Expression>, Box<Expression>),
    Call(Span<String>, Vec<Expression>),
    If(Box<Expression>, Vec<Stmt>, Option<Vec<Stmt>>),
}

#[derive(Debug, Clone)]
pub struct Func {
    pub name: Span<String>,
    pub params: Vec<usize>,
    pub return_type: Option<Span<String>>,
    pub block: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expr(Expression),
    Let(Span<usize>, Expression),
    Assign(Span<usize>, Expression),
    Return(Expression),
    While(Expression, Vec<Stmt>),
    Block(Vec<Stmt>),
    Func {
        name: Span<String>,
        params: Vec<usize>,
        return_type: Option<Span<String>>,
        block: Vec<Stmt>,
    },
}

#[derive(Debug, Clone)]
pub struct Class {
    //TODO: add type for object, class, case class, abstract class
    pub name: Span<String>,
    pub args: Vec<(Span<usize>, Span<String>)>,
    pub extends: Option<Span<String>>,
    pub with: Vec<Span<String>>,
    //pub value: Vec<Span<String>>,
    pub func: HashMap<String, usize>,
    pub funcs: HashMap<String, Func>,
    pub block: Vec<Stmt>,
}

pub fn hir_to_mir(from: Vec<crate::hir::Class>) -> Vec<Class> {
    from.into_iter()
        .map(|x| {
            let mut converter = MirConverter::new();
            converter.convert(x)
        })
        .collect()
}

struct MirConverter {
    rename: Vec<HashMap<String, usize>>,
    rename_idx: Vec<usize>,
    heap_idx: Vec<usize>,
}

impl MirConverter {
    pub fn new() -> Self {
        Self {
            rename: vec![Default::default()],
            rename_idx: vec![0],
            heap_idx: vec![0],
        }
    }
    pub fn convert(&mut self, from: crate::hir::Class) -> Class {
        for p in from.args.iter() {
            self.rename.last_mut().unwrap().insert(p.0.data.to_string(), *self.rename_idx.last().unwrap());
            *self.rename_idx.last_mut().unwrap() += 1;
        }
        Class {
            name: from.name.map(|x| x.to_owned()),
            args: from
                .args
                .into_iter()
                .enumerate()
                .map(|(idx, x)| (x.0.map(|_| idx), x.1.map(|x| x.to_owned())))
                .collect(),
            extends: from.extends.map(|x| x.map(|y| y.to_owned())),
            with: from.with.into_iter().map(|x| x.map(|y| y.to_owned())).collect(),
            func: from.func,
            funcs: Default::default(),//TODO:
            block: from.block.into_iter().map(|x| self.convert_stmt(x)).collect(),
        }
    }
    fn convert_stmt(&mut self, x: crate::hir::Stmt) -> Stmt {
        match x {
            crate::hir::Stmt::Expr(e) => Stmt::Expr(self.convert_expr(e)),
            crate::hir::Stmt::Let(a, b) => {
                self.rename.last_mut().unwrap().insert(a.data.to_string(), *self.rename_idx.last().unwrap());
                let ret = Stmt::Let(a.map(|_| *self.rename_idx.last().unwrap()), self.convert_expr(b));
                *self.rename_idx.last_mut().unwrap() += 1;
                ret
            },
            crate::hir::Stmt::Assign(a, b) => {
                let idx = self.rename.last().unwrap().get(&a.data).unwrap();
                Stmt::Assign(a.map(|_| *idx), self.convert_expr(b))
            },
            crate::hir::Stmt::Return(e) => Stmt::Return(self.convert_expr(e)),
            crate::hir::Stmt::While(e, v) => {
                Stmt::While(self.convert_expr(e), v.into_iter().map(|x| self.convert_stmt(x)).collect())
            }
            crate::hir::Stmt::Block(b) => Stmt::Block(b.into_iter().map(|bb| self.convert_stmt(bb)).collect()),
            crate::hir::Stmt::Func { name, params, return_type, block } => {
                self.rename.push(Default::default());
                self.rename_idx.push(0);
                self.heap_idx.push(0);
                let mut param = vec![];
                for (a, _type) in params {
                    self.rename.last_mut().unwrap().insert(a.data.to_string(), *self.rename_idx.last().unwrap());
                    param.push(*self.rename_idx.last_mut().unwrap());
                    *self.rename_idx.last_mut().unwrap() += 1;
                }
                let ret = Stmt::Func {
                    name: name.map(|x| x.to_owned()),
                    params: param,//params.into_iter().map(|x| x.0.map(|y| y.to_owned())).collect(),//TODO:
                    return_type: return_type.map(|x| x.map(|y| y.to_owned())),
                    block: block.into_iter().map(|x| self.convert_stmt(x)).collect(),//TODO:
                };
                self.rename.pop();
                self.rename_idx.pop();
                self.heap_idx.pop();
                ret
            },
        }
    }
    fn convert_expr(&mut self, x: crate::hir::Expression) -> Expression {
        match x {
            crate::hir::Expression::Int(x) => Expression::Int(x),
            crate::hir::Expression::String(x) => {
                let ret = Expression::String(*self.heap_idx.last().unwrap(), x);
                *self.heap_idx.last_mut().unwrap() += 1;
                ret
            },
            crate::hir::Expression::Bool(x) => Expression::Bool(x),
            crate::hir::Expression::Name(x) => {
                let idx = self.rename.last().unwrap().get(&x.data).unwrap();
                Expression::Name(x.map(|_| *idx))
            },
            crate::hir::Expression::Add(a, b) => {
                Expression::Add(Box::new(self.convert_expr(*a)), Box::new(self.convert_expr(*b)))
            }
            crate::hir::Expression::Sub(a, b) => {
                Expression::Sub(Box::new(self.convert_expr(*a)), Box::new(self.convert_expr(*b)))
            }
            crate::hir::Expression::Mul(a, b) => {
                Expression::Mul(Box::new(self.convert_expr(*a)), Box::new(self.convert_expr(*b)))
            }
            crate::hir::Expression::Div(a, b) => {
                Expression::Div(Box::new(self.convert_expr(*a)), Box::new(self.convert_expr(*b)))
            }
            crate::hir::Expression::Eq(a, b) => {
                Expression::Eq(Box::new(self.convert_expr(*a)), Box::new(self.convert_expr(*b)))
            }
            crate::hir::Expression::Neq(a, b) => {
                Expression::Neq(Box::new(self.convert_expr(*a)), Box::new(self.convert_expr(*b)))
            }
            crate::hir::Expression::Call(a, b) => {
                Expression::Call(a.map(|x| x.to_owned()), b.into_iter().map(|x| self.convert_expr(x)).collect())
            }
            crate::hir::Expression::ObjCall(a, b, c) => {
                Expression::Call(a.map(|x| x.to_owned()), c.into_iter().map(|x| self.convert_expr(x)).collect())//TODO: b
            }
            crate::hir::Expression::If(c, b, e) => Expression::If(
                Box::new(self.convert_expr(*c)),
                b.into_iter().map(|x| self.convert_stmt(x)).collect(),
                e.map(|x| x.into_iter().map(|y| self.convert_stmt(y)).collect()),
            ),
        }
    }
}

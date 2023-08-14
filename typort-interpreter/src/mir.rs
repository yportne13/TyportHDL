use std::collections::HashMap;

use crate::Span;

#[derive(Debug, Clone)]
pub enum Expression<'a> {
    Int(Span<i64>),
    String(usize, Span<String>),
    Bool(bool),
    Name(Span<usize>),
    Add(Box<Expression<'a>>, Box<Expression<'a>>),
    Sub(Box<Expression<'a>>, Box<Expression<'a>>),
    Mul(Box<Expression<'a>>, Box<Expression<'a>>),
    Div(Box<Expression<'a>>, Box<Expression<'a>>),
    Eq(Box<Expression<'a>>, Box<Expression<'a>>),
    Neq(Box<Expression<'a>>, Box<Expression<'a>>),
    Call(Span<&'a str>, Vec<Expression<'a>>),
    If(Box<Expression<'a>>, Vec<Stmt<'a>>, Option<Vec<Stmt<'a>>>),
}

#[derive(Debug, Clone)]
pub struct Func<'a> {
    pub name: Span<&'a str>,
    pub params: Vec<(Span<usize>, Span<&'a str>)>,
    pub return_type: Option<Span<&'a str>>,
    pub block: Vec<Stmt<'a>>,
}

#[derive(Debug, Clone)]
pub enum Stmt<'a> {
    Expr(Expression<'a>),
    Let(Span<usize>, Expression<'a>),
    Assign(Span<usize>, Expression<'a>),
    Return(Expression<'a>),
    While(Expression<'a>, Vec<Stmt<'a>>),
    Block(Vec<Stmt<'a>>),
}

pub fn hir_to_mir(from: Vec<crate::hir::Func<'_>>) -> Vec<Func<'_>> {
    from.into_iter()
        .map(|x| {
            let mut converter = MirConverter::new();
            converter.convert(x)
        })
        .collect()
}

struct MirConverter {
    rename: HashMap<String, usize>,
    rename_idx: usize,
    heap_idx: usize,
}

impl MirConverter {
    pub fn new() -> Self {
        Self {
            rename: Default::default(),
            rename_idx: 0,
            heap_idx: 0,
        }
    }
    pub fn convert<'b>(&'_ mut self, from: crate::hir::Func<'b>) -> Func<'b> {
        for p in from.params.iter() {
            self.rename.insert(p.0.data.to_string(), self.rename_idx);
            self.rename_idx += 1;
        }
        Func {
            name: from.name,
            params: from
                .params
                .into_iter()
                .enumerate()
                .map(|(idx, x)| (x.0.map(|_| idx), x.1))
                .collect(),
            return_type: from.return_type,
            block: from.block.into_iter().map(|x| self.convert_stmt(x)).collect(),
        }
    }
    fn convert_stmt<'b>(&'_ mut self, x: crate::hir::Stmt<'b>) -> Stmt<'b> {
        match x {
            crate::hir::Stmt::Expr(e) => Stmt::Expr(self.convert_expr(e)),
            crate::hir::Stmt::Let(a, b) => {
                self.rename.insert(a.data.to_string(), self.rename_idx);
                let ret = Stmt::Let(a.map(|_| self.rename_idx), self.convert_expr(b));
                self.rename_idx += 1;
                ret
            },
            crate::hir::Stmt::Assign(a, b) => {
                let idx = self.rename.get(a.data).unwrap();
                Stmt::Assign(a.map(|_| *idx), self.convert_expr(b))
            },
            crate::hir::Stmt::Return(e) => Stmt::Return(self.convert_expr(e)),
            crate::hir::Stmt::While(e, v) => {
                Stmt::While(self.convert_expr(e), v.into_iter().map(|x| self.convert_stmt(x)).collect())
            }
            crate::hir::Stmt::Block(b) => Stmt::Block(b.into_iter().map(|bb| self.convert_stmt(bb)).collect()),
        }
    }
    fn convert_expr<'b>(&'_ mut self, x: crate::hir::Expression<'b>) -> Expression<'b> {
        match x {
            crate::hir::Expression::Int(x) => Expression::Int(x),
            crate::hir::Expression::String(x) => {
                let ret = Expression::String(self.heap_idx, x);
                self.heap_idx += 1;
                ret
            },
            crate::hir::Expression::Bool(x) => Expression::Bool(x),
            crate::hir::Expression::Name(x) => {
                let idx = self.rename.get(x.data).unwrap();
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
                Expression::Call(a, b.into_iter().map(|x| self.convert_expr(x)).collect())
            }
            crate::hir::Expression::If(c, b, e) => Expression::If(
                Box::new(self.convert_expr(*c)),
                b.into_iter().map(|x| self.convert_stmt(x)).collect(),
                e.map(|x| x.into_iter().map(|y| self.convert_stmt(y)).collect()),
            ),
        }
    }
}

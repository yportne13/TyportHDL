use crate::Span;

#[derive(Debug, Clone)]
pub enum Expression<'a> {
    Int(Span<i64>),
    Bool(bool),
    Name(Span<&'a str>),
    Add(Box<Expression<'a>>, Box<Expression<'a>>),
    Sub(Box<Expression<'a>>, Box<Expression<'a>>),
    Mul(Box<Expression<'a>>, Box<Expression<'a>>),
    Div(Box<Expression<'a>>, Box<Expression<'a>>),
    Eq(Box<Expression<'a>>, Box<Expression<'a>>),
    Neq(Box<Expression<'a>>, Box<Expression<'a>>),
    Call(Span<&'a str>, Vec<Expression<'a>>),
    If(Box<Expression<'a>>, Vec<Stmt<'a>>, Option<Vec<Stmt<'a>>>),
}

impl<'a> From<crate::hir::Expression<'a>> for Expression<'a> {
    fn from(value: crate::hir::Expression<'a>) -> Self {
        match value {
            crate::hir::Expression::Int(x) => Expression::Int(x.into()),
            crate::hir::Expression::Bool(x) => Expression::Bool(x),
            crate::hir::Expression::Name(x) => Expression::Name(x.into()),
            crate::hir::Expression::Add(a, b) => {
                Expression::Add(Box::new((*a).into()), Box::new((*b).into()))
            }
            crate::hir::Expression::Sub(a, b) => {
                Expression::Sub(Box::new((*a).into()), Box::new((*b).into()))
            }
            crate::hir::Expression::Mul(a, b) => {
                Expression::Mul(Box::new((*a).into()), Box::new((*b).into()))
            }
            crate::hir::Expression::Div(a, b) => {
                Expression::Div(Box::new((*a).into()), Box::new((*b).into()))
            }
            crate::hir::Expression::Eq(a, b) => {
                Expression::Eq(Box::new((*a).into()), Box::new((*b).into()))
            }
            crate::hir::Expression::Neq(a, b) => {
                Expression::Neq(Box::new((*a).into()), Box::new((*b).into()))
            }
            crate::hir::Expression::Call(a, b) => {
                Expression::Call(a.into(), b.into_iter().map(|x| x.into()).collect())
            }
            crate::hir::Expression::If(c, b, e) => Expression::If(
                Box::new((*c).into()),
                b.into_iter().map(|x| x.into()).collect(),
                e.map(|x| x.into_iter().map(|y| y.into()).collect()),
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Func<'a> {
    pub name: Span<&'a str>,
    pub params: Vec<(Span<&'a str>, Span<&'a str>)>,
    pub return_type: Option<Span<&'a str>>,
    pub block: Vec<Stmt<'a>>,
}

#[derive(Debug, Clone)]
pub enum Stmt<'a> {
    Expr(Expression<'a>),
    Let(Span<&'a str>, Expression<'a>),
    Assign(Span<&'a str>, Expression<'a>),
    Return(Expression<'a>),
    //For(Span<&'a str>, Expression<'a>, Expression<'a>, Vec<Stmt<'a>>),
    While(Expression<'a>, Vec<Stmt<'a>>),
}

impl<'a> From<crate::hir::Stmt<'a>> for Stmt<'a> {
    fn from(value: crate::hir::Stmt<'a>) -> Self {
        match value {
            crate::hir::Stmt::Expr(e) => Stmt::Expr(e.into()),
            crate::hir::Stmt::Let(a, b) => Stmt::Let(a.into(), b.into()),
            crate::hir::Stmt::Assign(a, b) => Stmt::Assign(a.into(), b.into()),
            crate::hir::Stmt::Return(e) => Stmt::Return(e.into()),
            crate::hir::Stmt::While(e, v) => {
                Stmt::While(e.into(), v.into_iter().map(|x| x.into()).collect())
            }
        }
    }
}

pub fn hir_to_mir(from: Vec<crate::hir::Func<'_>>) -> Vec<Func<'_>> {
    from.into_iter()
        .map(|x| Func {
            name: x.name.into(),
            params: x
                .params
                .into_iter()
                .map(|x| (x.0.into(), x.1.into()))
                .collect(),
            return_type: x.return_type.map(|x| x.into()),
            block: x.block.into_iter().map(|x| x.into()).collect(),
        })
        .collect()
}

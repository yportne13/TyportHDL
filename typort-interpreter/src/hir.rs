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

impl<'a> From<typort_parser::simple_example::Expression<'a>> for Expression<'a> {
    fn from(value: typort_parser::simple_example::Expression<'a>) -> Self {
        match value {
            typort_parser::simple_example::Expression::Int(x) => Expression::Int(x.into()),
            typort_parser::simple_example::Expression::Bool(x) => Expression::Bool(x),
            typort_parser::simple_example::Expression::Name(x) => Expression::Name(x.into()),
            typort_parser::simple_example::Expression::Add(a, b) => {
                Expression::Add(Box::new((*a).into()), Box::new((*b).into()))
            }
            typort_parser::simple_example::Expression::Sub(a, b) => {
                Expression::Sub(Box::new((*a).into()), Box::new((*b).into()))
            }
            typort_parser::simple_example::Expression::Mul(a, b) => {
                Expression::Mul(Box::new((*a).into()), Box::new((*b).into()))
            }
            typort_parser::simple_example::Expression::Div(a, b) => {
                Expression::Div(Box::new((*a).into()), Box::new((*b).into()))
            }
            typort_parser::simple_example::Expression::Eq(a, b) => {
                Expression::Eq(Box::new((*a).into()), Box::new((*b).into()))
            }
            typort_parser::simple_example::Expression::Neq(a, b) => {
                Expression::Neq(Box::new((*a).into()), Box::new((*b).into()))
            }
            typort_parser::simple_example::Expression::Call(a, b) => {
                Expression::Call(a.into(), b.into_iter().map(|x| x.into()).collect())
            }
            typort_parser::simple_example::Expression::If(c, b, e) => Expression::If(
                Box::new((*c).into()),
                b.into_iter().flat_map(convert_stmt).collect(),
                e.map(|x| x.into_iter().flat_map(convert_stmt).collect()),
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

fn convert_stmt(value: typort_parser::simple_example::Stmt<'_>) -> Vec<Stmt<'_>> {
    match value {
        typort_parser::simple_example::Stmt::Expr(e) => vec![Stmt::Expr(e.into())],
        typort_parser::simple_example::Stmt::Let(a, b) => vec![Stmt::Let(a.into(), b.into())],
        typort_parser::simple_example::Stmt::Assign(a, b) => vec![Stmt::Assign(a.into(), b.into())],
        typort_parser::simple_example::Stmt::Return(e) => vec![Stmt::Return(e.into())],
        typort_parser::simple_example::Stmt::While(e, v) => {
            vec![Stmt::While(e.into(), v.into_iter().flat_map(convert_stmt).collect())]
        }
        typort_parser::simple_example::Stmt::For(v, from, to, b) => vec![
            Stmt::Let(v.clone().into(), from.into()),
            Stmt::While(
                Expression::Neq(Box::new(Expression::Name(v.clone().into())), Box::new(to.into())),
                [
                    b.into_iter().flat_map(convert_stmt).collect(),
                    vec![Stmt::Assign(v.clone().into(), Expression::Add(Box::new(Expression::Name(v.into())), Box::new(Expression::Int(Span{data: 1}))))]
                ].concat()
            )
        ],
    }
}

pub fn parse_to_hir(from: Vec<typort_parser::simple_example::Func<'_>>) -> Vec<Func<'_>> {
    from.into_iter()
        .map(|x| Func {
            name: x.name.into(),
            params: x
                .params
                .into_iter()
                .map(|x| (x.0.into(), x.1.into()))
                .collect(),
            return_type: x.return_type.map(|x| x.into()),
            block: x.block.into_iter().flat_map(convert_stmt).collect(),
        })
        .collect()
}

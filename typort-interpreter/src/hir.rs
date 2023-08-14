use crate::Span;

#[derive(Debug, Clone)]
pub enum Expression<'a> {
    Int(Span<i64>),
    String(Span<String>),
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
    While(Expression<'a>, Vec<Stmt<'a>>),
    Block(Vec<Stmt<'a>>),
}

/// convert parser result to hir
/// 1. for -> while
/// 2. xxx() -> xxx.apply()
struct HirConverter {

}

impl HirConverter {
    pub fn new() -> Self {
        Self {

        }
    }
    pub fn convert_stmt<'b>(&'_ mut self, value: typort_parser::simple_example::Stmt<'b>) -> Stmt<'b> {
        match value {
            typort_parser::simple_example::Stmt::Expr(e) => Stmt::Expr(self.convert_expr(e)),
            typort_parser::simple_example::Stmt::Let(a, b) => Stmt::Let(a.into(), self.convert_expr(b)),
            typort_parser::simple_example::Stmt::Assign(a, b) => Stmt::Assign(a.into(), self.convert_expr(b)),
            typort_parser::simple_example::Stmt::Return(e) => Stmt::Return(self.convert_expr(e)),
            typort_parser::simple_example::Stmt::While(e, v) => {
                Stmt::While(self.convert_expr(e), v.0.into_iter().map(|x| self.convert_stmt(x)).collect())
            }
            typort_parser::simple_example::Stmt::For(v, from, to, b) => Stmt::Block(vec![
                Stmt::Let(v.clone().into(), self.convert_expr(from)),
                Stmt::While(
                    Expression::Neq(Box::new(Expression::Name(v.clone().into())), Box::new(self.convert_expr(to))),
                    [
                        b.0.into_iter().map(|x| self.convert_stmt(x)).collect(),
                        vec![Stmt::Assign(v.clone().into(), Expression::Add(Box::new(Expression::Name(v.into())), Box::new(Expression::Int(Span{data: 1}))))]
                    ].concat()
                )
            ]),
        }
    }
    pub fn convert_expr<'b>(&'_ mut self, value: typort_parser::simple_example::Expression<'b>) -> Expression<'b> {
        match value {
            typort_parser::simple_example::Expression::Int(x) => Expression::Int(x.into()),
            typort_parser::simple_example::Expression::String(x) => Expression::String(x.into()),
            typort_parser::simple_example::Expression::Bool(x) => Expression::Bool(x),
            typort_parser::simple_example::Expression::Name(x) => Expression::Name(x.into()),
            typort_parser::simple_example::Expression::Add(a, b) => {
                Expression::Add(Box::new(self.convert_expr(*a)), Box::new(self.convert_expr(*b)))
            }
            typort_parser::simple_example::Expression::Sub(a, b) => {
                Expression::Sub(Box::new(self.convert_expr(*a)), Box::new(self.convert_expr(*b)))
            }
            typort_parser::simple_example::Expression::Mul(a, b) => {
                Expression::Mul(Box::new(self.convert_expr(*a)), Box::new(self.convert_expr(*b)))
            }
            typort_parser::simple_example::Expression::Div(a, b) => {
                Expression::Div(Box::new(self.convert_expr(*a)), Box::new(self.convert_expr(*b)))
            }
            typort_parser::simple_example::Expression::Eq(a, b) => {
                Expression::Eq(Box::new(self.convert_expr(*a)), Box::new(self.convert_expr(*b)))
            }
            typort_parser::simple_example::Expression::Neq(a, b) => {
                Expression::Neq(Box::new(self.convert_expr(*a)), Box::new(self.convert_expr(*b)))
            }
            typort_parser::simple_example::Expression::Call(a, b) => {
                Expression::Call(a.into(), b.into_iter().map(|x| self.convert_expr(x)).collect())
            }
            typort_parser::simple_example::Expression::If(c, b, e) => Expression::If(
                Box::new(self.convert_expr(*c)),
                b.0.into_iter().map(|x| self.convert_stmt(x)).collect(),
                e.map(|x| x.0.into_iter().map(|x| self.convert_stmt(x)).collect()),
            ),
        }
    }
}


pub fn parse_to_hir(from: Vec<typort_parser::simple_example::Func<'_>>) -> Vec<Func<'_>> {
    let mut converter = HirConverter::new();
    from.into_iter()
        .map(|x| Func {
            name: x.name.into(),
            params: x
                .params
                .into_iter()
                .map(|x| (x.0.into(), x.1.into()))
                .collect(),
            return_type: x.return_type.map(|x| x.into()),
            block: x.block.0.into_iter().map(|x| converter.convert_stmt(x)).collect(),
        })
        .collect()
}

use std::collections::HashMap;

use crate::{Diagnostic, Span};

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
    ObjCall(Span<&'a str>, Span<&'a str>, Vec<Expression<'a>>),
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
    Func {
        name: Span<&'a str>,
        params: Vec<(Span<&'a str>, Span<&'a str>)>,
        return_type: Option<Span<&'a str>>,
        block: Vec<Stmt<'a>>,
    }
}

#[derive(Debug, Clone)]
pub struct Class<'a> {
    //TODO: add type for object, class, case class, abstract class
    pub name: Span<&'a str>,
    pub args: Vec<(Span<&'a str>, Span<&'a str>)>,
    pub extends: Option<Span<&'a str>>,
    pub with: Vec<Span<&'a str>>,
    //pub value: Vec<Span<&'a str>>,
    pub func: HashMap<String, usize>,
    pub block: Vec<Stmt<'a>>,
}

type Mutable = bool;

/// convert parser result to hir
/// 1. for -> while
/// 2. xxx() -> xxx.apply()
struct HirConverter {
    values: Vec<HashMap<String, Mutable>>,
    diag: Vec<Diagnostic>,
}

impl HirConverter {
    pub fn new() -> Self {
        HirConverter { values: vec![Default::default()], diag: vec![] }
    }
    pub fn add_param<'b>(
        &'_ mut self,
        param: &'b [(
            typort_parser::simple_example::Span<&'b str>,
            typort_parser::simple_example::Span<&'b str>,
        )],
    ) {
        param.iter().for_each(|p| {
            self.values.last_mut().unwrap().insert(p.0.data.to_owned(), false);
        });
    }
    pub fn convert_stmt<'b>(
        &'_ mut self,
        value: typort_parser::simple_example::Stmt<'b>,
    ) -> Stmt<'b> {
        match value {
            typort_parser::simple_example::Stmt::Expr(e) => Stmt::Expr(self.convert_expr(e)),
            typort_parser::simple_example::Stmt::Val(a, b) => {
                if self.values.last_mut().unwrap().insert(a.data.to_owned(), false).is_some() {
                    self.diag.push(Diagnostic {
                        msg: format!("redefine {}", a.data),
                        range: a.range,
                    })
                }
                Stmt::Let(a.into(), self.convert_expr(b))
            }
            typort_parser::simple_example::Stmt::Var(a, b) => {
                if self.values.last_mut().unwrap().insert(a.data.to_owned(), true).is_some() {
                    self.diag.push(Diagnostic {
                        msg: format!("redefine {}", a.data),
                        range: a.range,
                    })
                }
                Stmt::Let(a.into(), self.convert_expr(b))
            }
            typort_parser::simple_example::Stmt::Assign(a, b) => {
                if let Some(mutable) = self.values.last().unwrap().get(a.data) {
                    if !mutable {
                        self.diag.push(Diagnostic {
                            msg: format!("\"{}\" is immutable", a.data),
                            range: a.range,
                        })
                    }
                } else {
                    self.diag.push(Diagnostic {
                        msg: format!("\"{}\" not defined", a.data),
                        range: a.range,
                    })
                }
                Stmt::Assign(a.into(), self.convert_expr(b))
            }
            typort_parser::simple_example::Stmt::Return(e) => Stmt::Return(self.convert_expr(e)),
            typort_parser::simple_example::Stmt::While(e, v) => Stmt::While(
                self.convert_expr(e),
                v.0.into_iter().map(|x| self.convert_stmt(x)).collect(),
            ),
            typort_parser::simple_example::Stmt::For(v, from, to, b) => {
                let name = v.data.to_owned();
                if self.values.last_mut().unwrap().insert(name.to_owned(), true).is_some() {
                    self.diag.push(Diagnostic {
                        msg: format!("redefine {name}"),
                        range: v.range,
                    })
                }
                let ret = Stmt::Block(vec![
                    Stmt::Let(v.clone().into(), self.convert_expr(from)),
                    Stmt::While(
                        Expression::Neq(
                            Box::new(Expression::Name(v.clone().into())),
                            Box::new(self.convert_expr(to)),
                        ),
                        [
                            b.0.into_iter().map(|x| self.convert_stmt(x)).collect(),
                            vec![Stmt::Assign(
                                v.clone().into(),
                                Expression::Add(
                                    Box::new(Expression::Name(v.into())),
                                    Box::new(Expression::Int(Span { data: 1 })),
                                ),
                            )],
                        ]
                        .concat(),
                    ),
                ]);
                self.values.last_mut().unwrap().remove(&name);
                ret
            }
            typort_parser::simple_example::Stmt::Func(f) => {
                self.values.push(Default::default());
                for param in f.params.iter() {
                    if self.values.last_mut().unwrap().insert(param.0.data.to_owned(), false).is_some() {
                        self.diag.push(Diagnostic {
                            msg: format!("redefine {}", param.0.data),
                            range: param.0.range,
                        })
                    }
                }
                let ret = Stmt::Func {
                    name: f.name.into(),
                    params: f.params.into_iter().map(|x| (x.0.into(), x.1.into())).collect(),
                    return_type: f.return_type.map(|x| x.into()),
                    block: f.block.0.into_iter().map(|x| self.convert_stmt(x)).collect(),
                };
                self.values.pop();
                ret
            }
        }
    }
    pub fn convert_expr<'b>(
        &'_ mut self,
        value: typort_parser::simple_example::Expression<'b>,
    ) -> Expression<'b> {
        match value {
            typort_parser::simple_example::Expression::Int(x) => Expression::Int(x.into()),
            typort_parser::simple_example::Expression::String(x) => Expression::String(x.into()),
            typort_parser::simple_example::Expression::Bool(x) => Expression::Bool(x),
            typort_parser::simple_example::Expression::Name(x) => {
                if !self.values.last().unwrap().contains_key(x.data) {
                    self.diag.push(Diagnostic {
                        msg: format!("use of undeclared value {}", x.data),
                        range: x.range,
                    })
                }
                Expression::Name(x.into())
            }
            typort_parser::simple_example::Expression::ObjVal(a, b) => {
                if !self.values.last().unwrap().contains_key(a.data) {
                    self.diag.push(Diagnostic {
                        msg: format!("use of undeclared value {}", a.data),
                        range: a.range,
                    })
                }
                Expression::Name(a.into()) //TODO: b
            }
            typort_parser::simple_example::Expression::Add(a, b) => Expression::Add(
                Box::new(self.convert_expr(*a)),
                Box::new(self.convert_expr(*b)),
            ),
            typort_parser::simple_example::Expression::Sub(a, b) => Expression::Sub(
                Box::new(self.convert_expr(*a)),
                Box::new(self.convert_expr(*b)),
            ),
            typort_parser::simple_example::Expression::Mul(a, b) => Expression::Mul(
                Box::new(self.convert_expr(*a)),
                Box::new(self.convert_expr(*b)),
            ),
            typort_parser::simple_example::Expression::Div(a, b) => Expression::Div(
                Box::new(self.convert_expr(*a)),
                Box::new(self.convert_expr(*b)),
            ),
            typort_parser::simple_example::Expression::Eq(a, b) => Expression::Eq(
                Box::new(self.convert_expr(*a)),
                Box::new(self.convert_expr(*b)),
            ),
            typort_parser::simple_example::Expression::Neq(a, b) => Expression::Neq(
                Box::new(self.convert_expr(*a)),
                Box::new(self.convert_expr(*b)),
            ),
            typort_parser::simple_example::Expression::Call(a, b) => {
                if self.values.last().unwrap().contains_key(a.data) {
                    Expression::ObjCall(
                        a.into(),
                        Span { data: "apply" },
                        b.into_iter().map(|x| self.convert_expr(x)).collect(),
                    )
                } else {
                    Expression::Call(
                        a.into(),
                        b.into_iter().map(|x| self.convert_expr(x)).collect(),
                    )
                }
            }
            typort_parser::simple_example::Expression::ObjCall(a, b, c) => Expression::ObjCall(
                a.into(),
                b.into(),
                c.into_iter().map(|x| self.convert_expr(x)).collect(),
            ),
            typort_parser::simple_example::Expression::If(c, b, e) => Expression::If(
                Box::new(self.convert_expr(*c)),
                b.0.into_iter().map(|x| self.convert_stmt(x)).collect(),
                e.map(|x| x.0.into_iter().map(|x| self.convert_stmt(x)).collect()),
            ),
        }
    }
}

pub fn parse_to_hir(from: Vec<typort_parser::simple_example::TopItem<'_>>) -> Vec<Class<'_>> {
    from.into_iter()
        .map(|x| {
            let mut converter = HirConverter::new();
            let ret = match x {
                typort_parser::simple_example::TopItem::Class(c) => {
                    converter.add_param(&c.args);
                    Class {
                        name: c.name.into(),
                        args: c
                            .args
                            .into_iter()
                            .map(|x| (x.0.into(), x.1.into()))
                            .collect(),
                        extends: c.extends.map(|x| x.into()),
                        with: c.with.into_iter().map(|x| x.into()).collect(),
                        func: Default::default(),
                        block: c
                            .block
                            .0
                            .into_iter()
                            .map(|x| converter.convert_stmt(x))
                            .collect(),
                    }
                }
                typort_parser::simple_example::TopItem::Object(o) => Class {
                    name: o.name.into(),
                    args: vec![],
                    extends: o.extends.map(|x| x.into()),
                    with: o.with.into_iter().map(|x| x.into()).collect(),
                    func: Default::default(),
                    block: o
                        .block
                        .0
                        .into_iter()
                        .map(|x| converter.convert_stmt(x))
                        .collect(),
                },
            };
            if !converter.diag.is_empty() {
                println!("{:?}", converter.diag); //TODO: do not print
            }
            ret
        })
        .collect()
}

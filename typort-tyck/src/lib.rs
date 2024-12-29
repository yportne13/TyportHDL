pub mod hir;
pub mod ty;

use std::collections::HashMap;

use typort_parser::{
    combinator::{Diagnostic, Maybe, ToSpan},
    expr::Expr,
    types::{Param, TypeExpr, TypeParam},
    Brace, Fn, Paren, Span, Square, Stmt,
};

type Error = String;

pub fn tyck(ast: Vec<Fn>, env: HashMap<String, Fn>) -> Result<Vec<hir::Fn>, Error> {
    ast.into_iter()
        .map(|x| tyck_fn(x, Default::default()))
        .collect::<Result<Vec<_>, Error>>()
}

pub fn tyck_fn(ast: Fn, env: HashMap<String, Fn>) -> Result<hir::Fn, Error> {
    let mut inference = Inference::new();
    let (new_fn, err, len) = inference.infer_fn(HashMap::new(), ast);
    if !err.is_empty() {
        println!("{:?}", err);
        todo!()
    }
    let mut substitution = (0..len).map(hir::TypeExpr::Variable).collect::<Vec<_>>();
    inference.solve_constraints(&mut substitution)?;
    Ok(Inference::substitute_fn(new_fn, &substitution))
}

impl TryFrom<TypeExpr> for hir::TypeExpr {
    type Error = Diagnostic;

    fn try_from(value: TypeExpr) -> Result<Self, Diagnostic> {
        match value {
            TypeExpr::Base(span) => Ok(hir::TypeExpr::Constructor(
                hir::TypeName::Span(span),
                vec![],
            )),
            TypeExpr::Arrow(span, span1, maybe) => Ok(hir::TypeExpr::Constructor(
                hir::TypeName::Span(span1.map(|_| "=>".to_owned())),
                vec![
                    hir::TypeExpr::Constructor(hir::TypeName::Span(span), vec![]),
                    match maybe.as_ref() {
                        Maybe::Some(x) => x.clone().try_into()?,
                        Maybe::Hole(span) => return Err(todo!()),
                    },
                ],
            )),
            TypeExpr::Tuple(paren) => match paren.data {
                Maybe::Some(x) => Ok(hir::TypeExpr::Constructor(
                    hir::TypeName::Span(paren.lparen.map(|_| format!("Tuple{}", x.len()))),
                    x.into_iter()
                        .map(|x| x.try_into())
                        .collect::<Result<Vec<_>, Diagnostic>>()?,
                )),
                Maybe::Hole(span) => todo!(),
            },
        }
    }
}

impl<'a, E> TryFrom<Maybe<TypeExpr, E>> for hir::TypeExpr {
    type Error = Diagnostic;

    fn try_from(value: Maybe<TypeExpr, E>) -> Result<Self, Self::Error> {
        match value {
            Maybe::Some(x) => x.try_into(),
            Maybe::Hole(span) => Err(todo!()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Constraint {
    CEquality(hir::TypeExpr, hir::TypeExpr),
}

/////////////////////////////////
// Type inference
/////////////////////////////////

#[derive(Debug)]
pub struct TypeError {
    pub message: String,
}

impl std::fmt::Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for TypeError {}

#[derive(Clone, Debug)]
pub struct Inference {
    type_constraints: Vec<Constraint>,
    //substitution: Vec<hir::TypeExpr>,
    substitution_len: usize,
}

impl Inference {
    pub fn new() -> Self {
        Self {
            type_constraints: Vec::new(),
            //substitution: Vec::new(),
            substitution_len: 0,
        }
    }

    pub fn fresh_type_variable(&mut self) -> hir::TypeExpr {
        //let result = hir::TypeExpr::Variable(self.substitution.len());
        //self.substitution.push(result.clone());
        let result = hir::TypeExpr::Variable(self.substitution_len);
        self.substitution_len += 1;
        result
    }

    pub fn infer_fn(
        &mut self,
        environment: HashMap<String, hir::TypeExpr>,
        fun: Fn,
    ) -> (hir::Fn, Vec<Diagnostic>, usize) {
        let mut err = vec![];
        //TODO:type param
        let new_name = fun
            .name
            .raise_err(&mut err)
            .unwrap_or_else(|e| e.map(|_| "".to_owned()));
        let param_len = fun
            .params
            .iter()
            .map(|x| match &x.data {
                Maybe::Some(x) => x.len(),
                Maybe::Hole(_) => 0,
            })
            .len();
        let new_return_type = fun.ret_type.0 .1.try_into().unwrap_or_else(|e| {
            err.push(e);
            self.fresh_type_variable()
        });
        let new_parameter_types: Vec<hir::TypeExpr> = fun
            .params
            .iter()
            //TODO: directly throw Maybe::Hole ?
            .flat_map(|x| x.data.clone().unwrap_or_else(|_| vec![]))
            .map(|p| {
                p.ty.clone().try_into().unwrap_or_else(|e| {
                    err.push(e);
                    self.fresh_type_variable()
                })
            })
            .collect();
        let new_parameters: Vec<_> = fun
            .params
            .iter()
            //TODO: directly throw Maybe::Hole ?
            .flat_map(|x| x.data.clone().unwrap_or_else(|_| vec![]))
            .zip(new_parameter_types.clone())
            .map(|(p, t)| hir::Param {
                name: p.name,
                ty: t,
            })
            .collect();
        let new_environment: HashMap<String, hir::TypeExpr> = environment
            .into_iter()
            .chain(
                new_parameters
                    .iter()
                    .map(|p| (p.name.data.clone(), p.ty.clone())),
            )
            .chain(std::iter::once((
                new_name.clone().data,
                hir::TypeExpr::Constructor(
                    hir::TypeName::Span(new_name.clone().map(|_| format!("Function{}", new_parameters.len()))),
                    new_parameters.iter().map(|p| p.ty.clone()).collect()
                )
            )))
            .collect();
        let new_body = self.infer_expr(new_environment, Some(new_return_type.clone()), fun.body);
        err.extend(new_body.1);
        /*if let Some(expected_type) = expected_type {
            self.type_constraints.push(Constraint::CEquality(
                expected_type,
                hir::TypeExpr::Constructor(
                    format!("Function{}", param_len),
                    new_parameter_types.into_iter().chain(vec![new_return_type.clone()]).collect(),
                ),
            ))
        }*/
        
        (
            hir::Fn {
                name: new_name,
                type_params: fun.type_params.map(|x| hir::TypeParam {
                    data: new_parameters.clone(),
                }),
                params: vec![new_parameters],//TODO:new_param
                ret_type: new_return_type,
                body: new_body.0,
            },
            err,
            self.substitution_len,
        )
    }

    pub fn infer_stmt(
        &mut self,
        mut environment: HashMap<String, hir::TypeExpr>,
        expected_type: Option<hir::TypeExpr>,
        stmt: Stmt,
    ) -> (hir::Stmt, Vec<Diagnostic>, HashMap<String, hir::TypeExpr>) {
        match stmt {
            Stmt::Return(span, expr) => todo!(),
            Stmt::Val {
                val: _,
                ident,
                eq,
                expr,
            } => match ident {
                Maybe::Some(ident) => {
                    let mut err = vec![];
                    eq.raise_err(&mut err);
                    //let new_type_annotation = type_annotation.unwrap_or_else(|| self.fresh_type_variable());
                    let new_type_annotation = self.fresh_type_variable();
                    let new_value = expr
                        .raise_err(&mut err)
                        .map(|x| {
                            self.infer_expr(
                                environment.clone(),
                                Some(new_type_annotation.clone()),
                                x,
                            )
                        })
                        .map(|(a, mut b)| {
                            err.append(&mut b);
                            a
                        })
                        .unwrap_or_else(|_| hir::Expr::Block(vec![]));
                    let new_environment = environment
                        .into_iter()
                        .chain(vec![(ident.data.clone(), new_type_annotation.clone())])
                        .collect();
                    (
                        hir::Stmt::Val {
                            ident,
                            expr: new_value,
                        },
                        err,
                        new_environment,
                    )
                }
                Maybe::Hole(span) => {
                    todo!()
                }
            },
            Stmt::Expr(expr) => {
                let (ret, err) = self.infer_expr(environment.clone(), expected_type, expr);
                (hir::Stmt::Expr(ret), err, environment)
            }
        }
    }

    pub fn infer_expr(
        &mut self,
        environment: HashMap<String, hir::TypeExpr>,
        expected_type: Option<hir::TypeExpr>,
        expression: Expr,
    ) -> (hir::Expr, Vec<Diagnostic>) {
        match expression {
            /*Expression::Lambda(parameters, return_type, body) => {
                let param_len = parameters.len();
                let new_return_type = return_type.unwrap_or_else(|| self.fresh_type_variable());
                let new_parameter_types: Vec<Type> = parameters
                    .iter()
                    .map(|p| p.type_annotation.clone().unwrap_or_else(|| self.fresh_type_variable()))
                    .collect();
                let new_parameters: Vec<Parameter> = parameters
                    .into_iter()
                    .zip(new_parameter_types.clone())
                    .map(|(p, t)| Parameter {
                        name: p.name,
                        type_annotation: Some(t),
                    })
                    .collect();
                let new_environment: HashMap<String, Type> = environment
                    .into_iter()
                    .chain(
                        new_parameters
                            .iter()
                            .map(|p| (p.name.clone(), p.type_annotation.clone().unwrap())),
                    )
                    .collect();
                let new_body = self.infer(new_environment, new_return_type.clone(), *body)?;
                self.type_constraints.push(Constraint::CEquality(
                    expected_type,
                    hir::TypeExpr::Constructor(
                        format!("Function{}", param_len),
                        new_parameter_types.into_iter().chain(vec![new_return_type.clone()]).collect(),
                    ),
                ));
                Ok(Expression::Lambda(new_parameters, Some(new_return_type), Box::new(new_body)))
            }*/
            Expr::Bool(x) => {
                if let Some(expected_type) = expected_type {
                    self.type_constraints.push(Constraint::CEquality(
                        expected_type,
                        hir::TypeExpr::Constructor(
                            hir::TypeName::Span(x.map(|_| "Boolean".to_string())),
                            vec![],
                        ),
                    ));
                }
                (hir::Expr::Bool(x), vec![])
            }
            Expr::Num(x) => {
                if let Some(expected_type) = expected_type {
                    self.type_constraints.push(Constraint::CEquality(
                        expected_type,
                        hir::TypeExpr::Constructor(
                            hir::TypeName::Span(x.map(|_| "Num".to_string())),
                            vec![],
                        ),
                    ));
                }
                (hir::Expr::Num(x), vec![])
            }
            Expr::Name(name) => {
                let mut err = vec![];
                let variable_type = environment
                    .get(&name.data)
                    .cloned()
                    .ok_or({ name.error(format!("Variable not in scope: {}", name.data)) })
                    .unwrap_or_else(|e| {
                        err.push(e);
                        self.fresh_type_variable()
                    });
                if let Some(expected_type) = expected_type {
                    self.type_constraints
                        .push(Constraint::CEquality(expected_type, variable_type.clone()));
                }
                (
                    hir::Expr::Name(name.map(|x| (x, variable_type.clone()))),
                    err,
                )
            }
            Expr::Paren(x) => {
                let mut err = vec![];
                x.rparen.raise_err(&mut err);
                (
                    hir::Expr::Paren(Box::new(
                        x.data
                            .raise_err(&mut err)
                            .map(|x| {
                                let ret =
                                    self.infer_expr(environment.clone(), expected_type.clone(), x);
                                err.extend(ret.1);
                                ret.0
                            })
                            .unwrap_or_else(|_| hir::Expr::Block(vec![])),
                    )),
                    err,
                )
            }
            Expr::Binary(lhs, op, rhs) => {
                let mut err = vec![];
                let (lhs, errl) = self.infer_expr(environment.clone(), None, *lhs);
                err.extend(errl);
                let rhs = match rhs.raise_err(&mut err) {
                    Maybe::Some(rhs) => {
                        let (rhs, err_ret) = self.infer_expr(environment, {
                            None // TODO: to check the type
                        }, *rhs);
                        err.extend(err_ret);
                        rhs
                    },
                    Maybe::Hole(_) => hir::Expr::Block(vec![]),
                };
                let op = match op {
                    typort_parser::expr::Operator::Add(span) => hir::Operator::Add(span),
                    typort_parser::expr::Operator::Sub(span) => hir::Operator::Sub(span),
                    typort_parser::expr::Operator::Mul(span) => hir::Operator::Mul(span),
                    typort_parser::expr::Operator::Div(span) => hir::Operator::Div(span),
                    typort_parser::expr::Operator::Op(span) => hir::Operator::Op(span),
                    typort_parser::expr::Operator::Unknown => hir::Operator::Unknown,
                };
                (hir::Expr::Binary(Box::new(lhs), op, Box::new(rhs)), err)
            }
            Expr::If {
                kw: _,
                cond,
                then,
                els,
            } => {
                let mut base_err = vec![];
                let (cond, err0) = self.infer_expr(
                    environment.clone(),
                    Some(hir::TypeExpr::Constructor(
                        hir::TypeName::Builtin("Bool".to_owned()),
                        vec![],
                    )),
                    cond.raise_err(&mut base_err)
                        .and_then(|x| x.data)
                        .map(|x| *x)
                        .unwrap_or_else(to_unit),
                );
                let (then, err1) = self.infer_expr(
                    environment.clone(),
                    expected_type.clone(),
                    then.raise_err(&mut base_err)
                        .map(|x| *x)
                        .unwrap_or_else(to_unit),
                );
                if let Some(els) = els {
                    let (els, err2) = self.infer_expr(
                        environment,
                        expected_type,
                        els.1
                            .raise_err(&mut base_err)
                            .map(|x| *x)
                            .unwrap_or_else(to_unit),
                    );
                    (
                        hir::Expr::If {
                            cond: Box::new(cond),
                            then: Box::new(then),
                            els: Some(Box::new(els)),
                        },
                        [base_err, err0, err1, err2].concat(),
                    )
                } else {
                    (
                        hir::Expr::If {
                            cond: Box::new(cond),
                            then: Box::new(then),
                            els: None,
                        },
                        [base_err, err0, err1].concat(),
                    )
                }
            }
            Expr::Call(function, arguments) => {
                let argument_types: Vec<_> = arguments
                    .data
                    .clone()
                    .unwrap_or_else(|_| vec![])
                    .iter()
                    .map(|_| self.fresh_type_variable())
                    .collect();
                let function_type = hir::TypeExpr::Constructor(
                    {
                        hir::TypeName::Span(
                            function
                                .to_span()
                                .map(|_| format!("Function{}", argument_types.len())),
                        )
                    },
                    argument_types
                        .clone()
                        .into_iter()
                        .chain(expected_type)
                        .collect(),
                );
                let (new_function, mut err) =
                    self.infer_expr(environment.clone(), Some(function_type), *function);
                let mut new_arguments = vec![];
                let iter = arguments
                    .data
                    .clone()
                    .unwrap_or_else(|_| vec![])
                    .into_iter()
                    .zip(argument_types);
                for (argument, t) in iter {
                    let ret = self.infer_expr(environment.clone(), Some(t), argument);
                    new_arguments.push(ret.0);
                    err.extend(ret.1);
                }
                (
                    hir::Expr::Call(Box::new(new_function), new_arguments.clone()),
                    err,
                )
            }
            Expr::Obj {
                lhs,
                endl,
                dot,
                obj,
            } => {
                todo!()
            }
            Expr::Tuple(_) => {
                todo!()
            }
            Expr::Block(block) => {
                let mut env = environment.clone();
                let mut ret_vec = vec![]; //TODO:unit
                let mut err = vec![];
                let blocks = block.data.unwrap_or_else(|_| vec![]);
                let block_len = blocks.len();
                for (idx, s) in blocks.into_iter().enumerate() {
                    let (ret_new, mut err_new, env_new) = self.infer_stmt(
                        env,
                        if idx == block_len - 1 {
                            expected_type.clone()
                        } else {
                            None
                        },
                        s,
                    );
                    err.append(&mut err_new);
                    env = env_new;
                    ret_vec.push(ret_new);
                }
                (hir::Expr::Block(ret_vec), err)
            }
        }
    }

    pub fn solve_constraints(
        &mut self,
        substitution: &mut Vec<hir::TypeExpr>,
    ) -> Result<(), String> {
        let constraint = self.type_constraints.drain(..).collect::<Vec<_>>();
        for constraint in constraint {
            if let Constraint::CEquality(t1, t2) = constraint {
                self.unify(t1, t2, substitution)?;
            }
        }
        Ok(())
    }

    pub fn unify(
        &mut self,
        t1: hir::TypeExpr,
        t2: hir::TypeExpr,
        substitution: &mut Vec<hir::TypeExpr>,
    ) -> Result<(), String> {
        match (t1, t2) {
            (hir::TypeExpr::Variable(i1), hir::TypeExpr::Variable(i2)) if i1 == i2 => {}
            (hir::TypeExpr::Variable(i), t2) if substitution[i] != hir::TypeExpr::Variable(i) => {
                self.unify(substitution[i].clone(), t2, substitution)?
            }
            (t1, hir::TypeExpr::Variable(i)) if substitution[i] != hir::TypeExpr::Variable(i) => {
                self.unify(t1, substitution[i].clone(), substitution)?
            }
            (hir::TypeExpr::Variable(i), t2) => {
                if self.occurs_in(i, &t2, substitution) {
                    return Err(format!(
                        "Infinite type: ${} = {:?}",
                        i,
                        Self::substitute(&t2, substitution)
                    ));
                }
                substitution[i] = t2;
            }
            (t1, hir::TypeExpr::Variable(i)) => {
                if self.occurs_in(i, &t1, substitution) {
                    return Err(format!(
                        "Infinite type: ${} = {:?}",
                        i,
                        Self::substitute(&t1, substitution)
                    ));
                }
                substitution[i] = t1;
            }
            (
                hir::TypeExpr::Constructor(name1, generics1),
                hir::TypeExpr::Constructor(name2, generics2),
            ) => {
                if name1 != name2 || generics1.len() != generics2.len() {
                    return Err(format!(
                        "Type mismatch: {:?} vs. {:?}",
                        Self::substitute(
                            &hir::TypeExpr::Constructor(name1, generics1),
                            substitution
                        ),
                        Self::substitute(
                            &hir::TypeExpr::Constructor(name2, generics2),
                            substitution
                        )
                    ));
                }
                for (t1, t2) in generics1.into_iter().zip(generics2) {
                    self.unify(t1, t2, substitution)?
                }
            }
        }
        Ok(())
    }

    pub fn occurs_in(
        &self,
        index: usize,
        t: &hir::TypeExpr,
        substitution: &Vec<hir::TypeExpr>,
    ) -> bool {
        match t {
            hir::TypeExpr::Variable(i) if substitution[*i] != hir::TypeExpr::Variable(*i) => {
                self.occurs_in(index, &substitution[*i], substitution)
            }
            hir::TypeExpr::Variable(i) => *i == index,
            hir::TypeExpr::Constructor(_, generics) => generics
                .iter()
                .any(|t| self.occurs_in(index, t, substitution)),
        }
    }

    pub fn substitute(t: &hir::TypeExpr, substitution: &Vec<hir::TypeExpr>) -> hir::TypeExpr {
        match t {
            hir::TypeExpr::Variable(i) if substitution[*i] != hir::TypeExpr::Variable(*i) => {
                Self::substitute(&substitution[*i], substitution)
            }
            hir::TypeExpr::Constructor(name, generics) => hir::TypeExpr::Constructor(
                name.clone(),
                generics
                    .iter()
                    .map(|t| Self::substitute(t, substitution))
                    .collect(),
            ),
            _ => t.clone(),
        }
    }

    pub fn substitute_stmt(stmt: hir::Stmt, substitution: &Vec<hir::TypeExpr>) -> hir::Stmt {
        match stmt {
            hir::Stmt::Return(span, expr) => {
                let new_expr = Self::substitute_expr(expr, substitution);
                hir::Stmt::Return(span, new_expr)
            }
            hir::Stmt::Val { ident, expr } => {
                let new_expr = Self::substitute_expr(expr, substitution);
                hir::Stmt::Val { ident, expr: new_expr }
            }
            hir::Stmt::Expr(expr) => {
                let new_expr = Self::substitute_expr(expr, substitution);
                hir::Stmt::Expr(new_expr)
            }
        }
    }

    pub fn substitute_expr(expr: hir::Expr, substitution: &Vec<hir::TypeExpr>) -> hir::Expr {
        match expr {
            hir::Expr::Bool(_) | hir::Expr::Num(_) | hir::Expr::Name(_) => expr,
            hir::Expr::Paren(paren) => todo!(),
            hir::Expr::Binary(lhs, op, rhs) => {
                let new_lhs = Self::substitute_expr(*lhs, substitution);
                let new_rhs = Self::substitute_expr(*rhs, substitution);
                hir::Expr::Binary(Box::new(new_lhs), op, Box::new(new_rhs))
            },
            hir::Expr::If { cond, then, els } => {
                let new_cond = Self::substitute_expr(*cond, substitution);
                let new_then = Self::substitute_expr(*then, substitution);
                let new_els = els.map(|e| Box::new(Self::substitute_expr(*e, substitution)));
                hir::Expr::If {
                    cond: Box::new(new_cond),
                    then: Box::new(new_then),
                    els: new_els,
                }
            },
            hir::Expr::Call(func, args) => {
                let new_func = Self::substitute_expr(*func, substitution);
                let new_args = args
                    .into_iter()
                    .map(|arg| Self::substitute_expr(arg, substitution))
                    .collect();
                hir::Expr::Call(Box::new(new_func), new_args)
            },
            hir::Expr::Obj {
                lhs,
                endl,
                dot,
                obj,
            } => todo!(),
            hir::Expr::Tuple(square) => todo!(),
            hir::Expr::Block(brace) => {
                let new_stmts = brace
                    .into_iter()
                    .map(|stmt| Self::substitute_stmt(stmt, substitution))
                    .collect();
                hir::Expr::Block(new_stmts)
            },
            /*Expr::Lambda(parameters, return_type, body) => {
                let new_return_type = return_type.map(|t| self.substitute(&t));
                let new_parameters: Vec<Parameter> = parameters
                    .into_iter()
                    .map(|p| Parameter {
                        name: p.name,
                        type_annotation: p.type_annotation.map(|t| self.substitute(&t)),
                    })
                    .collect();
                let new_body = self.substitute_expression(*body);
                Expression::Lambda(new_parameters, new_return_type, Box::new(new_body))
            }
            Expression::Apply(function, arguments) => {
                let new_function = self.substitute_expression(*function);
                let new_arguments: Vec<Expression> = arguments
                    .into_iter()
                    .map(|arg| self.substitute_expression(arg))
                    .collect();
                Expression::Apply(Box::new(new_function), new_arguments)
            }
            Expression::Variable(_) => expression,
            Expression::Let(name, type_annotation, value, body) => {
                let new_type_annotation = type_annotation.map(|t| self.substitute(&t));
                let new_value = self.substitute_expression(*value);
                let new_body = self.substitute_expression(*body);
                Expression::Let(name, new_type_annotation, Box::new(new_value), Box::new(new_body))
            }
            Expression::Array(item_type, items) => {
                let new_item_type = item_type.map(|t| self.substitute(&t));
                let new_items: Vec<Expression> = items
                    .into_iter()
                    .map(|item| self.substitute_expression(item))
                    .collect();
                Expression::Array(new_item_type, new_items)
            }*/
        }
    }

    pub fn substitute_fn(fun: hir::Fn, substitution: &Vec<hir::TypeExpr>) -> hir::Fn {
        hir::Fn {
            name: fun.name,
            type_params: fun.type_params,
            params: fun.params,
            ret_type: Self::substitute(&fun.ret_type, substitution),
            body: Self::substitute_expr(fun.body, substitution),
        }
    }
}

fn to_unit<T>(e: Span<T>) -> Expr {
    Expr::Block(Brace {
        lbrace: e.to_span(),
        endline1: None,
        data: typort_parser::combinator::Maybe::Some(vec![]),
        endline2: None,
        rbrace: typort_parser::combinator::Maybe::Some(e.to_span()),
    })
}

fn infer_maybe<I, O, F, Fd>(
    maybe: typort_parser::combinator::Maybe<I, typort_parser::Expect>,
    mut f: F,
    default: Fd,
    err: &mut Vec<Diagnostic>
) -> O
where
    F: FnMut(I) -> (O, Vec<Diagnostic>),
    Fd: std::ops::Fn() -> O,
{
    maybe.raise_err(err)
        .map(|x| {
            let ret = f(x);
            err.extend(ret.1);
            ret.0
        })
        .unwrap_or_else(|_| default())
}

/////////////////////////////////
// Tests
/////////////////////////////////

/*fn initial_environment() -> HashMap<String, Type> {
    vec![
        "+", "-", "*", "/",
    ]
    .into_iter()
    .map(|op| (
        op.to_string(),
        hir::TypeExpr::Constructor(
            "Function2".to_string(),
            vec![
                hir::TypeExpr::Constructor("Int".to_string(), vec![]),
                hir::TypeExpr::Constructor("Int".to_string(), vec![]),
                hir::TypeExpr::Constructor("Int".to_string(), vec![]),
            ],
        ),
    ))
    .collect()
}*/

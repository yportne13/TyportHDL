use typort_parser::Span;

#[derive(Clone, Debug)]
pub struct Fn {
    pub name: Span<String>,
    pub type_params: Option<TypeParam>,
    pub params: Vec<Vec<Param>>,
    pub ret_type: TypeExpr,
    pub body: Expr,
}

#[derive(Clone, Debug)]
pub enum Stmt {
    Return(Span<()>, Expr),
    Val { ident: Span<String>, expr: Expr },
    Expr(Expr),
}

#[derive(Clone, Debug)]
pub enum Expr {
    Bool(Span<bool>),
    Num(Span<i32>),
    Name(Span<(String, TypeExpr)>),
    Paren(Box<Expr>),
    Binary(Box<Expr>, Operator, Box<Expr>),
    If {
        cond: Box<Expr>,
        then: Box<Expr>,
        els: Option<Box<Expr>>,
    },
    Call(Box<Expr>, Vec<Expr>),
    Obj {
        lhs: Box<Expr>,
        endl: Option<Span<()>>,
        dot: Span<()>,
        obj: Span<String>,
    },
    Tuple(Vec<Expr>),
    Block(Vec<Stmt>),
}

#[derive(Clone, Debug)]
pub enum Operator {
    Add(Span<()>),
    Sub(Span<()>),
    Mul(Span<()>),
    Div(Span<()>),
    Op(Span<String>),
    Unknown,
}

#[derive(Clone, Debug)]
pub struct TypeParam {
    pub data: Vec<Param>,
}

#[derive(Clone, Debug)]
pub struct Param {
    pub name: Span<String>,
    pub ty: TypeExpr,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TypeName {
    Span(Span<String>),
    Builtin(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum TypeExpr {
    Constructor(TypeName, Vec<TypeExpr>),
    Variable(usize),
}

pub trait ExprIter {
    fn expr_iter(&self) -> Box<dyn Iterator<Item = &Expr> + '_>;
}

impl ExprIter for Fn {
    fn expr_iter(&self) -> Box<dyn Iterator<Item = &Expr> + '_> {
        let mut iterators: Vec<Box<dyn Iterator<Item = &Expr> + '_>> =
            vec![Box::new(std::iter::once(&self.body))];

        if let Some(type_params) = &self.type_params {
            iterators.push(Box::new(type_params.expr_iter()));
        }

        for param_group in &self.params {
            for param in param_group {
                iterators.push(Box::new(param.expr_iter()));
            }
        }

        Box::new(iterators.into_iter().flatten())
    }
}

impl ExprIter for Stmt {
    fn expr_iter(&self) -> Box<dyn Iterator<Item = &Expr> + '_> {
        match self {
            Stmt::Return(_, expr) => Box::new(std::iter::once(expr)),
            Stmt::Val { expr, .. } => Box::new(std::iter::once(expr)),
            Stmt::Expr(expr) => Box::new(std::iter::once(expr)),
        }
    }
}

impl ExprIter for Expr {
    fn expr_iter(&self) -> Box<dyn Iterator<Item = &Expr> + '_> {
        match self {
            Expr::Bool(_) | Expr::Num(_) | Expr::Name(_) => Box::new(std::iter::once(self)),
            Expr::Paren(expr) => Box::new(std::iter::once(self).chain(expr.expr_iter())),
            Expr::Binary(lhs, _, rhs) => Box::new(
                std::iter::once(self)
                    .chain(lhs.expr_iter())
                    .chain(rhs.expr_iter()),
            ),
            Expr::If { cond, then, els } => {
                if let Some(els) = els {
                    Box::new(
                        std::iter::once(self)
                            .chain(cond.expr_iter())
                            .chain(then.expr_iter())
                            .chain(els.expr_iter()),
                    )
                } else {
                    Box::new(
                        std::iter::once(self)
                            .chain(cond.expr_iter())
                            .chain(then.expr_iter()),
                    )
                }
            }
            Expr::Call(callee, args) => {
                let args_iter = args.iter().flat_map(|arg| arg.expr_iter());
                Box::new(
                    std::iter::once(self)
                        .chain(callee.expr_iter())
                        .chain(args_iter),
                )
            }
            Expr::Obj { lhs, .. } => Box::new(std::iter::once(self).chain(lhs.expr_iter())),
            Expr::Tuple(exprs) => Box::new(
                std::iter::once(self).chain(exprs.iter().flat_map(|expr| expr.expr_iter())),
            ),
            Expr::Block(exprs) => Box::new(
                std::iter::once(self).chain(exprs.iter().flat_map(|expr| expr.expr_iter())),
            ),
        }
    }
}

impl ExprIter for Operator {
    fn expr_iter(&self) -> Box<dyn Iterator<Item = &Expr> + '_> {
        Box::new(std::iter::empty())
    }
}

impl ExprIter for TypeParam {
    fn expr_iter(&self) -> Box<dyn Iterator<Item = &Expr> + '_> {
        Box::new(self.data.iter().flat_map(|param| param.expr_iter()))
    }
}

impl ExprIter for Param {
    fn expr_iter(&self) -> Box<dyn Iterator<Item = &Expr> + '_> {
        self.ty.expr_iter()
    }
}

impl ExprIter for TypeExpr {
    fn expr_iter(&self) -> Box<dyn Iterator<Item = &Expr> + '_> {
        match self {
            TypeExpr::Constructor(_, type_exprs) => Box::new(
                type_exprs
                    .iter()
                    .flat_map(|type_expr| type_expr.expr_iter()),
            ),
            TypeExpr::Variable(_) => Box::new(std::iter::empty()),
        }
    }
}

use crate::{
    arg_list,
    combinator::{AstDebug, Maybe, Parser},
    kw,
    lex::TokenNode,
    paren, string, Expect, Paren, Span, Square, TokenKind,
};
use TokenKind::*;

#[derive(Clone)]
pub enum Expr<'a> {
    Bool(Span<'a, bool>),
    Num(Span<'a, i32>),
    Name(Span<'a, String>),
    Paren(Box<Paren<'a, Expr<'a>>>),
    Binary(
        Box<Expr<'a>>,
        Operator<'a>,
        Maybe<'a, Box<Expr<'a>>, Expect>,
    ),
    Call(Box<Expr<'a>>, Paren<'a, Vec<Expr<'a>>>),
    Obj(Box<Expr<'a>>, Span<'a, ()>, Span<'a, String>),
    Tuple(Square<'a, Vec<Expr<'a>>>),
}

impl<'a> AstDebug for Expr<'a> {
    fn fmt(&self, s: &mut String, depth: usize) {
        match self {
            Expr::Bool(b) => s.push_str(&format!("{}{:?}\n", " ".repeat(depth), b)),
            Expr::Num(n) => s.push_str(&format!("{}{:?}\n", " ".repeat(depth), n)),
            Expr::Name(n) => s.push_str(&format!("{}{:?}\n", " ".repeat(depth), n)),
            Expr::Paren(p) => p.fmt(s, depth),
            Expr::Binary(l, op, r) => {
                let op = match op {
                    Operator::Add(x) => format!("+ @ {}", x.start_offset),
                    Operator::Sub(x) => format!("- @ {}", x.start_offset),
                    Operator::Mul(x) => format!("* @ {}", x.start_offset),
                    Operator::Div(x) => format!("/ @ {}", x.start_offset),
                    Operator::Op(x) => format!("{:?}", x),
                    Operator::Unknown => "Unknown".to_string(),
                };
                s.push_str(&format!("{}BinaryExpr\n", " ".repeat(depth)));
                l.fmt(s, depth + 1);
                s.push_str(&format!("{}{}\n", " ".repeat(depth + 1), op));
                r.fmt(s, depth + 1);
            }
            Expr::Call(l, param_list) => {
                s.push_str(&format!("{}Call\n", " ".repeat(depth)));
                l.fmt(s, depth + 1);
                param_list.fmt(s, depth + 1);
            }
            Expr::Obj(lhs, dot, name) => {
                s.push_str(&format!("{}Object\n", " ".repeat(depth)));
                lhs.fmt(s, depth + 1);
                s.push_str(&format!(
                    "{}. @ {}\n",
                    " ".repeat(depth + 1),
                    dot.start_offset
                ));
                s.push_str(&format!("{}{:?}\n", " ".repeat(depth + 1), name))
            }
            Expr::Tuple(x) => {
                s.push_str(&format!("{}Tuple\n", " ".repeat(depth)));
                x.fmt(s, depth + 1)
            }
        }
    }
}

impl<'a> std::fmt::Debug for Expr<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut ret = String::new();
        AstDebug::fmt(self, &mut ret, 0);
        write!(f, "{}", ret)
    }
}

#[derive(Clone, Debug)]
pub enum Operator<'a> {
    Add(Span<'a, ()>),
    Sub(Span<'a, ()>),
    Mul(Span<'a, ()>),
    Div(Span<'a, ()>),
    Op(Span<'a, String>),
    Unknown,
}

impl<'a> Operator<'a> {
    pub fn to_level(&self) -> Option<usize> {
        match self {
            Operator::Add(_) | Operator::Sub(_) => Some(0),
            Operator::Mul(_) | Operator::Div(_) => Some(1),
            Operator::Op(_) => Some(2),
            Operator::Unknown => None,
        }
    }
}

pub fn expr<'a: 'b, 'b>(input: &'b [TokenNode<'a>]) -> Option<(&'b [TokenNode<'a>], Expr<'a>)> {
    expr_rec(input, None)
}

fn expr_rec<'a: 'b, 'b>(
    input: &'b [TokenNode<'a>],
    left: Option<usize>,
) -> Option<(&'b [TokenNode<'a>], Expr<'a>)> {
    let (mut input, mut lhs) = expr_call(input)?;
    while let Some(op) = op(input) {
        if right_binds_tighter(left, op.1.to_level()) {
            input = op.0;
            match expr_rec(input, op.1.to_level()) {
                Some((i, rhs)) => {
                    input = i;
                    lhs = Expr::Binary(Box::new(lhs), op.1, Maybe::Some(Box::new(rhs)));
                }
                None => {
                    lhs = Expr::Binary(
                        Box::new(lhs),
                        op.1,
                        Maybe::Hole(Span {
                            data: Expect::Expr,
                            start_offset: input.last().map(|x| x.start_offset)?,
                            end_offset: input.last().map(|x| x.start_offset)?,
                            path: input.last().map(|x| x.path)?,
                        }),
                    );
                }
            }
        } else {
            break;
        }
    }
    Some((input, lhs))
}

fn op<'a: 'b, 'b>(input: &'b [TokenNode<'a>]) -> Option<(&'b [TokenNode<'a>], Operator<'a>)> {
    kw(Plus)
        .map(Operator::Add)
        .or(kw(Minus).map(Operator::Sub))
        .or(kw(Star).map(Operator::Mul))
        .or(kw(Slash).map(Operator::Div))
        .or(string(Op).map(Operator::Op))
        .parse(input)
}

/// expr_call = expr_call arg_list
///         | expr_call . ident
///         | expr_delimited
fn expr_call<'a: 'b, 'b>(input: &'b [TokenNode<'a>]) -> Option<(&'b [TokenNode<'a>], Expr<'a>)> {
    let (mut input, mut lhs) = expr_delimited(input)?;
    loop {
        if let Some((i, rhs)) = arg_list(input) {
            input = i;
            lhs = Expr::Call(Box::new(lhs), rhs);
        } else if let Some((i, rhs)) = kw(Dot).with(string(Ident)).parse(input) {
            input = i;
            lhs = Expr::Obj(Box::new(lhs), rhs.0, rhs.1);
        } else {
            break;
        }
    }
    Some((input, lhs))
}

fn right_binds_tighter(left: Option<usize>, right: Option<usize>) -> bool {
    let Some(right_tightness) = right else {
        return false;
    };
    let Some(left_tightness) = left else {
        //assert!(left == Eof);
        return true;
    };
    right_tightness > left_tightness
}

fn expr_delimited<'a: 'b, 'b>(
    input: &'b [TokenNode<'a>],
) -> Option<(&'b [TokenNode<'a>], Expr<'a>)> {
    kw(TrueKeyword)
        .map(|x| Expr::Bool(x.map(|_| true)))
        .or(kw(FalseKeyword).map(|x| Expr::Bool(x.map(|_| false))))
        .or(string(Num).map(|x| Expr::Num(x.map(|y| y.parse().unwrap()))))
        .or(string(Ident).map(Expr::Name))
        .or(paren(expr, Expect::Expr).map(|x| Expr::Paren(Box::new(x))))
        .parse(input)
}

#[test]
fn test() {
    use crate::tester;
    tester!(expr, "1");
    tester!(expr, "1 + 2");
    tester!(expr, "1 + 2 * 3");
    tester!(expr, "1 + 2 + 3");
    tester!(expr, "1 + foo.num + 3");
    tester!(expr, "foo.num + list.item.length().div(2)(3) * job()");
    //tester(expr, "1 + 2");
}

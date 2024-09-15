use crate::{
    arg_list,
    combinator::{Maybe, Parser},
    kw,
    lex::TokenNode,
    paren, string, Expect, Paren, Span, Square, TokenKind,
};
use TokenKind::*;

#[derive(Clone, Debug)]
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
    Tuple(Square<'a, Vec<Expr<'a>>>),
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

pub fn expr<'a>(input: &'a [TokenNode<'a>]) -> Option<(&'a [TokenNode<'a>], Expr<'a>)> {
    expr_rec(input, None)
}

fn expr_rec<'a>(
    input: &'a [TokenNode<'a>],
    left: Option<usize>,
) -> Option<(&'a [TokenNode<'a>], Expr<'a>)> {
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

fn op<'a>(input: &'a [TokenNode<'a>]) -> Option<(&'a [TokenNode<'a>], Operator<'a>)> {
    kw(Plus)
        .map(Operator::Add)
        .or(kw(Minus).map(Operator::Sub))
        .or(kw(Star).map(Operator::Mul))
        .or(kw(Slash).map(Operator::Div))
        .or(string(Op).map(Operator::Op))
        .parse(input)
}

/// expr_call = expr_delimited arg_list
///         | expr_delimited
fn expr_call<'a>(input: &'a [TokenNode<'a>]) -> Option<(&'a [TokenNode<'a>], Expr<'a>)> {
    expr_delimited
        .with(arg_list.option())
        .map(|(a, b)| match b {
            Some(b) => Expr::Call(Box::new(a), b),
            None => a,
        })
        .parse(input)
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

fn expr_delimited<'a>(input: &'a [TokenNode<'a>]) -> Option<(&'a [TokenNode<'a>], Expr<'a>)> {
    kw(TrueKeyword)
        .map(|x| Expr::Bool(x.map(|_| true)))
        .or(kw(FalseKeyword).map(|x| Expr::Bool(x.map(|_| false))))
        .or(string(Ident).map(Expr::Name))
        .or(paren(expr, Expect::Expr).map(|x| Expr::Paren(Box::new(x))))
        .parse(input)
}

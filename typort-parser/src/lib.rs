#![feature(pattern)]

use std::{cell::Cell, fmt, path::Path};

use crate::expr::expr;
use crate::lex::lex;
use crate::types::type_param;

pub use combinator::Span;
use combinator::{maybe, Maybe, Parser};
use expr::Expr;
use lex::TokenNode;

mod class;
mod combinator;
mod decl;
mod expr;
mod lex;
//pub mod resilient;
mod types;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TokenKind {
    DefKeyword,
    ValKeyword,
    VarKeyword,
    IfKeyword,
    ElseKeyword,
    TrueKeyword,
    FalseKeyword,
    ForKeyword,
    WhileKeyword,
    ReturnKeyword,

    LParen,
    RParen,
    LSquare,
    RSquare,
    LCurly,
    RCurly,
    Eq,
    Semi,
    Comma,
    Colon,
    Dot,
    Arrow,
    Plus,
    Minus,
    Star,
    Slash,

    Ident,
    Num,
    Op,
    EndLine,
    Str,

    ErrToken,

    Eof,
}

pub type Token<'a> = Span<'a, (&'a str, TokenKind)>;

#[derive(Clone, Copy, Debug)]
pub enum Expect {
    Ident,
    Eq,
    In,
    Then,
    Else,
    Arrow,
    Term,
    Colon,
    LParen,
    RParen,
    LSquare,
    RSquare,
    LCurly,
    RCurly,
    TypeExpr,
    Expr,
    Stmt,
    ArgList,
    Param,
}

#[derive(Clone, Debug)]
pub enum Stmt<'a> {
    Return(Span<'a, ()>, Expr<'a>),
    Val {
        val: Span<'a, ()>,
        ident: Maybe<'a, Span<'a, String>, Expect>,
        eq: Maybe<'a, Span<'a, ()>, Expect>,
        expr: Maybe<'a, Expr<'a>, Expect>,
    },
    Expr(Expr<'a>),
}

#[derive(Clone, Debug)]
pub enum Term<'a> {
    Lit(Span<'a, i32>),
    Bool(Span<'a, bool>),
    Var(Span<'a, String>),
    //Paren(Box<Term>),
    Lam {
        fun: Span<'a, ()>,
        ident: Maybe<'a, Span<'a, String>, Expect>,
        arrow: Maybe<'a, Span<'a, ()>, Expect>,
        term: Box<Maybe<'a, Term<'a>, Expect>>,
    },
    App(Box<Term<'a>>, Box<Term<'a>>),
    Rcd {
        left: Span<'a, ()>,
        data: Vec<(Span<'a, String>, Span<'a, ()>, Term<'a>)>,
        right: Span<'a, ()>,
    },
    Sel(Box<Term<'a>>, Span<'a, String>),
    Let {
        keyword: Span<'a, ()>,
        name: Maybe<'a, Span<'a, String>, Expect>,
        rhs: Box<Maybe<'a, Term<'a>, Expect>>,
    },
    IfExpr {
        if_kw: Span<'a, ()>,
        lparen: Maybe<'a, Span<'a, ()>, Expect>,
        cond: Box<Maybe<'a, Term<'a>, Expect>>,
        rparen: Maybe<'a, Span<'a, ()>, Expect>,
        rhs1: Box<Maybe<'a, Term<'a>, Expect>>,
        else_kw: Maybe<'a, Span<'a, ()>, Expect>,
        rhs2: Box<Maybe<'a, Term<'a>, Expect>>,
    },
}

#[derive(Clone, Debug)]
pub struct Fn<'a> {
    def: Span<'a, ()>,
    name: Maybe<'a, Span<'a, String>, Expect>,
    type_params: Option<TypeParam<'a>>,
    params: Vec<Paren<'a, Vec<Param<'a>>>>,
    ret_type: (
        (Span<'a, ()>, Maybe<'a, TypeExpr<'a>, Expect>),
        Maybe<'a, Span<'a, ()>, Expect>,
    ),
    body: Brace<'a, Vec<Stmt<'a>>>,
}

#[derive(Debug, Clone)]
pub struct Paren<'a, T> {
    lparen: Span<'a, ()>,
    data: Maybe<'a, T, Expect>,
    rparen: Maybe<'a, Span<'a, ()>, Expect>,
}

#[derive(Debug, Clone)]
pub struct Square<'a, T> {
    lbracket: Span<'a, ()>,
    data: Maybe<'a, T, Expect>,
    rbracket: Maybe<'a, Span<'a, ()>, Expect>,
}

#[derive(Clone, Debug)]
pub struct Brace<'a, T> {
    lbrace: Span<'a, ()>,
    data: Maybe<'a, T, Expect>,
    rbrace: Maybe<'a, Span<'a, ()>, Expect>,
}

fn kw<'a: 'b, 'b>(p: TokenKind) -> impl Parser<&'b [TokenNode<'a>], Span<'a, ()>> {
    move |input: &'b [TokenNode<'a>]| match input.first() {
        Some(x) if x.data.1 == p => input.get(1..).map(|i| (i, x.map(|_| ()))),
        _ => None,
    }
}

fn string<'a: 'b, 'b>(p: TokenKind) -> impl Parser<&'b [TokenNode<'a>], Span<'a, String>> {
    move |input: &'b [TokenNode<'a>]| match input.first() {
        Some(x) if x.data.1 == p => input.get(1..).map(|i| (i, x.map(|s| s.0.to_owned()))),
        _ => None,
    }
}

fn paren<'a: 'b, 'b, P, O>(p: P, expect: Expect) -> impl Parser<&'b [TokenNode<'a>], Paren<'a, O>>
where
    P: Parser<&'b [TokenNode<'a>], O>,
{
    kw(TokenKind::LParen)
        .with(maybe(p, expect))
        .with(maybe(kw(TokenKind::RParen), Expect::RParen))
        .map(|x| Paren {
            lparen: x.0 .0,
            data: x.0 .1,
            rparen: x.1,
        })
}

fn square<'a: 'b, 'b, P, O>(p: P, expect: Expect) -> impl Parser<&'b [TokenNode<'a>], Square<'a, O>>
where
    P: Parser<&'b [TokenNode<'a>], O>,
{
    kw(TokenKind::LSquare)
        .with(maybe(p, expect))
        .with(maybe(kw(TokenKind::RSquare), Expect::RSquare))
        .map(|x| Square {
            lbracket: x.0 .0,
            data: x.0 .1,
            rbracket: x.1,
        })
}

fn brace<'a: 'b, 'b, P, O>(p: P, expect: Expect) -> impl Parser<&'b [TokenNode<'a>], Brace<'a, O>>
where
    P: Parser<&'b [TokenNode<'a>], O>,
{
    kw(TokenKind::LCurly)
        .with(maybe(p, expect))
        .with(maybe(kw(TokenKind::RCurly), Expect::RCurly))
        .map(|x| Brace {
            lbrace: x.0 .0,
            data: x.0 .1,
            rbrace: x.1,
        })
}

use types::{param, type_expr, Param, TypeExpr, TypeParam};
use TokenKind::*;

pub fn parse<'a>(text: &'a str, path: &'a Path) -> Vec<Fn<'a>> {
    let input = Span {
        data: text,
        start_offset: 0,
        end_offset: text.len() as u32,
        path,
    };
    let tokens = lex(input).expect("some unexpected error happens in compiler. lex error");
    if !tokens.0.data.is_empty() {
        panic!("some unexpected error happens in compiler. lex error");
    }
    let ret = file(&tokens.1).expect("some unexpected error happens in compiler. parse error");
    if !ret.0.is_empty() {
        println!("{:#?}", ret.0);
        panic!("some unexpected error happens in compiler. parse error")
    }
    ret.1
}

fn file<'a: 'b, 'b>(input: &'b [TokenNode<'a>]) -> Option<(&'b [TokenNode<'a>], Vec<Fn<'a>>)> {
    func.many0().parse(input)
}

/// def ident [ type_param ] param_list (: type_expr = | [=] ) block
fn func<'a: 'b, 'b>(input: &'b [TokenNode<'a>]) -> Option<(&'b [TokenNode<'a>], Fn<'a>)> {
    kw(DefKeyword)
        .with(maybe(string(Ident), Expect::Ident))
        .with(type_param.option())
        .with(param_list.many0())
        .with(
            kw(Colon)
                .with(maybe(type_expr, Expect::TypeExpr))
                .with(maybe(kw(Eq), Expect::Eq)),
        )
        .with(block)
        .map(
            |(((((def, name), type_params), params), ret_type), body)| Fn {
                def,
                name,
                type_params,
                params,
                ret_type,
                body,
            },
        )
        .parse(input)
}

const PARAM_LIST_RECOVERY: &[TokenKind] = &[DefKeyword, LCurly];
fn param_list<'a: 'b, 'b>(
    input: &'b [TokenNode<'a>],
) -> Option<(&'b [TokenNode<'a>], Paren<'a, Vec<Param<'a>>>)> {
    paren(param.many0_sep(kw(TokenKind::Comma)), Expect::Param).parse(input)
}

const STMT_RECOVERY: &[TokenKind] = &[DefKeyword];
const EXPR_FIRST: &[TokenKind] = &[Num, TrueKeyword, FalseKeyword, Ident, LParen];
fn block<'a: 'b, 'b>(
    input: &'b [TokenNode<'a>],
) -> Option<(&'b [TokenNode<'a>], Brace<'a, Vec<Stmt<'a>>>)> {
    brace(stmt.many0(), Expect::Stmt).parse(input)
}

fn stmt<'a: 'b, 'b>(input: &'b [TokenNode<'a>]) -> Option<(&'b [TokenNode<'a>], Stmt<'a>)> {
    stmt_val
        .or(stmt_return)
        .or(expr.map(Stmt::Expr))
        .parse(input)
}

fn stmt_val<'a: 'b, 'b>(input: &'b [TokenNode<'a>]) -> Option<(&'b [TokenNode<'a>], Stmt<'a>)> {
    kw(ValKeyword)
        .with(maybe(string(Ident), Expect::Ident))
        .with(maybe(kw(Eq), Expect::Eq))
        .with(maybe(expr, Expect::Expr))
        .map(|x| Stmt::Val {
            val: x.0 .0 .0,
            ident: x.0 .0 .1,
            eq: x.0 .1,
            expr: x.1,
        })
        .parse(input)
}

/// stmt_return ::= return expr
fn stmt_return<'a: 'b, 'b>(input: &'b [TokenNode<'a>]) -> Option<(&'b [TokenNode<'a>], Stmt<'a>)> {
    kw(ReturnKeyword)
        .with(expr)
        .map(|x| Stmt::Return(x.0, x.1))
        .parse(input)
}

/// arg_list ::= ( {expr [,]} )
pub fn arg_list<'a: 'b, 'b>(
    input: &'b [TokenNode<'a>],
) -> Option<(&'b [TokenNode<'a>], Paren<'a, Vec<Expr<'a>>>)> {
    paren(expr.many0_sep(kw(Comma)), Expect::ArgList).parse(input)
}

#[macro_export]
macro_rules! tester {
    ($p:expr, $input:expr) => {
        let input = Span {
            data: $input,
            start_offset: 0,
            end_offset: $input.len() as u32,
            path: std::path::Path::new(""),
        };
        let tokens = $crate::lex(input).unwrap();
        // 将 tokens.1 的生命周期显式绑定到 'b
        let ret = $p.parse(&tokens.1);
        println!("{:#?}", ret);
    };
}

#[test]
fn smoke() {
    let text = "
def f(): Int = {
  val x = 1 +
  val y = 2
}

def uncurry[A: U, B: U, C: U](t: (A, B), f: A -> B -> C): C = {
  f(t(0))(t(1))
}
";
    let path = std::path::PathBuf::from("./");
    let cst = parse(text, &path);
    println!("{:#?}", cst);
    /*let lex = lex(Span {
        data: text,
        start_offset: 0,
        end_offset: text.len() as u32,
        path: &path,
    })
    .unwrap();
    eprintln!("{:?}\n\n{:#?}", lex.0, lex.1);*/
}

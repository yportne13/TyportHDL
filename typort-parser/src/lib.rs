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

pub enum Type<'a> {
    Top,
    Bot,
    Union {
        lhs: Box<Type<'a>>,
        rhs: Box<Type<'a>>,
    },
    Inter {
        lhs: Box<Type<'a>>,
        rhs: Box<Type<'a>>,
    },
    Fun {
        arg: Box<Type<'a>>,
        ret: Box<Type<'a>>,
    },
    Record {
        fields: Vec<(String, Type<'a>)>,
    },
    Recursive {
        uv: TypeVariable<'a>,
        body: Box<Type<'a>>,
    },
    Primitive {
        name: Span<'a, String>,
    },
    Variable(TypeVariable<'a>),
}

pub struct TypeVariable<'a> {
    name_hint: Span<'a, String>,
    hash: Span<'a, i32>,
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

fn kw<'a, 'b: 'a>(p: TokenKind) -> impl Parser<&'a [TokenNode<'b>], Span<'a, ()>> {
    move |input: &'a [TokenNode<'b>]| match input.first() {
        Some(x) if x.data.1 == p => input.get(1..).map(|i| (i, x.map(|_| ()))),
        _ => None,
    }
}

fn string<'a>(p: TokenKind) -> impl Parser<&'a [TokenNode<'a>], Span<'a, String>> {
    move |input: &'a [TokenNode<'a>]| match input.first() {
        Some(x) if x.data.1 == p => input.get(1..).map(|i| (i, x.map(|s| s.0.to_owned()))),
        _ => None,
    }
}

fn paren<'a, P, O>(p: P, expect: Expect) -> impl Parser<&'a [TokenNode<'a>], Paren<'a, O>>
where
    P: Parser<&'a [TokenNode<'a>], O>,
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

fn square<'a, P, O>(p: P, expect: Expect) -> impl Parser<&'a [TokenNode<'a>], Square<'a, O>>
where
    P: Parser<&'a [TokenNode<'a>], O>,
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

fn brace<'a, P, O>(p: P, expect: Expect) -> impl Parser<&'a [TokenNode<'a>], Brace<'a, O>>
where
    P: Parser<&'a [TokenNode<'a>], O>,
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

fn term<'a>(input: &'a [TokenNode<'a>]) -> Option<(&'a [TokenNode<'a>], Term<'a>)> {
    r#let.or(fun).or(ite).or(apps).parse(input)
}

fn const_or_var<'a>(input: &'a [TokenNode<'a>]) -> Option<(&'a [TokenNode<'a>], Term<'a>)> {
    match input.first() {
        Some(x) if x.data.1 == TokenKind::Num => input
            .get(1..)
            .map(|i| (i, Term::Lit(x.map(|d| d.0.parse().unwrap())))),
        Some(x) if x.data.1 == TokenKind::TrueKeyword => {
            input.get(1..).map(|i| (i, Term::Bool(x.map(|_| true))))
        }
        Some(x) if x.data.1 == TokenKind::FalseKeyword => {
            input.get(1..).map(|i| (i, Term::Bool(x.map(|_| false))))
        }
        Some(x) if x.data.1 == TokenKind::Ident => input
            .get(1..)
            .map(|i| (i, Term::Var(x.map(|d| d.0.to_owned())))),
        _ => None,
    }
}

fn parens<'a>(input: &'a [TokenNode<'a>]) -> Option<(&'a [TokenNode<'a>], Term<'a>)> {
    kw(TokenKind::LParen)
        .with(term)
        .with(kw(TokenKind::RParen))
        .map(|x| x.0 .1)
        .parse(input)
}

fn subterm_no_sel<'a>(input: &'a [TokenNode<'a>]) -> Option<(&'a [TokenNode<'a>], Term<'a>)> {
    parens.or(record).or(const_or_var).parse(input)
}

fn subterm<'a>(input: &'a [TokenNode<'a>]) -> Option<(&'a [TokenNode<'a>], Term<'a>)> {
    let (input, lhs) = subterm_no_sel(input)?;
    let (input, rhs) = (kw(TokenKind::Dot)
        .with(string(TokenKind::Ident))
        .map(|x| x.1))
    .many0()
    .parse(input)?;
    Some((
        input,
        rhs.into_iter()
            .fold(lhs, |acc, ident| Term::Sel(Box::new(acc), ident)),
    ))
}

fn record<'a>(input: &'a [TokenNode<'a>]) -> Option<(&'a [TokenNode<'a>], Term<'a>)> {
    let (input, l) = kw(TokenKind::LCurly).parse(input)?;
    let (input, data) = string(TokenKind::Ident)
        .with(kw(TokenKind::Eq))
        .with(term)
        .map(|x| (x.0 .0, x.0 .1, x.1))
        .many0_sep(kw(TokenKind::Semi))
        .parse(input)?;
    let (input, r) = kw(TokenKind::RCurly).parse(input)?;
    Some((
        input,
        Term::Rcd {
            left: l,
            data,
            right: r,
        },
    ))
}

fn fun<'a>(input: &'a [TokenNode<'a>]) -> Option<(&'a [TokenNode<'a>], Term<'a>)> {
    let (input, fun) = kw(TokenKind::DefKeyword).parse(input)?;
    let (input, ident) = maybe(string(TokenKind::Ident), Expect::Ident).parse(input)?;
    let (input, arrow) = maybe(kw(TokenKind::Arrow), Expect::Arrow).parse(input)?;
    let (input, rhs) = maybe(term, Expect::Term).parse(input)?;
    Some((
        input,
        Term::Lam {
            fun,
            ident,
            arrow,
            term: Box::new(rhs),
        },
    ))
}

fn r#let<'a>(input: &'a [TokenNode<'a>]) -> Option<(&'a [TokenNode<'a>], Term<'a>)> {
    let (input, keyword) = kw(TokenKind::ValKeyword).parse(input)?;
    let (input, name) = maybe(string(TokenKind::Ident), Expect::Ident).parse(input)?;
    let (input, eq) = maybe(kw(TokenKind::Eq), Expect::Eq).parse(input)?;
    let (input, term1) = maybe(term, Expect::Term).parse(input)?;
    Some((
        input,
        Term::Let {
            keyword,
            name,
            rhs: Box::new(term1),
        },
    ))
}

fn ite<'a>(input: &'a [TokenNode<'a>]) -> Option<(&'a [TokenNode<'a>], Term<'a>)> {
    let (input, if_kw) = kw(TokenKind::IfKeyword).parse(input)?;
    let (input, lparen) = maybe(kw(TokenKind::LParen), Expect::LParen).parse(input)?;
    let (input, cond) = maybe(term, Expect::Term).parse(input)?;
    let (input, rparen) = maybe(kw(TokenKind::RParen), Expect::RParen).parse(input)?;
    let (input, rhs1) = maybe(term, Expect::Term).parse(input)?;
    let (input, else_kw) = maybe(kw(TokenKind::ElseKeyword), Expect::Else).parse(input)?;
    let (input, rhs2) = maybe(term, Expect::Term).parse(input)?;
    Some((
        input,
        Term::IfExpr {
            if_kw,
            lparen,
            cond: Box::new(cond),
            rparen,
            rhs1: Box::new(rhs1),
            else_kw,
            rhs2: Box::new(rhs2),
        },
    ))
}

fn apps<'a>(input: &'a [TokenNode<'a>]) -> Option<(&'a [TokenNode<'a>], Term<'a>)> {
    subterm
        .many1()
        .map(|x| {
            x.into_iter()
                .reduce(|a, b| Term::App(Box::new(a), Box::new(b)))
                .unwrap()
        })
        .parse(input)
}

use types::{param, Param};
use TokenKind::*;
/*
pub fn parse<'a>(text: &'a str, path: &'a Path) -> (Tree<'a>, Vec<PError<'a>>) {
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
    let mut p = Parser::new(tokens.1);
    file(&mut p);
    p.build_tree()
}

fn file(p: &mut Parser) {
    let m = p.open();
    while !p.eof() {
        if p.at(DefKeyword) {
            func(p)
        } else {
            p.advance_with_error(ErrKind::TreeKind(Fn));
        }
    }
    p.close(m, File);
}

/// def ident [ type_param ] param_list (: type_expr = | [=] ) block
fn func(p: &mut Parser) {
    assert!(p.at(DefKeyword));
    let m = p.open();
    p.expect(DefKeyword);
    p.expect(Ident);
    if p.at(LSquare) {
        type_param(p);
    }
    while p.at(LParen) {
        param_list(p);
    }
    if p.eat(Colon) {
        type_expr(p);
        p.expect(Eq);
    } else if p.eat(Eq) {
        p.expect(Eq);
    }
    if p.at(LCurly) {
        block(p);
    }
    p.close(m, Fn);
}*/

const PARAM_LIST_RECOVERY: &[TokenKind] = &[DefKeyword, LCurly];
fn param_list<'a>(
    input: &'a [TokenNode<'a>],
) -> Option<(&'a [TokenNode<'a>], Paren<'a, Vec<Param<'a>>>)> {
    paren(param.many0_sep(kw(TokenKind::Comma)), Expect::Param).parse(input)
}

const STMT_RECOVERY: &[TokenKind] = &[DefKeyword];
const EXPR_FIRST: &[TokenKind] = &[Num, TrueKeyword, FalseKeyword, Ident, LParen];
fn block<'a>(
    input: &'a [TokenNode<'a>],
) -> Option<(&'a [TokenNode<'a>], Brace<'a, Vec<Stmt<'a>>>)> {
    brace(stmt.many0(), Expect::Stmt).parse(input)
}

fn stmt<'a>(input: &'a [TokenNode<'a>]) -> Option<(&'a [TokenNode<'a>], Stmt<'a>)> {
    stmt_val
        .or(stmt_return)
        .or(expr.map(Stmt::Expr))
        .parse(input)
}

fn stmt_val<'a>(input: &'a [TokenNode<'a>]) -> Option<(&'a [TokenNode<'a>], Stmt<'a>)> {
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
fn stmt_return<'a>(input: &'a [TokenNode<'a>]) -> Option<(&'a [TokenNode<'a>], Stmt<'a>)> {
    kw(ReturnKeyword)
        .with(expr)
        .map(|x| Stmt::Return(x.0, x.1))
        .parse(input)
}

/// arg_list ::= ( {expr [,]} )
pub fn arg_list<'a>(
    input: &'a [TokenNode<'a>],
) -> Option<(&'a [TokenNode<'a>], Paren<'a, Vec<Expr<'a>>>)> {
    paren(expr.many0_sep(kw(Comma)), Expect::ArgList).parse(input)
}

#[test]
fn smoke() {
    let text = "
def f() {
  val x = 1 +
  val y = 2
}

def uncurry[A: U, B: U, C: U](t: (A, B), f: A -> B -> C): C = {
  f(t(0))(t(1))
}
";
    let path = std::path::PathBuf::from("./");
    //let cst = parse(text, &path);
    let lex = lex(Span {
        data: text,
        start_offset: 0,
        end_offset: text.len() as u32,
        path: &path,
    })
    .unwrap();
    eprintln!("{:?}\n\n{:#?}", lex.0, lex.1);
}

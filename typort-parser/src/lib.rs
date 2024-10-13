#![feature(pattern)]

use std::path::Path;

use crate::expr::expr;
use crate::lex::lex;
use crate::types::type_param;

pub use combinator::Span;
use combinator::{maybe, AstDebug, Maybe, Parser};
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
pub struct Fn<'a> {
    pub def: Span<'a, ()>,
    pub name: Maybe<'a, Span<'a, String>, Expect>,
    pub type_params: Option<TypeParam<'a>>,
    pub params: Vec<Paren<'a, Vec<Param<'a>>>>,
    pub ret_type: (
        (Span<'a, ()>, Maybe<'a, TypeExpr<'a>, Expect>),
        Maybe<'a, Span<'a, ()>, Expect>,
    ),
    pub body: Brace<'a, Vec<Stmt<'a>>>,
}

#[derive(Debug, Clone)]
pub struct Paren<'a, T> {
    pub lparen: Span<'a, ()>,
    pub data: Maybe<'a, T, Expect>,
    pub rparen: Maybe<'a, Span<'a, ()>, Expect>,
}

impl<'a, T: AstDebug> AstDebug for Paren<'a, T> {
    fn fmt(&self, s: &mut String, depth: usize) {
        s.push_str(&format!(
            "{}( @ {}\n",
            " ".repeat(depth),
            self.lparen.start_offset
        ));
        self.data.fmt(s, depth + 1);
        s.push_str(&format!("{})\n", " ".repeat(depth)));
    }
}

#[derive(Debug, Clone)]
pub struct Square<'a, T> {
    pub lbracket: Span<'a, ()>,
    pub data: Maybe<'a, T, Expect>,
    pub rbracket: Maybe<'a, Span<'a, ()>, Expect>,
}

impl<'a, T: AstDebug> AstDebug for Square<'a, T> {
    fn fmt(&self, s: &mut String, depth: usize) {
        s.push_str(&format!(
            "{}[ @ {}\n",
            " ".repeat(depth),
            self.lbracket.start_offset
        ));
        self.data.fmt(s, depth + 1);
        s.push_str(&format!("{}]\n", " ".repeat(depth)));
    }
}

#[derive(Clone, Debug)]
pub struct Brace<'a, T> {
    pub lbrace: Span<'a, ()>,
    pub endline1: Option<Span<'a, ()>>,
    pub data: Maybe<'a, T, Expect>,
    pub endline2: Option<Span<'a, ()>>,
    pub rbrace: Maybe<'a, Span<'a, ()>, Expect>,
}

impl<'a, T: AstDebug> AstDebug for Brace<'a, T> {
    fn fmt(&self, s: &mut String, depth: usize) {
        s.push_str(&format!(
            "{}{{ @ {}\n",
            " ".repeat(depth),
            self.lbrace.start_offset
        ));
        self.data.fmt(s, depth + 1);
        s.push_str(&format!("{}}}\n", " ".repeat(depth)));
    }
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
        .with(kw(EndLine).option())
        .with(maybe(p, expect))
        .with(kw(EndLine).option())
        .with(maybe(kw(TokenKind::RCurly), Expect::RCurly))
        .map(|((((lbrace, endline1), data), endline2), rbrace)| Brace {
            lbrace,
            endline1,
            data,
            endline2,
            rbrace,
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

//const PARAM_LIST_RECOVERY: &[TokenKind] = &[DefKeyword, LCurly];
fn param_list<'a: 'b, 'b>(
    input: &'b [TokenNode<'a>],
) -> Option<(&'b [TokenNode<'a>], Paren<'a, Vec<Param<'a>>>)> {
    paren(param.many0_sep(kw(TokenKind::Comma)), Expect::Param).parse(input)
}

//const STMT_RECOVERY: &[TokenKind] = &[DefKeyword];
//const EXPR_FIRST: &[TokenKind] = &[Num, TrueKeyword, FalseKeyword, Ident, LParen];
fn block<'a: 'b, 'b>(
    input: &'b [TokenNode<'a>],
) -> Option<(&'b [TokenNode<'a>], Brace<'a, Vec<Stmt<'a>>>)> {
    brace(stmt.many0(), Expect::Stmt)
        .with(kw(EndLine).many0())
        .map(|(x, _endline)| x)
        .parse(input)
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
        .with(kw(EndLine).many1())
        .map(|(x, _endline)| Stmt::Val {
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

  val z = x

  val t = z
    .map(x => x + 1)
}

def uncurry[A: U, B: U, C: U](t: (A, B), f: A -> B -> C): C = {
  f(t(0))(t(1))
}
";
    let path = std::path::PathBuf::from("./");
    let cst = parse(text, &path);
    println!("{:#?}", cst);
}

#![feature(pattern)]

use crate::expr::expr;
use crate::lex::lex;
use crate::types::type_param;

pub use combinator::Span;
use combinator::{maybe, AstDebug, Maybe, Parser, PathId, ToSpan};
use expr::Expr;
use lex::TokenNode;

mod class;
pub mod combinator;
mod decl;
pub mod expr;
mod lex;
//pub mod resilient;
pub mod types;

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
    MatchKeyword,
    CaseKeyword,

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
    DoubleArrow,
    Plus,
    Minus,
    Star,
    Slash,
    At,

    Ident,
    Num,
    Op,
    EndLine,
    Str,

    ErrToken,

    Eof,
}

pub type Token<'a> = Span<(&'a str, TokenKind)>;

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
    Case,
    TypeExpr,
    Expr,
    Stmt,
    ArgList,
    Param,
}

#[derive(Clone, Debug)]
pub enum Stmt<T> {
    Return(Span<()>, Expr<T>),
    Val {
        val: Span<()>,
        ident: Maybe<Span<String>, Expect>,
        eq: Maybe<Span<()>, Expect>,
        expr: Maybe<Expr<T>, Expect>,
    },
    Expr(Expr<T>),
}

impl<T: std::fmt::Debug> AstDebug for Stmt<T> {
    fn fmt(&self, s: &mut String, depth: usize) {
        match self {
            Stmt::Return(span, expr) => {
                s.push_str(&format!(
                    "{}return @ {}\n",
                    " ".repeat(depth),
                    span.start_offset
                ));
                expr.fmt(s, depth + 1);
            },
            Stmt::Val { val, ident, eq: _, expr } => {
                s.push_str(&format!(
                    "{}val @ {}\n",
                    " ".repeat(depth),
                    val.start_offset
                ));
                ident.fmt(s, depth + 1);
                s.push_str(&format!(
                    "{}=\n",
                    " ".repeat(depth + 1),
                ));
                expr.fmt(s, depth + 1);
            },
            Stmt::Expr(expr) => {
                s.push_str(&format!(
                    "{}stmt_expr\n",
                    " ".repeat(depth),
                ));
                expr.fmt(s, depth + 1);
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct Fn<T> {
    pub def: Span<()>,
    pub name: Maybe<Span<String>, Expect>,
    pub type_params: Option<TypeParam>,
    pub params: Vec<Paren<Vec<Param>>>,
    pub ret_type: (
        (Span<()>, Maybe<TypeExpr, Expect>),
        Maybe<Span<()>, Expect>,
    ),
    pub body: Expr<T>,
}

impl<T: std::fmt::Debug> AstDebug for Fn<T> {
    fn fmt(&self, s: &mut String, depth: usize) {
        s.push_str(&format!(
            "{}def @ {}\n",
            " ".repeat(depth),
            self.def.start_offset,
        ));
        self.name.fmt(s, depth + 1);
        if let Some(x) = self.type_params.as_ref() {
            x.fmt(s, depth + 1)
        }
        self.params.iter()
            .for_each(|x| x.fmt(s, depth + 1));
        //self.ret_type.0.1.fmt(s, depth + 1);
        self.body.fmt(s, depth + 1);
    }
}

#[derive(Debug, Clone)]
pub struct Paren<T> {
    pub lparen: Span<()>,
    pub data: Maybe<T, Expect>,
    pub rparen: Maybe<Span<()>, Expect>,
}

impl<T: AstDebug> AstDebug for Paren<T> {
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

impl<'a, T> ToSpan<'a> for Paren<T> {
    fn to_span(&self) -> Span<()> {
        self.lparen.to_span() + self.rparen.to_span()
    }
}

#[derive(Debug, Clone)]
pub struct Square<T> {
    pub lbracket: Span<()>,
    pub data: Maybe<T, Expect>,
    pub rbracket: Maybe<Span<()>, Expect>,
}

impl<T: AstDebug> AstDebug for Square<T> {
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

impl<'a, T> ToSpan<'a> for Square<T> {
    fn to_span(&self) -> Span<()> {
        self.lbracket.to_span() + self.rbracket.to_span()
    }
}

#[derive(Clone, Debug)]
pub struct Brace<T> {
    pub lbrace: Span<()>,
    pub endline1: Option<Span<()>>,
    pub data: Maybe<T, Expect>,
    pub endline2: Option<Span<()>>,
    pub rbrace: Maybe<Span<()>, Expect>,
}

impl<T: AstDebug> AstDebug for Brace<T> {
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

impl<'a, T> ToSpan<'a> for Brace<T> {
    fn to_span(&self) -> Span<()> {
        self.lbrace.to_span() + self.rbrace.to_span()
    }
}

fn kw<'a: 'b, 'b>(p: TokenKind) -> impl Parser<&'b [TokenNode<'a>], Span<()>> {
    move |input: &'b [TokenNode<'a>]| match input.first() {
        Some(x) if x.data.1 == p => input.get(1..).map(|i| (i, x.map(|_| ()))),
        _ => None,
    }
}

fn string<'a: 'b, 'b>(p: TokenKind) -> impl Parser<&'b [TokenNode<'a>], Span<String>> {
    move |input: &'b [TokenNode<'a>]| match input.first() {
        Some(x) if x.data.1 == p => input.get(1..).map(|i| (i, x.map(|s| s.0.to_owned()))),
        _ => None,
    }
}

fn paren<'a: 'b, 'b, P, O>(p: P, expect: Expect) -> impl Parser<&'b [TokenNode<'a>], Paren<O>>
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

fn square<'a: 'b, 'b, P, O>(p: P, expect: Expect) -> impl Parser<&'b [TokenNode<'a>], Square<O>>
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

fn brace<'a: 'b, 'b, P, O>(p: P, expect: Expect) -> impl Parser<&'b [TokenNode<'a>], Brace<O>>
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

fn tobox<'a: 'b, 'b, P, O>(p: P) -> impl Parser<&'b [TokenNode<'a>], Box<O>>
where
    P: Parser<&'b [TokenNode<'a>], O>,
{
    p.map(|x| Box::new(x))
}

use types::{param, type_expr, Param, TypeExpr, TypeParam};
use TokenKind::*;

pub fn parse(text: &str, path_id: PathId) -> Vec<Fn<String>> {
    let input = Span {
        data: text,
        start_offset: 0,
        end_offset: text.len() as u32,
        path_id,
    };
    let tokens = lex(input).expect("some unexpected error happens in compiler. lex error");
    if !tokens.0.data.is_empty() {
        panic!("some unexpected error happens in compiler. lex error");
    }
    let ret = file(&tokens.1).expect("some unexpected error happens in compiler. parse error");
    if !ret.0.is_empty() {
        let mut s = String::new();
        for t in ret.1 {
            t.fmt(&mut s, 0);
        }
        println!("{s}");
        println!("{:#?}", ret.0);
        panic!("some unexpected error happens in compiler. parse error")
    }
    ret.1
}

fn file<'a: 'b, 'b>(input: &'b [TokenNode<'a>]) -> Option<(&'b [TokenNode<'a>], Vec<Fn<String>>)> {
    func.many0().parse(input)
}

/// def ident [ type_param ] param_list (: type_expr = | [=] ) block
fn func<'a: 'b, 'b>(input: &'b [TokenNode<'a>]) -> Option<(&'b [TokenNode<'a>], Fn<String>)> {
    kw(DefKeyword)
        .with(maybe(string(Ident), Expect::Ident))
        .with(type_param.option())
        .with(param_list.many0())
        .with(
            kw(Colon)
                .with(maybe(type_expr, Expect::TypeExpr))
                .with(maybe(kw(Eq), Expect::Eq)),
        )
        .with(expr)
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
) -> Option<(&'b [TokenNode<'a>], Paren<Vec<Param>>)> {
    paren(param.many0_sep(kw(TokenKind::Comma)), Expect::Param).parse(input)
}

//const STMT_RECOVERY: &[TokenKind] = &[DefKeyword];
//const EXPR_FIRST: &[TokenKind] = &[Num, TrueKeyword, FalseKeyword, Ident, LParen];
fn block<'a: 'b, 'b>(
    input: &'b [TokenNode<'a>],
) -> Option<(&'b [TokenNode<'a>], Brace<Vec<Stmt<String>>>)> {
    brace(stmt.many0(), Expect::Stmt)
        .with(kw(EndLine).many0())
        .map(|(x, _endline)| x)
        .parse(input)
}

fn stmt<'a: 'b, 'b>(input: &'b [TokenNode<'a>]) -> Option<(&'b [TokenNode<'a>], Stmt<String>)> {
    stmt_val
        .or(stmt_return)
        .or(expr.map(Stmt::Expr))
        .parse(input)
}

fn stmt_val<'a: 'b, 'b>(input: &'b [TokenNode<'a>]) -> Option<(&'b [TokenNode<'a>], Stmt<String>)> {
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
fn stmt_return<'a: 'b, 'b>(input: &'b [TokenNode<'a>]) -> Option<(&'b [TokenNode<'a>], Stmt<String>)> {
    kw(ReturnKeyword)
        .with(expr)
        .map(|x| Stmt::Return(x.0, x.1))
        .parse(input)
}

/// arg_list ::= ( {expr [,]} )
pub fn arg_list<'a: 'b, 'b>(
    input: &'b [TokenNode<'a>],
) -> Option<(&'b [TokenNode<'a>], Paren<Vec<Expr<String>>>)> {
    paren(expr.many0_sep(kw(Comma)), Expect::ArgList).parse(input)
}

#[macro_export]
macro_rules! tester {
    ($p:expr, $input:expr) => {
        let input = Span {
            data: $input,
            start_offset: 0,
            end_offset: $input.len() as u32,
            path_id: 0,
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
    let cst = parse(text, 0);
    println!("{:#?}", cst);
}

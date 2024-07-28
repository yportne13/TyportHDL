#![feature(pattern)]

use std::{cell::Cell, fmt, path::Path};

use crate::expr::expr;
use crate::lex::lex;
use crate::types::type_param;

use combinator::Span;

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

#[derive(Debug, Clone, Copy)]
pub enum TreeKind {
    ErrorTree,
    File,
    Fn,
    TypeExpr,
    ParamList,
    Param,
    Block,
    StmtLet,
    StmtReturn,
    StmtExpr,
    ExprLiteral,
    ExprName,
    ExprParen,
    ExprBinary,
    ExprCall,
    ArgList,
    Arg,
}

pub struct Tree<'a> {
    kind: TreeKind,
    children: Vec<Child<'a>>,
}

enum Child<'a> {
    Token(Token<'a>),
    Tree(Tree<'a>),
}

#[macro_export]
macro_rules! format_to {
    ($buf:expr) => ();
    ($buf:expr, $lit:literal $($arg:tt)*) => {
        { use ::std::fmt::Write as _; let _ = ::std::write!($buf, $lit $($arg)*); }
    };
}

impl<'a> Tree<'a> {
    fn print(&self, buf: &mut String, level: usize) {
        let indent = "  ".repeat(level);
        format_to!(buf, "{indent}{:?}\n", self.kind);
        for child in &self.children {
            match child {
                Child::Token(token) => {
                    format_to!(
                        buf,
                        "{indent}  '{}' @ {}:{}\n",
                        token.data.0,
                        token.start_offset,
                        token.end_offset
                    )
                }
                Child::Tree(tree) => tree.print(buf, level + 1),
            }
        }
        assert!(buf.ends_with('\n'));
    }
}

impl<'a> fmt::Debug for Tree<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buf = String::new();
        self.print(&mut buf, 0);
        write!(f, "{}", buf)
    }
}

#[derive(Debug)]
enum Event {
    Open { kind: TreeKind },
    Close,
    Advance,
}

struct MarkOpened {
    index: usize,
}

struct MarkClosed {
    index: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum ErrKind {
    TokenKind(TokenKind),
    TreeKind(TreeKind),
    Stmt,
    TypeExpr,
}

type PError<'a> = Span<'a, ErrKind>;

struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    pos: usize,
    fuel: Cell<u32>,
    events: Vec<Event>,
    err: Vec<PError<'a>>,
}

impl<'a> Parser<'a> {
    fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens,
            pos: 0,
            fuel: Cell::new(256),
            events: Vec::new(),
            err: vec![],
        }
    }

    fn build_tree(self) -> (Tree<'a>, Vec<PError<'a>>) {
        let mut tokens = self.tokens.into_iter();
        let mut events = self.events;

        assert!(matches!(events.pop(), Some(Event::Close)));
        let mut stack = Vec::new();
        for event in events {
            match event {
                Event::Open { kind } => stack.push(Tree {
                    kind,
                    children: Vec::new(),
                }),
                Event::Close => {
                    let tree = stack.pop().unwrap();
                    stack.last_mut().unwrap().children.push(Child::Tree(tree));
                }
                Event::Advance => {
                    let token = tokens.next().unwrap();
                    stack.last_mut().unwrap().children.push(Child::Token(token));
                }
            }
        }

        let tree = stack.pop().unwrap();
        assert!(stack.is_empty());
        assert!(tokens.next().is_none());
        (tree, self.err)
    }

    fn open(&mut self) -> MarkOpened {
        let mark = MarkOpened {
            index: self.events.len(),
        };
        self.events.push(Event::Open {
            kind: TreeKind::ErrorTree,
        });
        mark
    }

    fn open_before(&mut self, m: MarkClosed) -> MarkOpened {
        let mark = MarkOpened { index: m.index };
        self.events.insert(
            m.index,
            Event::Open {
                kind: TreeKind::ErrorTree,
            },
        );
        mark
    }

    fn close(&mut self, m: MarkOpened, kind: TreeKind) -> MarkClosed {
        self.events[m.index] = Event::Open { kind };
        self.events.push(Event::Close);
        MarkClosed { index: m.index }
    }

    fn advance(&mut self) {
        assert!(!self.eof());
        self.fuel.set(256);
        self.events.push(Event::Advance);
        self.pos += 1;
    }

    fn advance_with_error(&mut self, error: ErrKind) {
        let m = self.open();
        self.err.push(
            self.tokens
                .get(self.pos)
                .unwrap_or_else(|| self.tokens.get(self.pos - 1).unwrap())
                .map(|_| error));
        self.advance();
        self.close(m, ErrorTree);
    }

    fn eof(&self) -> bool {
        self.pos == self.tokens.len()
    }

    fn nth(&self, lookahead: usize) -> TokenKind {
        if self.fuel.get() == 0 {
            panic!("parser is stuck")
        }
        self.fuel.set(self.fuel.get() - 1);
        self.tokens
            .get(self.pos + lookahead)
            .map_or(TokenKind::Eof, |it| it.data.1)
    }

    fn at(&self, kind: TokenKind) -> bool {
        self.nth(0) == kind
    }

    fn at_any(&self, kinds: &[TokenKind]) -> bool {
        kinds.contains(&self.nth(0))
    }

    fn eat(&mut self, kind: TokenKind) -> bool {
        if self.at(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, kind: TokenKind) {
        if self.eat(kind) {
            return;
        }
        self.err.push(
            self.tokens
                .get(self.pos)
                .unwrap_or_else(|| self.tokens.get(self.pos - 1).unwrap())
                .map(|_| ErrKind::TokenKind(kind))
        );
    }
}

use expr::stmt_expr;
use types::type_expr;
use TokenKind::*;
use TreeKind::*;

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
}

const PARAM_LIST_RECOVERY: &[TokenKind] = &[DefKeyword, LCurly];
fn param_list(p: &mut Parser) {
    assert!(p.at(LParen));
    let m = p.open();

    p.expect(LParen);
    while !p.at(RParen) && !p.eof() {
        if p.at(Ident) {
            param(p);
        } else {
            if p.at_any(PARAM_LIST_RECOVERY) {
                break;
            }
            p.advance_with_error(ErrKind::TreeKind(Param));
        }
    }
    p.expect(RParen);

    p.close(m, ParamList);
}

fn param(p: &mut Parser) {
    assert!(p.at(Ident));
    let m = p.open();

    p.expect(Ident);
    p.expect(Colon);
    type_expr(p);
    if !p.at(RParen) {
        p.expect(Comma);
    }

    p.close(m, Param);
}

const STMT_RECOVERY: &[TokenKind] = &[DefKeyword];
const EXPR_FIRST: &[TokenKind] = &[Num, TrueKeyword, FalseKeyword, Ident, LParen];
fn block(p: &mut Parser) {
    assert!(p.at(LCurly));
    let m = p.open();

    p.expect(LCurly);
    while !p.at(RCurly) && !p.eof() {
        match p.nth(0) {
            ValKeyword => stmt_let(p),
            ReturnKeyword => stmt_return(p),
            _ => {
                if p.at_any(EXPR_FIRST) {
                    stmt_expr(p)
                } else {
                    if p.at_any(STMT_RECOVERY) {
                        break;
                    }
                    p.advance_with_error(ErrKind::Stmt);
                }
            }
        }
    }
    p.expect(RCurly);

    p.close(m, Block);
}

fn stmt_let(p: &mut Parser) {
    assert!(p.at(ValKeyword));
    let m = p.open();

    p.expect(ValKeyword);
    p.expect(Ident);
    p.expect(Eq);
    expr(p);

    p.close(m, StmtLet);
}

/// stmt_return ::= return expr
fn stmt_return(p: &mut Parser) {
    assert!(p.at(ReturnKeyword));
    let m = p.open();

    p.expect(ReturnKeyword);
    expr(p);

    p.close(m, StmtReturn);
}

/// arg_list ::= ( {expr [,]} )
fn arg_list(p: &mut Parser) {
    assert!(p.at(LParen));
    let m = p.open();

    p.expect(LParen);
    while !p.at(RParen) && !p.eof() {
        if p.at_any(EXPR_FIRST) {
            arg(p);
        } else {
            break;
        }
    }
    p.expect(RParen);

    p.close(m, ArgList);
}

fn arg(p: &mut Parser) {
    let m = p.open();
    expr(p);
    if !p.at(RParen) {
        p.expect(Comma);
    }
    p.close(m, Arg);
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
    let cst = parse(text, &path);
    eprintln!("{:?}\n\n{:#?}", cst.0, cst.1);
}

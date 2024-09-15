use crate::{
    combinator::{maybe, Maybe, Parser},
    kw,
    lex::TokenNode,
    paren, square, string, Expect, Paren, Span, Square,
    TokenKind::{self, *},
};

const PARAM_LIST_RECOVERY: &[TokenKind] = &[DefKeyword, LCurly, LParen];

pub struct TypeParam<'a> {
    data: Square<'a, Vec<Param<'a>>>,
}

pub fn type_param<'a>(input: &'a [TokenNode<'a>]) -> Option<(&'a [TokenNode<'a>], TypeParam<'a>)> {
    square(param.many0_sep(kw(TokenKind::Comma)), Expect::LSquare)
        .map(|x| TypeParam { data: x })
        .parse(input)
}

pub struct Param<'a> {
    pub name: Span<'a, String>,
    pub colon: Maybe<'a, Span<'a, ()>, Expect>,
    pub ty: Maybe<'a, TypeExpr<'a>, Expect>,
}

pub fn param<'a>(input: &'a [TokenNode<'a>]) -> Option<(&'a [TokenNode<'a>], Param<'a>)> {
    string(TokenKind::Ident)
        .with(maybe(kw(TokenKind::Colon), Expect::Colon))
        .with(maybe(type_expr, Expect::TypeExpr))
        .map(|x| Param {
            name: x.0 .0,
            colon: x.0 .1,
            ty: x.1,
        })
        .parse(input)
}

pub enum TypeExpr<'a> {
    Base(Span<'a, String>),
    Arrow(
        Span<'a, String>,
        Span<'a, ()>,
        Box<Maybe<'a, TypeExpr<'a>, Expect>>,
    ),
    Tuple(Paren<'a, Vec<TypeExpr<'a>>>),
}

fn type_expr<'a>(input: &'a [TokenNode<'a>]) -> Option<(&'a [TokenNode<'a>], TypeExpr<'a>)> {
    let base_or_arrow = string(TokenKind::Ident)
        .with(
            kw(TokenKind::Arrow)
                .with(maybe(type_expr, Expect::TypeExpr))
                .option(),
        )
        .map(|(base, ret)| match ret {
            Some((arrow, ty)) => TypeExpr::Arrow(base, arrow, Box::new(ty)),
            None => TypeExpr::Base(base),
        });
    paren(type_expr.many0_sep(kw(TokenKind::Comma)), Expect::TypeExpr)
        .map(TypeExpr::Tuple)
        .or(base_or_arrow)
        .parse(input)
}

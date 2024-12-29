use crate::{
    combinator::{maybe, AstDebug, Maybe, Parser},
    kw,
    lex::TokenNode,
    paren, square, string, Expect, Paren, Span, Square,
    TokenKind::*,
};

//const PARAM_LIST_RECOVERY: &[TokenKind] = &[DefKeyword, LCurly, LParen];

#[derive(Clone, Debug)]
pub struct TypeParam {
    pub data: Square<Vec<Param>>,
}

impl AstDebug for TypeParam {
    fn fmt(&self, s: &mut String, depth: usize) {
        self.data.fmt(s, depth)
    }
}

pub fn type_param<'a: 'b, 'b>(
    input: &'b [TokenNode<'a>],
) -> Option<(&'b [TokenNode<'a>], TypeParam)> {
    square(param.many0_sep(kw(Comma)), Expect::LSquare)
        .map(|x| TypeParam { data: x })
        .parse(input)
}

#[derive(Clone, Debug)]
pub struct Param {
    pub name: Span<String>,
    pub colon: Maybe<Span<()>, Expect>,
    pub ty: Maybe<TypeExpr, Expect>,
}

impl AstDebug for Param {
    fn fmt(&self, s: &mut String, depth: usize) {
        s.push_str(&format!("{}Param\n", " ".repeat(depth)));
        self.name.data.fmt(s, depth + 1);
        //colon
        s.push_str(&format!("{}{:?}\n", " ".repeat(depth), self.ty));
    }
}

pub fn param<'a: 'b, 'b>(input: &'b [TokenNode<'a>]) -> Option<(&'b [TokenNode<'a>], Param)> {
    string(Ident)
        .with(maybe(kw(Colon), Expect::Colon))
        .with(maybe(type_expr, Expect::TypeExpr))
        .map(|x| Param {
            name: x.0 .0,
            colon: x.0 .1,
            ty: x.1,
        })
        .parse(input)
}

#[derive(Clone, Debug)]
pub enum TypeExpr {
    Base(Span<String>),
    Arrow(
        Span<String>,
        Span<()>,
        Box<Maybe<TypeExpr, Expect>>,
    ),
    Tuple(Paren<Vec<TypeExpr>>),
}

pub fn type_expr<'a: 'b, 'b>(
    input: &'b [TokenNode<'a>],
) -> Option<(&'b [TokenNode<'a>], TypeExpr)> {
    let base_or_arrow = string(Ident)
        .with(kw(Arrow).with(maybe(type_expr, Expect::TypeExpr)).option())
        .map(|(base, ret)| match ret {
            Some((arrow, ty)) => TypeExpr::Arrow(base, arrow, Box::new(ty)),
            None => TypeExpr::Base(base),
        });
    paren(type_expr.many0_sep(kw(Comma)), Expect::TypeExpr)
        .map(TypeExpr::Tuple)
        .or(base_or_arrow)
        .parse(input)
}

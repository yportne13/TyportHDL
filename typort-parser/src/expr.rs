use macro_parser_combinator::*;

use crate::lex::{id, Literal, literal};

#[derive(Debug, Clone)]
pub enum Expression<'a> {
    Literal(Literal),
    Id(Span<&'a str>),
    Add(Box<Expression<'a>>, Box<Expression<'a>>),
    Sub(Box<Expression<'a>>, Box<Expression<'a>>),
    Mul(Box<Expression<'a>>, Box<Expression<'a>>),
    Div(Box<Expression<'a>>, Box<Expression<'a>>),
    Eq(Box<Expression<'a>>, Box<Expression<'a>>),
    Neq(Box<Expression<'a>>, Box<Expression<'a>>),
    If(Box<Expression<'a>>, Vec<Expression<'a>>, Vec<Expression<'a>>),
}

parser! {
    expression: Expression<'a> = (("if" >> expression) * (
            "{" >> {expression} << "}"
        ) * ("else" >>
            "{" >> {expression} << "}"
        )) -> (|((a, b), c)| Expression::If(Box::new(a), b, c))
        | expression3
    expression3: Expression<'a> = (expression2 * ("==" | "!=") * expression)
        -> (|((e1, op), e2)| if op == "==" {
            Expression::Eq(Box::new(e1), Box::new(e2))
        }else {
            Expression::Neq(Box::new(e1), Box::new(e2))
        })
        | expression2
    expression2: Expression<'a> = (expression1 * ("*" | "/") * expression)
        -> (|((e1, op), e2)| if op == "*" {
            Expression::Mul(Box::new(e1), Box::new(e2))
        }else {
            Expression::Div(Box::new(e1), Box::new(e2))
        })
        | expression1
    expression1: Expression<'a> = (expression0 * ("+" | "-") * expression)
        -> (|((e1, op), e2)| if op == "+" {
            Expression::Add(Box::new(e1), Box::new(e2))
        } else {
            Expression::Sub(Box::new(e1), Box::new(e2))
        })
        | expression0
    expression0: Expression<'a> = literal -> (Expression::Literal)
        | id -> (Expression::Id)
}

#[test]
fn test() {
    println!("{:#?}", expression().run("a == bcd"));
    println!("{:#?}", expression().run("a +   bcd * efghi"));
}

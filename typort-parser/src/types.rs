use macro_parser_combinator::*;
use crate::lex::id;

#[derive(Debug, Clone)]
pub struct Type<'a>(Span<&'a str>, Option<Vec<Span<&'a str>>>);

parser! {
    r#type: Type<'a> = (id * ["[" >> {id(",")} << "]"]) -> (|(a, b)| Type(a, b))
}

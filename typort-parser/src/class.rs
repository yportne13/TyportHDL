use macro_parser_combinator::*;

use crate::{lex::id, expr::{expression, Expression}};



parser! {
    app: Vec<Expression<'a>> = "object" >> id >> "extends" >> "App" >> "{" >> {expression} << "}"
}

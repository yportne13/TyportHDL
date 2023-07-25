use macro_parser_combinator::*;

use crate::{types::{r#type, Type}, expr::{expression, Expression}, lex::id};

#[derive(Clone, Debug)]
pub struct TypeParam<'a>(Vec<Span<&'a str>>);

parser! {
    //valdcl: ((Vec<Span<&'a str>>, Option<Type<'a>>), Expression<'a>) = ("val" >> {id(",")} * [":" >> r#type] << "=") * expression
    valdcl: (Span<&'a str>, Expression<'a>) = ("val" >> id << "=") * expression

    //defdcl: () = "def" >> id * ["[" >> {r#type(",")} << "]"]
    //    >> ("(" >>  << ")")

    //type_param_clause: = "[" >> {variant_type_param(",")} << "]"

    //variant_type_param: = {Annotation} ["+" | "-"] TypeParam

    type_param: TypeParam<'a> = {id(",")} -> (TypeParam)//(id | "_") * type_param_clause * [">:" >> r#type] * ["<:" >> r#type] * [":" >> r#type]
}

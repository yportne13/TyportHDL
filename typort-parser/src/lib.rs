mod class;
mod decl;
mod expr;
mod lex;
mod types;

pub mod simple_example {
    use macro_parser_combinator::*;

    pub fn name<'a>() -> Parser!(Span<&'a str>) {
        fn f(input: &str, loc: Location) -> (Option<Span<&str>>, &str, Location) {
            let mut a = input.bytes();
            let mut len = 0;
            if let Some(x) = a.next() {
                if x.is_ascii_alphabetic() {
                    len += 1;
                } else {
                    return (None, input, loc);
                }
            }
            loop {
                match a.next() {
                    Some(x) if x.is_ascii_alphanumeric() => {
                        len += 1;
                    }
                    Some(b'_') => {
                        len += 1;
                    }
                    _ => {
                        break;
                    }
                }
            }
            let ret = Span {
                data: unsafe { input.get_unchecked(..len) },
                offset: loc.offset,
                line: loc.line,
                len,
                path: None,
            };
            let mut loc = loc;
            loc.offset += len;
            loc.col += len;
            (Some(ret), input.get(len..).unwrap_or(""), loc)
        }
        Parser::new(f) << whitespace()
    }

    pub fn int<'a>() -> Parser!(Span<i64>) {
        fn f(input: &str, loc: Location) -> (Option<Span<i64>>, &str, Location) {
            let mut a = input.bytes();
            let mut len = 0;
            if let Some(x) = a.next() {
                if x.is_ascii_digit() || x == b'-' || x == b'+' {
                    len += 1;
                } else {
                    return (None, input, loc);
                }
            }
            loop {
                match a.next() {
                    Some(x) if x.is_ascii_digit() => {
                        len += 1;
                    }
                    _ => {
                        break;
                    }
                }
            }
            let ret = Span {
                data: unsafe { input.get_unchecked(..len) }
                    .parse::<i64>()
                    .unwrap(), //TODO:unwrap may overflow
                offset: loc.offset,
                line: loc.line,
                len,
                path: None,
            };
            let mut loc = loc;
            loc.offset += len;
            loc.col += len;
            (Some(ret), input.get(len..).unwrap_or(""), loc)
        }
        Parser::new(f) << whitespace()
    }

    #[derive(Debug, Clone)]
    pub enum Expression<'a> {
        Int(Span<i64>),
        Bool(bool),
        Name(Span<&'a str>),
        Add(Box<Expression<'a>>, Box<Expression<'a>>),
        Sub(Box<Expression<'a>>, Box<Expression<'a>>),
        Mul(Box<Expression<'a>>, Box<Expression<'a>>),
        Div(Box<Expression<'a>>, Box<Expression<'a>>),
        Eq(Box<Expression<'a>>, Box<Expression<'a>>),
        Neq(Box<Expression<'a>>, Box<Expression<'a>>),
        Call(Span<&'a str>, Vec<Expression<'a>>),
        If(Box<Expression<'a>>, Vec<Stmt<'a>>, Option<Vec<Stmt<'a>>>),
    }

    #[derive(Debug, Clone)]
    pub struct Param<'a>(pub Span<&'a str>, pub Span<&'a str>);

    #[derive(Debug, Clone)]
    pub struct Func<'a> {
        pub name: Span<&'a str>,
        pub params: Vec<Param<'a>>,
        pub return_type: Option<Span<&'a str>>,
        pub block: Vec<Stmt<'a>>,
    }

    #[derive(Debug, Clone)]
    pub enum Stmt<'a> {
        Expr(Expression<'a>),
        Let(Span<&'a str>, Expression<'a>),
        Return(Expression<'a>),
    }

    parser! {

        file: Vec<Func<'a>> = whitespace >> {r#fn}

        r#fn: Func<'a> = ("fn" >> name * param_list * ["->" >> type_expr] * block)
            -> (|(((name, params), return_type), block)| Func {
                name,
                params,
                return_type,
                block,
            })

        param_list: Vec<Param<'a>> = "(" >> {param(",")} << [","] << ")"

        param: Param<'a> = ((name << ":") * type_expr) -> (|(a, b)| Param(a, b))

        type_expr: Span<&'a str> = name

        block: Vec<Stmt<'a>> = "{" >> {stmt} << "}"

        stmt: Stmt<'a> = stmt_let
            | stmt_return
            | stmt_expr

        stmt_expr: Stmt<'a> = expr -> (Stmt::Expr) << ";"

        stmt_let: Stmt<'a> = (("let" >> name << "=") * expr << ";") -> (|(a, b)| Stmt::Let(a, b))

        stmt_return: Stmt<'a> = "return" >> expr -> (Stmt::Return) << ";"

        expr: Expression<'a> = expr_binary

        expr_base1: Expression<'a> = expr_literal
            | expr_if
            | expr_name
            | expr_paren

        expr_if: Expression<'a> = ("if" >> expr * block * ["else" >> block])
            -> (|((cond, a), b)| Expression::If(Box::new(cond), a, b))

        expr_base: Expression<'a> = expr_call

        expr_literal: Expression<'a> = int -> (Expression::Int)
            | "true" -> (|_| Expression::Bool(true))
            | "false" -> (|_| Expression::Bool(false))

        expr_name: Expression<'a> = (name * [arg_list]) -> (|(a, b)| if let Some(args) = b {
                    Expression::Call(a, args)
                }else {
                    Expression::Name(a)
                })

        expr_paren: Expression<'a> = "(" >> expr << ")"

        expr_binary: Expression<'a> = (expr_binary1 * [("+" | "-") * expr])
            -> (|(e1, r)| {
                if let Some((op, e2)) = r {
                    if op == "+" {Expression::Add(Box::new(e1), Box::new(e2))}
                    else {Expression::Sub(Box::new(e1), Box::new(e2))}
                }else {
                    e1
                }
            })

        expr_binary1: Expression<'a> = (expr_binary0 * [("*" | "/") * expr])
            -> (|(e1, r)| {
                if let Some((op, e2)) = r {
                    if op == "*" {Expression::Mul(Box::new(e1), Box::new(e2))}
                    else {Expression::Div(Box::new(e1), Box::new(e2))}
                }else {
                    e1
                }
            })
    
        expr_binary0: Expression<'a> = (expr_base * [("==" | "!=") * expr])
            -> (|(e1, r)| {
                if let Some((op, e2)) = r {
                    if op == "==" {Expression::Eq(Box::new(e1), Box::new(e2))}
                    else {Expression::Neq(Box::new(e1), Box::new(e2))}
                }else {
                    e1
                }
            })

        expr_call: Expression<'a> = expr_base1

        arg_list: Vec<Expression<'a>> = "(" >> {arg(",")} << [","] << ")"

        arg: Expression<'a> = expr

    }

    #[test]
    fn test() {
        let f = file().run(r#"
fn main() {
    let x = 123;
    let y = x + 345;
    println(y);
}
        "#);
        println!("{:#?}", f);
        let f = file().run(r#"
fn recursive_fib(n : i64) -> i64 {
    if n == 0 {
        return 0;
    } else {
        if n == 1 {
            return 1;
        } else {
            return recursive_fib(n - 1) + recursive_fib(n - 2);
        };
    };
}
        "#);
        println!("{:#?}", f);
        let f = stmt().run(r#"let x = 123;"#);
        println!("{:#?}", f);
        let f = stmt().run(r#"let y = x + 345;"#);
        println!("{:#?}", f);
        let f = block().run(r#"{let x = 123;
        let y = x + 345;}"#);
        println!("{:#?}", f);
        let f = expr().run(r#"println(y);"#);
        println!("{:#?}", f);
        let f = expr().run(r#"if n == 0 {return 0;}"#);
        println!("{:#?}", f);
        //let f = expr().run(r#"n == 0"#);
        //println!("{:#?}", f);
        //let f = r#fn().run(r#"fn recursive_fib(n: i64) -> i64 {}"#);
        //println!("{:#?}", f);
    }
}

mod class;
mod decl;
mod expr;
mod lex;
mod types;

pub mod simple_example {
    use macro_parser_combinator::*;

    pub use macro_parser_combinator::Span;

    pub fn name<'a>() -> Parser!(Span<String>) {
        fn f(input: &str, loc: Location) -> (Option<Span<String>>, &str, Location) {
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
                data: unsafe { input.get_unchecked(..len) }.to_owned(),
                offset: loc.offset,
                range: ((loc.line, loc.col), (loc.line, loc.col + len)),
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
            }else {
                return (None, input, loc);
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
                range: ((loc.line, loc.col), (loc.line, loc.col + len)),
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

    pub fn escaped_quoted_span<'a>() -> Parser!(Span<String>) {
        fn f(input: &str, loc: Location) -> (Option<Span<String>>, &str, Location) {
            if let Some(x) = input.strip_prefix('\"') {
                if let Some((a, b)) = x.split_once('\"') {
                    //TODO: check if there is any \n in a
                    let len = a.len() + 2;
                    let ret = Span {
                        data: a.to_owned(),
                        offset: loc.offset,
                        range: ((loc.line, loc.col), (loc.line, loc.col + len)),
                        len,
                        path: None,
                    };
                    let mut loc = loc;
                    loc.offset += len;
                    loc.col += len;
                    (Some(ret), b, loc)
                } else {
                    (None, input, loc)
                }
            }else {
                (None, input, loc)
            }            
        }
        Parser::new(f) << whitespace()
    }


    #[derive(Debug, Clone)]
    pub enum Expression {
        Int(Span<i64>),
        String(Span<String>),
        Bool(bool),
        Name(Span<String>),
        ObjVal(Span<String>, Span<String>),
        Add(Box<Expression>, Box<Expression>),
        Sub(Box<Expression>, Box<Expression>),
        Mul(Box<Expression>, Box<Expression>),
        Div(Box<Expression>, Box<Expression>),
        Eq(Box<Expression>, Box<Expression>),
        Neq(Box<Expression>, Box<Expression>),
        Call(Span<String>, Vec<Expression>),
        ObjCall(Span<String>, Span<String>, Vec<Expression>),
        If(Box<Expression>, Block, Option<Block>),
    }

    #[derive(Debug, Clone)]
    pub struct Func {
        pub name: Span<String>,
        pub params: Vec<(Span<String>, Span<String>)>,
        pub return_type: Option<Span<String>>,
        pub block: Block,
    }

    #[derive(Debug, Clone)]
    pub struct Block(pub Vec<Stmt>);

    #[derive(Debug, Clone)]
    pub struct Object {
        pub name: Span<String>,
        pub extends: Option<Span<String>>,
        pub with: Vec<Span<String>>,
        pub block: Block,
    }

    #[derive(Debug, Clone)]
    pub struct Class {
        pub name: Span<String>,
        pub args: Vec<(Span<String>, Span<String>)>,
        pub extends: Option<Span<String>>,
        pub with: Vec<Span<String>>,
        pub block: Block,
    }

    #[derive(Debug, Clone)]
    pub enum TopItem {
        Class(Class),
        Object(Object),
    }

    #[derive(Debug, Clone)]
    pub enum Stmt {
        Expr(Expression),
        Val(Span<String>, Expression),
        Var(Span<String>, Expression),
        Assign(Span<String>, Expression),
        Return(Expression),
        For(Span<String>, Expression, Expression, Block),
        While(Expression, Block),
        Func(Func),
    }

    parser! {

        file: Vec<TopItem> = whitespace >> {
            object -> (TopItem::Object)
            | class -> (TopItem::Class)
        }

        object: Object = (("object" >> name) * ["extends" >> name] * {"with" >> name} * block)
            -> (|(((name, extends), with), block)| {
                Object {
                    name,
                    extends,
                    with,
                    block,
                }
            })

        class: Class = (("class" >> name) * [param_list] * ["extends" >> name] * {"with" >> name} * block)
            -> (|((((name, args), extends), with), block)| {
                Class {
                    name,
                    args: args.unwrap_or(vec![]),
                    extends,
                    with,
                    block,
                }
            })

        func: Func = ("def" >> name * param_list * [":" >> type_expr << ["="]] * block)
            -> (|(((name, params), return_type), block)| Func {
                name,
                params,
                return_type,
                block,
            })

        param_list: Vec<(Span<String>, Span<String>)> = "(" >> {param(",")} << [","] << ")"

        param: (Span<String>, Span<String>) = (name << ":") * type_expr

        type_expr: Span<String> = name

        block: Block = "{" >> {stmt} -> (Block) << "}"

        stmt: Stmt = stmt_let
            | func -> (Stmt::Func)
            | stmt_return
            | stmt_while
            | stmt_for
            | stmt_assign
            | stmt_expr

        stmt_expr: Stmt = expr -> (Stmt::Expr)

        stmt_let: Stmt = (("val" >> name << "=") * expr) -> (|(a, b)| Stmt::Val(a, b))
            | (("var" >> name << "=") * expr) -> (|(a, b)| Stmt::Var(a, b))

        stmt_assign: Stmt = ((name << "=") * expr) -> (|(a, b)| Stmt::Assign(a, b))

        stmt_return: Stmt = "return" >> expr -> (Stmt::Return)

        stmt_for: Stmt = ("for" >> ("(" >> (name << "<-") * (expr << "until") * (expr << ")")) * block)
            -> (|(((n, from), to), b)| Stmt::For(n, from, to, b))

        stmt_while: Stmt = ("while" >> ("(" >> expr << ")") * block) -> (|(cond, b)| Stmt::While(cond, b))

        expr: Expression = expr_binary

        expr_base1: Expression = expr_literal
            | expr_if
            | expr_name
            | expr_paren

        expr_if: Expression = ("if" >> expr * block * ["else" >> block])
            -> (|((cond, a), b)| Expression::If(Box::new(cond), a, b))

        expr_base: Expression = expr_call

        expr_literal: Expression = int -> (Expression::Int)
            | escaped_quoted_span -> (Expression::String)
            | "true" -> (|_| Expression::Bool(true))
            | "false" -> (|_| Expression::Bool(false))

        expr_obj_item: Expression = ((name << ".") * name * [arg_list]) -> (|((a, b), args)| if let Some(args) = args {
            Expression::ObjCall(a, b, args)
        }else {
            Expression::ObjVal(a, b)
        })

        expr_name: Expression = (name * [arg_list]) -> (|(a, b)| if let Some(args) = b {
                    Expression::Call(a, args)
                }else {
                    Expression::Name(a)
                })

        expr_paren: Expression = "(" >> expr << ")"

        expr_binary: Expression = (expr_binary1 * [("+" | "-") * expr])
            -> (|(e1, r)| {
                if let Some((op, e2)) = r {
                    if op == "+" {Expression::Add(Box::new(e1), Box::new(e2))}
                    else {Expression::Sub(Box::new(e1), Box::new(e2))}
                }else {
                    e1
                }
            })

        expr_binary1: Expression = (expr_binary0 * [("*" | "/") * expr])
            -> (|(e1, r)| {
                if let Some((op, e2)) = r {
                    if op == "*" {Expression::Mul(Box::new(e1), Box::new(e2))}
                    else {Expression::Div(Box::new(e1), Box::new(e2))}
                }else {
                    e1
                }
            })
    
        expr_binary0: Expression = (expr_base * [("==" | "!=") * expr])
            -> (|(e1, r)| {
                if let Some((op, e2)) = r {
                    if op == "==" {Expression::Eq(Box::new(e1), Box::new(e2))}
                    else {Expression::Neq(Box::new(e1), Box::new(e2))}
                }else {
                    e1
                }
            })

        expr_call: Expression = expr_base1

        arg_list: Vec<Expression> = "(" >> {arg(",")} << [","] << ")"

        arg: Expression = expr

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
        return 0
    } else {
        if n == 1 {
            return 1
        } else {
            return recursive_fib(n - 1) + recursive_fib(n - 2)
        }
    }
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
        let f = file().run(r#"
fn testloop(n : i64) -> i64 {
    let x = 0
    let ret = 0
    while(x != n) {
        x = x + 1
        ret = ret + x
    }
    return ret
}
        "#);
        println!("{:#?}", f);
        let f = file().run(r#"
fn testloop(n : i64) -> i64 {
    let ret = 0
    for(x <- 0 until n) {
        ret = ret + x
    }
    return ret
}
        "#);//TODO:when remove last }, infinited loop
        println!("{:#?}", f);
        let f = file().run(r#"
fn main() -> String {
    let ret = "abcd"
    ret
}
        "#);
        println!("{:#?}", f);
        let f = file().run(r#"
fn main() -> Unit {
    let x = Array(1,2,3,4)
    print(x)
}
        "#);
        println!("{:#?}", f);
    }
}

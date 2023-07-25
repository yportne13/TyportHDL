use macro_parser_combinator::*;

#[derive(Debug, Clone)]
pub enum Literal {
    Int(),
    Float(),
    Boolean(bool),
    Character(),
    String(),
    Symbol(),
    Null,
}

parser! {
    literal: Literal =  //["-""] integerLiteral
           //|  ["-""] floatingPointLiteral
           boolean
           //|  characterLiteral
           //|  stringLiteral
           //|  symbolLiteral
           |  "null" -> (|_| Literal::Null)

    boolean: Literal = "true" -> (|_| Literal::Boolean(true))
        | "false" -> (|_| Literal::Boolean(false))

    //character: Literal = "\'" >> 
}

pub fn id<'a>() -> Parser!(Span<&'a str>) {
    fn f(input: &str, loc: Location) -> (Option<Span<&str>>, &str, Location) {
        let mut a = input.chars();
        let mut len = 0;
        if let Some(x) = a.next() {
            if x.is_alphabetic() {
                len += 1;
            }else {
                return (None, input, loc)
            }
        }
        loop {
            match a.next() {
                Some(x) if x.is_alphanumeric() => {len += 1;},
                Some('_') => {len += 1;},
                _ => {break;}
            }
        }
        let ret = Span {
            data: unsafe { input.get_unchecked(..len)},
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

pub fn varid<'a>() -> Parser!(Span<&'a str>) {
    fn f(input: &str, loc: Location) -> (Option<Span<&str>>, &str, Location) {
        let mut a = input.chars();
        let mut len = 0;
        if let Some(x) = a.next() {
            if x.is_ascii_lowercase() {
                len += 1;
            }else {
                return (None, input, loc)
            }
        }
        loop {
            match a.next() {
                Some(x) if x.is_alphanumeric() => {len += 1;},
                Some('_') => {len += 1;},
                _ => {break;}
            }
        }
        let ret = Span {
            data: unsafe { input.get_unchecked(..len)},
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

#[test]
fn test() {
    println!("{:#?}", literal().run("true"));
    assert_eq!("Some(Boolean(true))", format!("{:?}", literal().run("true")));
}

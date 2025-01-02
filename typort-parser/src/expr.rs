use crate::{
    arg_list, block, brace, combinator::{maybe, AstDebug, Maybe, Parser, ToSpan}, kw, lex::TokenNode, paren, string, tobox, Brace, Expect, Paren, Span, Square, Stmt, TokenKind
};
use TokenKind::*;

#[derive(Clone)]
pub enum Expr {
    Bool(Span<bool>),
    Num(Span<i32>),
    Name(Span<String>),
    Paren(Box<Paren<Expr>>),
    Binary(Box<Expr>, Operator, Maybe<Box<Expr>, Expect>),
    If {
        kw: Span<()>,
        cond: Maybe<Paren<Box<Expr>>, Expect>, // a block
        then: Maybe<Box<Expr>, Expect>,        // a block
        els: Option<(Span<()>, Maybe<Box<Expr>, Expect>)>,
    },
    Match {
        kw: Span<()>,
        expr: Maybe<Box<Expr>, Expect>,
        arms: Brace<Vec<MatchCase>>,
    },
    Call(Box<Expr>, Paren<Vec<Expr>>),
    Obj {
        lhs: Box<Expr>,
        endl: Option<Span<()>>,
        dot: Span<()>,
        obj: Span<String>,
    },
    UnnamedFunc {
        param: Span<String>,//TODO: or tuple
        arrow: Span<()>,
        body: Maybe<Box<Expr>, Expect>,
    },
    Tuple(Square<Vec<Expr>>),
    Block(Brace<Vec<Stmt>>),
}

impl AstDebug for Expr {
    fn fmt(&self, s: &mut String, depth: usize) {
        match self {
            Expr::Bool(b) => s.push_str(&format!("{}{:?}\n", " ".repeat(depth), b)),
            Expr::Num(n) => s.push_str(&format!("{}{:?}\n", " ".repeat(depth), n)),
            Expr::Name(n) => s.push_str(&format!("{}{:?}\n", " ".repeat(depth), n)),
            Expr::Paren(p) => p.fmt(s, depth),
            Expr::Binary(l, op, r) => {
                let op = match op {
                    Operator::Add(x) => format!("+ @ {}", x.start_offset),
                    Operator::Sub(x) => format!("- @ {}", x.start_offset),
                    Operator::Mul(x) => format!("* @ {}", x.start_offset),
                    Operator::Div(x) => format!("/ @ {}", x.start_offset),
                    Operator::Op(x) => format!("{:?}", x),
                    Operator::Unknown => "Unknown".to_string(),
                };
                s.push_str(&format!("{}BinaryExpr\n", " ".repeat(depth)));
                l.fmt(s, depth + 1);
                s.push_str(&format!("{}{}\n", " ".repeat(depth + 1), op));
                r.fmt(s, depth + 1);
            }
            Expr::If {
                kw: _,
                cond,
                then,
                els,
            } => {
                s.push_str(&format!("{}If\n", " ".repeat(depth)));
                cond.fmt(s, depth + 1);
                then.fmt(s, depth + 1);
                if let Some(x) = els {
                    x.1.fmt(s, depth + 1);
                }
            }
            Expr::Match { kw, expr, arms } => {
                s.push_str(&format!("{}Match\n", " ".repeat(depth)));
                expr.fmt(s, depth + 1);
                if let Maybe::Some(arms) = &arms.data {
                    for arm in arms {
                        arm.fmt(s, depth + 1);
                    }
                }
            }
            Expr::Call(l, param_list) => {
                s.push_str(&format!("{}Call\n", " ".repeat(depth)));
                l.fmt(s, depth + 1);
                param_list.fmt(s, depth + 1);
            }
            Expr::Obj {
                lhs,
                endl: _,
                dot,
                obj: name,
            } => {
                s.push_str(&format!("{}Object\n", " ".repeat(depth)));
                lhs.fmt(s, depth + 1);
                s.push_str(&format!(
                    "{}. @ {}\n",
                    " ".repeat(depth + 1),
                    dot.start_offset
                ));
                s.push_str(&format!("{}{:?}\n", " ".repeat(depth + 1), name))
            }
            Expr::UnnamedFunc { param, arrow, body } => {
                s.push_str(&format!("{}UnnamedFunc\n", " ".repeat(depth)));
                param.fmt(s, depth + 1);
                //TODO:arrow
                body.fmt(s, depth + 1);
            }
            Expr::Tuple(x) => {
                s.push_str(&format!("{}Tuple\n", " ".repeat(depth)));
                x.fmt(s, depth + 1)
            }
            Expr::Block(b) => {
                s.push_str(&format!("{}Block\n", " ".repeat(depth)));
                b.fmt(s, depth + 1)
            }
        }
    }
}

#[derive(Clone)]
pub struct MatchCase {
    case: Span<()>,
    pat: Pattern,
    cond: Option<(Span<()>, Expr)>,
    arrow: Span<()>,
    expr: Expr,
}

impl AstDebug for MatchCase {
    fn fmt(&self, s: &mut String, depth: usize) {
        s.push_str(&format!("{}MatchCase\n", " ".repeat(depth)));
        s.push_str(&format!("{}. @ {}\n", " ".repeat(depth + 1), self.case.start_offset));
        self.pat.fmt(s, depth + 1);
        if let Some((_, cond)) = &self.cond {
            cond.fmt(s, depth + 1);
        }
    }
}

#[derive(Clone)]
enum Pattern {
    Binding {
        ident: Span<String>, // 绑定的变量名
        at: Span<()>,    // `@` 符号
        pat: Box<Pattern>, // 子模式
    },
    Num(Span<i128>),       // 数字模式
    Ident(Span<String>),     // 标识符模式
}

impl AstDebug for Pattern {
    fn fmt(&self, s: &mut String, depth: usize) {
        match self {
            Pattern::Binding {
                ident,
                at: _,
                pat,
            } => {
                s.push_str(&format!("{}Binding\n", " ".repeat(depth)));
                s.push_str(&format!("{}{:?}\n", " ".repeat(depth + 1), ident));
                pat.fmt(s, depth + 1);
            }
            Pattern::Num(n) => {
                s.push_str(&format!("{}Num\n", " ".repeat(depth)));
                s.push_str(&format!("{}{:?}\n", " ".repeat(depth + 1), n));
            }
            Pattern::Ident(ident) => {
                s.push_str(&format!("{}Ident\n", " ".repeat(depth)));
                s.push_str(&format!("{}{:?}\n", " ".repeat(depth + 1), ident));
            }
        }
    }
}

impl std::fmt::Debug for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut ret = String::new();
        AstDebug::fmt(self, &mut ret, 0);
        write!(f, "{}", ret)
    }
}

impl<'a> ToSpan<'a> for Expr {
    fn to_span(&self) -> Span<()> {
        match self {
            Expr::Bool(span) => span.to_span(),
            Expr::Num(span) => span.to_span(),
            Expr::Name(span) => span.to_span(),
            Expr::Paren(paren) => paren.to_span(),
            Expr::Binary(expr, _, maybe) => expr.to_span() + maybe.to_span(),
            Expr::If {
                kw,
                cond: _,
                then,
                els,
            } => {
                kw.to_span()
                    + if let Some(x) = els {
                        x.1.to_span()
                    } else {
                        then.to_span()
                    }
            }
            Expr::Match {
                kw,
                expr: _,
                arms,
            } => kw.to_span() + arms.to_span(),
            Expr::Call(expr, paren) => expr.to_span() + paren.to_span(),
            Expr::Obj {
                lhs,
                endl: _,
                dot: _,
                obj,
            } => lhs.to_span() + obj.to_span(),
            Expr::UnnamedFunc { param, arrow: _, body } => {
                param.to_span() + body.to_span()
            }
            Expr::Tuple(square) => square.to_span(),
            Expr::Block(brace) => brace.to_span(),
        }
    }
}

/*impl<'a, T: AsRef<Expr<'a>> + 'a> From<T> for Span<()> {
    fn from(value: T) -> Self {
        match value.as_ref() {
            Expr::Bool(span) => span.into(),
            Expr::Num(span) => span.into(),
            Expr::Name(span) => span.into(),
            Expr::Paren(paren) => paren.as_ref().into(),
            Expr::Binary(expr, operator, maybe) => {
                let l: Span<()> = expr.into();
                l + maybe.into()
            },
            Expr::Call(expr, paren) => {
                expr.into()// + paren.into()
            },
            Expr::Obj { lhs, endl, dot, obj } => {
                lhs.into()// + obj.into()
            },
            Expr::Tuple(square) => square.into(),
            Expr::Block(brace) => brace.into(),
        }
    }
}*/

#[derive(Clone, Debug)]
pub enum Operator {
    Add(Span<()>),
    Sub(Span<()>),
    Mul(Span<()>),
    Div(Span<()>),
    Op(Span<String>),
    Unknown,
}

impl Operator {
    pub fn to_level(&self) -> Option<usize> {
        match self {
            Operator::Add(_) | Operator::Sub(_) => Some(0),
            Operator::Mul(_) | Operator::Div(_) => Some(1),
            Operator::Op(_) => Some(2),
            Operator::Unknown => None,
        }
    }
}

pub fn expr<'a: 'b, 'b>(input: &'b [TokenNode<'a>]) -> Option<(&'b [TokenNode<'a>], Expr)> {
    expr_rec(input, None)
}

fn expr_rec<'a: 'b, 'b>(
    input: &'b [TokenNode<'a>],
    left: Option<usize>,
) -> Option<(&'b [TokenNode<'a>], Expr)> {
    let (mut input, mut lhs) = expr_call(input)?;
    while let Some(op) = op(input) {
        if right_binds_tighter(left, op.1.to_level()) {
            input = op.0;
            match expr_rec(input, op.1.to_level()) {
                Some((i, rhs)) => {
                    input = i;
                    lhs = Expr::Binary(Box::new(lhs), op.1, Maybe::Some(Box::new(rhs)));
                }
                None => {
                    lhs = Expr::Binary(
                        Box::new(lhs),
                        op.1,
                        Maybe::Hole(Span {
                            data: Expect::Expr,
                            start_offset: input.last().map(|x| x.start_offset)?,
                            end_offset: input.last().map(|x| x.start_offset)?,
                            path_id: input.last().map(|x| x.path_id)?,
                        }),
                    );
                }
            }
        } else {
            break;
        }
    }
    Some((input, lhs))
}

fn op<'a: 'b, 'b>(input: &'b [TokenNode<'a>]) -> Option<(&'b [TokenNode<'a>], Operator)> {
    kw(Plus)
        .map(Operator::Add)
        .or(kw(Minus).map(Operator::Sub))
        .or(kw(Star).map(Operator::Mul))
        .or(kw(Slash).map(Operator::Div))
        .or(string(Op).map(Operator::Op))
        .parse(input)
}

/// expr_call = expr_call arg_list
///         | expr_call . ident
///         | expr_delimited
fn expr_call<'a: 'b, 'b>(input: &'b [TokenNode<'a>]) -> Option<(&'b [TokenNode<'a>], Expr)> {
    let (mut input, mut lhs) = expr_delimited(input)?;
    loop {
        if let Some((i, rhs)) = arg_list(input) {
            input = i;
            lhs = Expr::Call(Box::new(lhs), rhs);
        } else if let Some((i, rhs)) = kw(EndLine)
            .option()
            .with(kw(Dot))
            .with(string(Ident))
            .parse(input)
        {
            input = i;
            lhs = Expr::Obj {
                lhs: Box::new(lhs),
                endl: rhs.0 .0,
                dot: rhs.0 .1,
                obj: rhs.1,
            };
        } else {
            break;
        }
    }
    Some((input, lhs))
}

fn right_binds_tighter(left: Option<usize>, right: Option<usize>) -> bool {
    let Some(right_tightness) = right else {
        return false;
    };
    let Some(left_tightness) = left else {
        //assert!(left == Eof);
        return true;
    };
    right_tightness > left_tightness
}

fn expr_delimited<'a: 'b, 'b>(input: &'b [TokenNode<'a>]) -> Option<(&'b [TokenNode<'a>], Expr)> {
    kw(TrueKeyword)
        .map(|x| Expr::Bool(x.map(|_| true)))
        .or(kw(FalseKeyword).map(|x| Expr::Bool(x.map(|_| false))))
        .or(kw(IfKeyword)
            .with(maybe(paren(tobox(expr), Expect::Expr), Expect::LParen))
            .with(maybe(tobox(expr), Expect::Expr))
            .with((kw(ElseKeyword).with(maybe(tobox(expr), Expect::Expr))).option())
            .map(|(((kw, cond), then), els)| Expr::If {
                kw,
                cond,
                then,
                els,
            }))
        .or(kw(MatchKeyword).with(maybe(expr.map(Box::new), Expect::Expr)).with(brace(
            kw(CaseKeyword)
                .with(pattern)
                .with((kw(IfKeyword).with(expr)).option())
                .with(kw(DoubleArrow))
                .with(expr)
                .map(|((((case, pat), cond), arrow), expr)| MatchCase {
                    case,
                    pat,
                    cond,
                    arrow,
                    expr,
                }).many0_sep(kw(EndLine)),
            Expect::Case,
        )).map(|((kw, expr), arms)| Expr::Match { kw, expr, arms }))
        .or(string(Num).map(|x| Expr::Num(x.map(|y| y.parse().unwrap()))))
        .or(string(Ident).with(kw(DoubleArrow).with(maybe(tobox(expr), Expect::Expr))).map(|(ident, (arrow, body))|
            Expr::UnnamedFunc { param: ident, arrow, body }
        ))
        .or(string(Ident).map(Expr::Name))
        .or(paren(expr, Expect::Expr).map(|x| Expr::Paren(Box::new(x))))
        .or(block.map(Expr::Block))
        .parse(input)
}

fn pattern<'a: 'b, 'b>(input: &'b [TokenNode<'a>]) -> Option<(&'b [TokenNode<'a>], Pattern)> {
    // 解析绑定模式：`num @ xxxx`
    (string(Ident)
        .with(kw(At))
        .with(pattern)
        .map(|((ident, at), pat)| Pattern::Binding {
            ident,
            at,
            pat: Box::new(pat),
        }))
    // 解析简单模式：数字或标识符
    .or(string(Num).map(|x| Pattern::Num(x.map(|y| y.parse().unwrap()))))
    .or(string(Ident).map(Pattern::Ident))
    .parse(input)
}

#[test]
fn test() {
    use crate::tester;
    tester!(expr, "1");
    tester!(expr, "1 + 2");
    tester!(expr, "1 + 2 * 3");
    tester!(expr, "1 + 2 + 3");
    tester!(expr, "1 + foo.num + 3");
    tester!(expr, "foo.num + list.item.length().div(2)(3) * job()");
    tester!(
        expr,
        "foo.num + list
        .item.length().div(2)(3) * job()"
    );
    //tester(expr, "1 + 2");
}

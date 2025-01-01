use std::{fmt::Debug, ops::Add, str::pattern::Pattern};

pub type PathId = u32;

#[derive(Clone, Copy)]
pub struct Span<T> {
    pub data: T,
    pub start_offset: u32,
    pub end_offset: u32,
    pub path_id: PathId,
}

pub trait ToSpan<'a> {
    fn to_span(&self) -> Span<()>;
}

impl<'a, T: ToSpan<'a>> ToSpan<'a> for Box<T> {
    fn to_span(&self) -> Span<()> {
        self.as_ref().to_span()
    }
}

impl<T: PartialEq> PartialEq for Span<T> {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl<T> Span<T> {
    pub fn map<U>(self, f: impl Fn(T) -> U) -> Span<U> {
        Span {
            data: f(self.data),
            start_offset: self.start_offset,
            end_offset: self.end_offset,
            path_id: self.path_id,
        }
    }
    pub fn len(&self) -> u32 {
        self.end_offset - self.start_offset
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn contains(&self, offset: usize) -> bool {
        offset >= self.start_offset as usize && offset < self.end_offset as usize
    }
    pub fn diagnostic<U: AsRef<str>>(&self, severity: Severity, msg: U) -> Diagnostic {
        Diagnostic {
            severity,
            range: (self.start_offset, self.end_offset),
            path_id: self.path_id,
            msg: msg.as_ref().to_owned(),
            notes: vec![],
        }
    }
    pub fn error<U: AsRef<str>>(&self, msg: U) -> Diagnostic {
        self.diagnostic(Severity::Error, msg)
    }
    pub fn warning<U: AsRef<str>>(&self, msg: U) -> Diagnostic {
        self.diagnostic(Severity::Warning, msg)
    }
    pub fn info<U: AsRef<str>>(&self, msg: U) -> Diagnostic {
        self.diagnostic(Severity::Info, msg)
    }
    pub fn note<U: AsRef<str>>(&self, msg: U) -> (PathId, u32, u32, String) {
        (self.path_id, self.start_offset, self.end_offset, msg.as_ref().to_owned())
    }
}

impl<T: Debug> Debug for Span<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} @ {},{}",
            self.data, self.start_offset, self.end_offset
        )
    }
}

impl Add<Span<()>> for Span<()> {
    type Output = Span<()>;

    fn add(self, rhs: Span<()>) -> Self::Output {
        Span {
            data: (),
            start_offset: self.start_offset,
            end_offset: rhs.end_offset,
            path_id: self.path_id,
        }
    }
}

impl<'a, T> ToSpan<'a> for Span<T> {
    fn to_span(&self) -> Span<()> {
        Span {
            data: (),
            start_offset: self.start_offset,
            end_offset: self.end_offset,
            path_id: self.path_id,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Clone, Debug)]
pub struct Diagnostic {
    severity: Severity,
    range: (u32, u32),
    path_id: PathId,
    msg: String,
    notes: Vec<(PathId, u32, u32, String)>,
}

impl Diagnostic {
    pub fn with_note(mut self, note: (PathId, u32, u32, String)) -> Diagnostic {
        self.notes.push(note);
        self
    }
}

pub trait Parser<I: Copy, A>: Sized + Copy {
    fn parse(&self, input: I) -> Option<(I, A)>;
    fn with<P, B>(self, rhs: P) -> impl Parser<I, (A, B)>
    //T: AsRef<P>,
    where
        P: Parser<I, B>,
    {
        move |input| {
            let (input, a) = self.parse(input)?;
            let (input, b) = rhs.parse(input)?;
            Some((input, (a, b)))
        }
    }
    fn or<P>(self, rhs: P) -> impl Parser<I, A>
    where
        P: Parser<I, A>,
    {
        move |input| self.parse(input).or_else(|| rhs.parse(input))
    }
    fn map<B, F>(self, f: F) -> impl Parser<I, B>
    where
        F: Fn(A) -> B + Copy,
    {
        move |input| {
            let (input, a) = self.parse(input)?;
            Some((input, f(a)))
        }
    }
    fn option(self) -> impl Parser<I, Option<A>> {
        move |input| match self.parse(input) {
            Some((i, a)) => Some((i, Some(a))),
            None => Some((input, None)),
        }
    }
    fn many0(self) -> impl Parser<I, Vec<A>> {
        move |input| {
            let mut input = input;
            let mut result = Vec::new();
            while let Some((input_, a)) = self.parse(input) {
                input = input_;
                result.push(a);
            }
            Some((input, result))
        }
        //self.with(self.many0()).map(|(a, b)| [vec![a], b].concat()).or(&|input| Some((input, vec![])))
    }
    fn many0_sep<P, X>(self, sep: P) -> impl Parser<I, Vec<A>>
    where
        P: Parser<I, X>,
    {
        move |input| {
            let mut input = input;
            let mut result = Vec::new();
            while let Some((input_, a)) = self.parse(input) {
                input = input_;
                result.push(a);
                if let Some((i, _)) = sep.parse(input) {
                    input = i;
                } else {
                    break;
                }
            }
            Some((input, result))
        }
        //self.with(self.many0()).map(|(a, b)| [vec![a], b].concat()).or(&|input| Some((input, vec![])))
    }
    #[allow(unused)]
    fn many1(self) -> impl Parser<I, Vec<A>> {
        move |input| match self.many0().parse(input) {
            Some((_, v)) if v.is_empty() => None,
            x => x,
        }
    }
}

impl<I: Copy, A, F> Parser<I, A> for F
where
    F: Fn(I) -> Option<(I, A)> + Copy,
{
    fn parse(&self, input: I) -> Option<(I, A)> {
        self(input)
    }
}

pub type Input<'a> = Span<&'a str>;

#[derive(Clone, Copy, Debug)]
pub enum Maybe<T, E> {
    Some(T),
    Hole(Span<E>),
}

impl<T: AstDebug, E> AstDebug for Maybe<T, E> {
    fn fmt(&self, s: &mut String, depth: usize) {
        match self {
            Maybe::Some(x) => x.fmt(s, depth),
            Maybe::Hole(x) => {
                s.push_str(&format!("{}Hole @ {}", " ".repeat(depth), x.start_offset))
            }
        }
    }
}

impl<T, E> Maybe<T, E> {
    pub fn map<U, F>(self, mut f: F) -> Maybe<U, E>
    where
        F: FnMut(T) -> U
    {
        match self {
            Maybe::Some(x) => Maybe::Some(f(x)),
            Maybe::Hole(span) => Maybe::Hole(span),
        }
    }
    pub fn and_then<U, F>(self, f: F) -> Maybe<U, E>
    where
        F: FnOnce(T) -> Maybe<U, E>
    {
        match self {
            Maybe::Some(x) => f(x),
            Maybe::Hole(span) => Maybe::Hole(span),
        }
    }
    pub fn unwrap_or_else<F>(self, f: F) -> T
    where
        F: Fn(Span<E>) -> T
    {
        match self {
            Maybe::Some(x) => x,
            Maybe::Hole(span) => f(span),
        }
    }
}

impl<T, E: Debug> Maybe<T, E> {
    pub fn raise_err(self, err: &mut Vec<Diagnostic>) -> Self {
        match &self {
            Maybe::Some(_) => {},
            Maybe::Hole(span) => {
                err.push(span.error(format!("{:?}", span.data)));
            },
        }
        self
    }
}

impl<'a, T: ToSpan<'a>, E> ToSpan<'a> for Maybe<T, E> {
    fn to_span(&self) -> Span<()> {
        match self {
            Maybe::Some(x) => x.to_span(),
            Maybe::Hole(span) => span.to_span(),
        }
    }
}

pub fn maybe<'a: 'b, 'b, T, P, N: 'a + Copy, E: Copy>(
    x: P,
    err: E,
) -> impl Parser<&'b [Span<N>], Maybe<T, E>>
where
    P: Parser<&'b [Span<N>], T>,
{
    move |input| match x.parse(input) {
        Some((input, a)) => Some((input, Maybe::Some(a))),
        None => input
            .last()
            .map(|span| (input, Maybe::Hole(span.map(|_| err)))),
    }
}

pub fn pmatch<'a, P: Pattern + Copy>(pat: P) -> impl Parser<Input<'a>, Span<&'a str>> {
    move |input: Input<'a>| {
        let x = input.data.trim_start_matches(pat);
        if x.len() == input.data.len() {
            None
        } else {
            Some((
                Span {
                    data: x,
                    start_offset: input.start_offset + (input.data.len() - x.len()) as u32,
                    end_offset: input.end_offset,
                    path_id: input.path_id,
                },
                Span {
                    data: &input.data[..(input.data.len() - x.len())],
                    start_offset: input.start_offset,
                    end_offset: input.start_offset + (input.data.len() - x.len()) as u32,
                    path_id: input.path_id,
                },
            ))
        }
    }
}

pub fn is<'a, P: Pattern + Copy>(pat: P) -> impl Parser<Input<'a>, Span<&'a str>> {
    move |input: Input<'a>| {
        input.data.strip_prefix(pat).map(|x| {
            (
                Span {
                    data: x,
                    start_offset: input.start_offset + (input.data.len() - x.len()) as u32,
                    end_offset: input.end_offset,
                    path_id: input.path_id,
                },
                Span {
                    data: &input.data[..(input.data.len() - x.len())],
                    start_offset: input.start_offset,
                    end_offset: input.start_offset + (input.data.len() - x.len()) as u32,
                    path_id: input.path_id,
                },
            )
        })
    }
}

pub trait AstDebug {
    fn fmt(&self, s: &mut String, depth: usize);
}

impl<T: AstDebug> AstDebug for Box<T> {
    fn fmt(&self, s: &mut String, depth: usize) {
        self.as_ref().fmt(s, depth)
    }
}

impl<T: AstDebug> AstDebug for Vec<T> {
    fn fmt(&self, s: &mut String, depth: usize) {
        for x in self {
            x.fmt(s, depth);
        }
    }
}

impl AstDebug for String {
    fn fmt(&self, s: &mut String, depth: usize) {
        s.push_str(&format!("{}{}\n", " ".repeat(depth), self))
    }
}

impl AstDebug for &str {
    fn fmt(&self, s: &mut String, depth: usize) {
        s.push_str(&format!("{}{}\n", " ".repeat(depth), self))
    }
}

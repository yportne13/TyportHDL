use std::{fmt::Debug, path::Path, str::pattern::Pattern};

use super::lex::TokenNode;

#[derive(Clone, Copy)]
pub struct Span<'a, T> {
    pub data: T,
    pub start_offset: u32,
    pub end_offset: u32,
    pub path: &'a Path,
}

impl<'a, T> Span<'a, T> {
    pub fn map<U>(self, f: impl Fn(T) -> U) -> Span<'a, U> {
        Span {
            data: f(self.data),
            start_offset: self.start_offset,
            end_offset: self.end_offset,
            path: self.path,
        }
    }
    pub fn len(&self) -> u32 {
        self.end_offset - self.start_offset
    }
}

impl<'a, T: Debug> Debug for Span<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}:{},{}",
            self.data, self.start_offset, self.end_offset
        )
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

pub type Input<'a> = Span<'a, &'a str>;

#[derive(Clone, Copy, Debug)]
pub enum Maybe<'a, T, E> {
    Some(T),
    Hole(Span<'a, E>),
}

pub fn maybe<'a: 'b, 'b, T, P, E: Copy>(
    x: P,
    err: E,
) -> impl Parser<&'b [TokenNode<'a>], Maybe<'a, T, E>>
where
    P: Parser<&'b [TokenNode<'a>], T>,
{
    move |input| match x.parse(input) {
        Some((input, a)) => Some((input, Maybe::Some(a))),
        None => input
            .last()
            .map(|span| (input, Maybe::Hole(span.map(|_| err)))),
    }
}

pub fn pmatch<'a, P: Pattern + Copy>(pat: P) -> impl Parser<Input<'a>, Span<'a, &'a str>> {
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
                    path: input.path,
                },
                Span {
                    data: &input.data[..(input.data.len() - x.len())],
                    start_offset: input.start_offset,
                    end_offset: input.start_offset + (input.data.len() - x.len()) as u32,
                    path: input.path,
                },
            ))
        }
    }
}

pub fn is<'a, P: Pattern + Copy>(pat: P) -> impl Parser<Input<'a>, Span<'a, &'a str>> {
    move |input: Input<'a>| {
        input.data.strip_prefix(pat).map(|x| {
            (
                Span {
                    data: x,
                    start_offset: input.start_offset + (input.data.len() - x.len()) as u32,
                    end_offset: input.end_offset,
                    path: input.path,
                },
                Span {
                    data: &input.data[..(input.data.len() - x.len())],
                    start_offset: input.start_offset,
                    end_offset: input.start_offset + (input.data.len() - x.len()) as u32,
                    path: input.path,
                },
            )
        })
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct ParserInput<'a> {
    content: &'a str,
    line: usize,
    col: usize,
}

impl<'a> ParserInput<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            content: input,
            line: 0,
            col: 0,
        }
    }
    fn generate_error(self, error_msg: String) -> ParserError {
        let error = format!(
            "Error At: line {}, column {} , \n At: {} \n Reason: {}",
            self.line + 1,
            self.col + 1,
            self.content,
            error_msg
        );
        ParserError { error }
    }
}

impl<'a> From<&'a str> for ParserInput<'a> {
    fn from(value: &'a str) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, PartialEq)]
pub struct ParserSuccess<'a, Output> {
    pub content: Output,
    pub next_input: ParserInput<'a>,
}

#[derive(Debug, PartialEq)]
pub struct ParserError {
    pub error: String,
}

pub type ParserResult<'a, Output> = Result<ParserSuccess<'a, Output>, ParserError>;

pub trait Parser<'a, Output> {
    fn parse(&self, input: ParserInput<'a>) -> ParserResult<'a, Output>;
    fn map<F, B>(self, f: F) -> impl Parser<'a, B>
    where
        F: Fn(Output) -> B;
    fn map_error<F>(self, f: F) -> impl Parser<'a, Output>
    where
        F: Fn(ParserError) -> ParserError;
    fn chain<P1, Output2>(self, p: P1) -> impl Parser<'a, (Output, Output2)>
    where
        P1: Parser<'a, Output2>;
    fn or_else<P1>(self, p: P1) -> impl Parser<'a, Output>
    where
        P1: Parser<'a, Output>;
    fn predicate<Pred>(self, p: Pred, msg: &str) -> impl Parser<'a, Output>
    where
        Pred: Fn(&Output) -> bool;
    fn zero_or_more(self) -> impl Parser<'a, Vec<Output>>;
    fn one_or_more(self) -> impl Parser<'a, Vec<Output>>;
    fn skip_me<P1, Output1>(self, p: P1) -> impl Parser<'a, Output1>
    where
        P1: Parser<'a, Output1>;
    fn skip_next<P1, Output1>(self, p: P1) -> impl Parser<'a, Output>
    where
        P1: Parser<'a, Output1>;
}

impl<'a, Output, P> Parser<'a, Output> for P
where
    P: Fn(ParserInput<'a>) -> ParserResult<'a, Output>,
{
    fn parse(&self, input: ParserInput<'a>) -> ParserResult<'a, Output> {
        self(input)
    }

    fn map<F, B>(self, f: F) -> impl Parser<'a, B>
    where
        F: Fn(Output) -> B,
    {
        move |input| {
            let result = self.parse(input)?;
            Ok(ParserSuccess {
                content: f(result.content),
                next_input: result.next_input,
            })
        }
    }

    fn map_error<F>(self, f: F) -> impl Parser<'a, Output>
    where
        F: Fn(ParserError) -> ParserError,
    {
        move |input| match self.parse(input) {
            Ok(result) => Ok(result),
            Err(err) => Err(f(err)),
        }
    }

    fn chain<P1, Output2>(self, p: P1) -> impl Parser<'a, (Output, Output2)>
    where
        P1: Parser<'a, Output2>,
    {
        move |input| {
            let r1 = self.parse(input)?;
            let r2 = p.parse(r1.next_input)?;
            Ok(ParserSuccess {
                content: (r1.content, r2.content),
                next_input: r2.next_input,
            })
        }
    }

    fn predicate<Pred>(self, p: Pred, msg: &str) -> impl Parser<'a, Output>
    where
        Pred: Fn(&Output) -> bool,
    {
        move |input| {
            let result = self.parse(input)?;
            if p(&result.content) {
                Ok(result)
            } else {
                Err(ParserError { error: msg.into() })
            }
        }
    }

    fn or_else<P1>(self, p: P1) -> impl Parser<'a, Output>
    where
        P1: Parser<'a, Output>,
    {
        move |input| self.parse(input).or(p.parse(input))
    }

    fn zero_or_more(self) -> impl Parser<'a, Vec<Output>> {
        move |mut input| {
            let mut results = Vec::new();
            while let Ok(result) = self.parse(input) {
                input = result.next_input;
                results.push(result.content);
            }
            Ok(ParserSuccess {
                content: results,
                next_input: input,
            })
        }
    }

    fn one_or_more(self) -> impl Parser<'a, Vec<Output>> {
        move |mut input| {
            let mut results = Vec::new();
            let result = self.parse(input)?;
            input = result.next_input;
            results.push(result.content);
            while let Ok(result) = self.parse(input) {
                input = result.next_input;
                results.push(result.content);
            }
            Ok(ParserSuccess {
                content: results,
                next_input: input,
            })
        }
    }

    fn skip_me<P1, Output1>(self, p: P1) -> impl Parser<'a, Output1>
    where
        P1: Parser<'a, Output1>,
    {
        self.chain(p).map(|(_, r)| r)
    }

    fn skip_next<P1, Output1>(self, p: P1) -> impl Parser<'a, Output>
    where
        P1: Parser<'a, Output1>,
    {
        self.chain(p).map(|(r, _)| r)
    }
}

pub fn space0<'a>() -> impl Parser<'a, ()> {
    anychar
        .predicate(|c| c.is_whitespace(), "")
        .zero_or_more()
        .map(|_| ())
}

pub fn space1<'a>() -> impl Parser<'a, ()> {
    anychar
        .predicate(|c| c.is_whitespace(), "")
        .one_or_more()
        .map(|_| ())
}

pub fn between_spaces<'a, P, R>(parser: P) -> impl Parser<'a, R>
where
    P: Parser<'a, R>,
{
    space0().chain(parser).chain(space0()).map(|((_, r), _)| r)
}

fn naked_string<'a>() -> impl Parser<'a, String> {
    anychar
        .predicate(|c| c.is_alphabetic(), "Expected alphabetic character")
        .chain(
            anychar
                .predicate(|c| c.is_alphanumeric(), "Expected alphanumeric character")
                .zero_or_more(),
        )
        .map(|(head, mut tail)| {
            tail.insert(0, head);
            tail.into_iter().collect::<String>()
        })
}

fn quoted_string<'a>() -> impl Parser<'a, String> {
    anychar
        .predicate(|c| *c == '"', "Expected double quote")
        .skip_me(
            anychar
                .predicate(|c| *c != '"', "Expected non empty string")
                .one_or_more(),
        )
        .skip_next(anychar.predicate(|c| *c == '"', "Expected double quote"))
        .map(|v| v.into_iter().collect::<String>())
}

pub fn identifier<'a>() -> impl Parser<'a, String> {
    naked_string().or_else(quoted_string())
}

pub fn natural_number<'a>() -> impl Parser<'a, u32> {
    anychar
        .predicate(|c| c.is_numeric(), "Expected natural number")
        .one_or_more()
        .map(|v| v.into_iter().collect::<String>().parse().unwrap())
}

pub fn real_number<'a>() -> impl Parser<'a, f32> {
    anychar
        .predicate(|c| c.is_numeric() || *c == '.', "Expected number")
        .one_or_more()
        .predicate(
            |v| v.into_iter().collect::<String>().parse::<f32>().is_ok(),
            "Expected real number",
        )
        .map(|v| v.into_iter().collect::<String>().parse().unwrap())
}

fn anychar<'a>(input: ParserInput<'a>) -> ParserResult<'a, char> {
    let mut chars = input.content.chars();
    match chars.next() {
        Some(c) => {
            if c == '\n' {
                Ok(ParserSuccess {
                    content: c,
                    next_input: ParserInput {
                        content: &input.content[c.len_utf8()..],
                        line: input.line + 1,
                        col: 0,
                    },
                })
            } else {
                Ok(ParserSuccess {
                    content: c,
                    next_input: ParserInput {
                        content: &input.content[c.len_utf8()..],
                        line: input.line,
                        col: input.col + 1,
                    },
                })
            }
        }
        None => Err(input.generate_error("Unexpected end of file".to_string())),
    }
}

pub fn literal<'a>(keyword: &'static str) -> impl Parser<'a, &'static str> {
    move |input: ParserInput<'a>| {
        let line_break_position = keyword.rfind(|c| c == '\n').unwrap_or(0);
        let line = keyword.chars().filter(|c| *c != '\n').count();
        let col = keyword.len() - line_break_position;
        if input.content.starts_with(keyword) {
            Ok(ParserSuccess {
                content: keyword,
                next_input: ParserInput {
                    content: &input.content[keyword.len()..],
                    line,
                    col,
                },
            })
        } else {
            Err(input.generate_error(format!("expected keyword {}", keyword)))
        }
    }
}

#[cfg(test)]
mod test {
    use core::panic;

    use crate::parser_combinator::*;

    // #[test]
    // fn t_anychar() {
    //     assert_eq!(None, Some(anychar(ParserInput::new("hello"))));
    // }

    // #[test]
    // fn t_map() {
    //     assert_eq!(None, Some(map(anychar, |c| c as u8).parse("nice".into())))
    // }

    // #[test]
    // fn t_predicate() {
    //     assert_eq!(
    //         None,
    //         Some(predicate(anychar, |c| c.is_alphabetic()).parse("\nice".into()))
    //     )
    // }

    // #[test]
    // fn t_pair() {
    //     let p1 = predicate(anychar, |c| *c == 'h');
    //     let p2 = predicate(anychar, |c| *c == 'e');
    //     assert_eq!(None, Some(pair(p1, p2).parse("hello".into())))
    // }

    // #[test]
    // fn t_literal() {
    //     assert_eq!(None, Some(literal("init").parse("init hello = 12".into())));
    // }

    // #[test]
    // fn t_right() {
    //     assert_eq!(
    //         None,
    //         Some(right(literal("1"), literal("2")).parse("1234".into()))
    //     );
    // }
    // #[test]
    // fn t_left() {
    //     assert_eq!(
    //         None,
    //         Some(left(literal("1"), literal("2")).parse("1234".into()))
    //     );
    // }

    // #[test]
    // fn t_either() {
    //     assert_eq!(
    //         None,
    //         Some(
    //             literal("init")
    //                 .chain(literal("vitesse"))
    //                 .parse("vitesse(E1)=0.4;".into())
    //         )
    //     );
    // }

    // #[test]
    // fn t_zero_or_more() {
    //     assert_eq!(
    //         None,
    //         Some(zero_or_more(literal("1")).parse("112122".into()))
    //     )
    // }
    // #[test]
    // fn t_one_or_more() {
    //     assert_eq!(None, Some(one_or_more(literal("1")).parse("2122".into())))
    // }
    // #[test]
    // fn t_naked_string() {
    //     assert_eq!(None, Some(naked_string().parse("EA31+".into())))
    // }
    // #[test]
    // fn t_quoted_string() {
    //     assert_eq!(
    //         None,
    //         Some(quoted_string().parse("\"EA+  45 ()\":i->p;".into()))
    //     )
    // }
}

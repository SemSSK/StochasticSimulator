#[derive(Debug, PartialEq, Copy, Clone)]
struct ParserInput<'a> {
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
struct ParserSuccess<'a, Output> {
    content: Output,
    next_input: ParserInput<'a>,
}

#[derive(Debug, PartialEq)]
struct ParserError {
    error: String,
}

type ParserResult<'a, Output> = Result<ParserSuccess<'a, Output>, ParserError>;

trait Parser<'a, Output> {
    fn parse(&self, input: ParserInput<'a>) -> ParserResult<'a, Output>;
    fn map<F, B>(self, f: F) -> impl Parser<'a, B>
    where
        F: Fn(Output) -> B;
    fn map_error<F>(self, f: F) -> impl Parser<'a, Output>
    where
        F: Fn(ParserError) -> ParserError;
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
}

fn map<'a, P, F, A, B>(parser: P, f: F) -> impl Parser<'a, B>
where
    P: Parser<'a, A>,
    F: Fn(A) -> B,
{
    move |input| {
        let result = parser.parse(input)?;
        Ok(ParserSuccess {
            content: f(result.content),
            next_input: result.next_input,
        })
    }
}

fn predicate<'a, P, R, Predicate>(parser: P, p: Predicate) -> impl Parser<'a, R>
where
    P: Parser<'a, R>,
    Predicate: Fn(&R) -> bool,
{
    move |input: ParserInput<'a>| {
        let result = parser.parse(input)?;
        if p(&result.content) {
            Ok(result)
        } else {
            Err(input.generate_error("predicate not respected".into()))
        }
    }
}

fn pair<'a, P1, P2, R1, R2>(p1: P1, p2: P2) -> impl Parser<'a, (R1, R2)>
where
    P1: Parser<'a, R1>,
    P2: Parser<'a, R2>,
{
    move |input| {
        p1.parse(input).and_then(|r1| {
            p2.parse(r1.next_input).map(|r2| ParserSuccess {
                content: (r1.content, r2.content),
                next_input: r2.next_input,
            })
        })
    }
}

fn right<'a, P1, P2, R1, R2>(p1: P1, p2: P2) -> impl Parser<'a, R2>
where
    P1: Parser<'a, R1>,
    P2: Parser<'a, R2>,
{
    map(pair(p1, p2), |(r1, r2)| r2)
}

fn left<'a, P1, P2, R1, R2>(p1: P1, p2: P2) -> impl Parser<'a, R1>
where
    P1: Parser<'a, R1>,
    P2: Parser<'a, R2>,
{
    map(pair(p1, p2), |(r1, r2)| r1)
}

fn either<'a, P1, P2, R>(p1: P1, p2: P2) -> impl Parser<'a, R>
where
    P1: Parser<'a, R>,
    P2: Parser<'a, R>,
{
    move |input| match p1.parse(input) {
        Ok(result) => Ok(result),
        Err(err1) => match p2.parse(input) {
            Ok(result) => Ok(result),
            Err(err2) => Err(ParserError {
                error: format!("{} \nand {}", err1.error, err2.error),
            }),
        },
    }
}

fn zero_or_more<'a, P, R>(parser: P) -> impl Parser<'a, Vec<R>>
where
    P: Parser<'a, R>,
{
    move |mut input| {
        let mut results = Vec::new();
        while let Ok(result) = parser.parse(input) {
            input = result.next_input;
            results.push(result.content);
        }
        Ok(ParserSuccess {
            content: results,
            next_input: input,
        })
    }
}

fn one_or_more<'a, P, R>(parser: P) -> impl Parser<'a, Vec<R>>
where
    P: Parser<'a, R>,
{
    move |mut input| {
        let mut results = Vec::new();
        let result = parser.parse(input)?;
        input = result.next_input;
        results.push(result.content);
        while let Ok(result) = parser.parse(input) {
            input = result.next_input;
            results.push(result.content);
        }
        Ok(ParserSuccess {
            content: results,
            next_input: input,
        })
    }
}

fn space0<'a>() -> impl Parser<'a, ()> {
    map(
        zero_or_more(predicate(anychar, |c| c.is_whitespace())),
        |_| (),
    )
}

fn space1<'a>() -> impl Parser<'a, ()> {
    map(
        one_or_more(predicate(anychar, |c| c.is_whitespace())),
        |_| (),
    )
}

fn starts_with_space<'a, P, R>(parser: P) -> impl Parser<'a, R>
where
    P: Parser<'a, R>,
{
    right(space0(), parser)
}

fn naked_string<'a>() -> impl Parser<'a, String> {
    map(
        pair(
            predicate(anychar, |c| c.is_alphabetic()),
            zero_or_more(predicate(anychar, |c| c.is_alphanumeric())),
        ),
        |(head, mut tail)| {
            tail.insert(0, head);
            tail.into_iter().collect::<String>()
        },
    )
}

fn quoted_string<'a>() -> impl Parser<'a, String> {
    map(
        right(
            predicate(anychar, |c| *c == '"'),
            left(
                one_or_more(predicate(anychar, |c| *c != '"')),
                predicate(anychar, |c| *c == '"'),
            ),
        ),
        |cs| cs.into_iter().collect::<String>(),
    )
}

fn identifier<'a>() -> impl Parser<'a, String> {
    either(naked_string(), quoted_string())
}

fn natural_number<'a>() -> impl Parser<'a, u32> {
    one_or_more(predicate(anychar, |c| c.is_numeric()))
        .map(|v| v.into_iter().collect::<String>().parse().unwrap())
        .map_error(|_| ParserError {
            error: "Natural number expected".into(),
        })
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

fn literal<'a>(keyword: &'static str) -> impl Parser<'a, &'static str> {
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

    #[test]
    fn t_either() {
        assert_eq!(
            None,
            Some(either(literal("init"), literal("vitesse")).parse("vitesse(E1)=0.4;".into()))
        );
    }

    #[test]
    fn t_zero_or_more() {
        assert_eq!(
            None,
            Some(zero_or_more(literal("1")).parse("112122".into()))
        )
    }
    #[test]
    fn t_one_or_more() {
        assert_eq!(None, Some(one_or_more(literal("1")).parse("2122".into())))
    }
    #[test]
    fn t_naked_string() {
        assert_eq!(None, Some(naked_string().parse("EA31+".into())))
    }
    #[test]
    fn t_quoted_string() {
        assert_eq!(
            None,
            Some(quoted_string().parse("\"EA+  45 ()\":i->p;".into()))
        )
    }
}

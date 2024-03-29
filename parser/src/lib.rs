use brenda_parser_helpers::*;
use parser_combinator::*;
mod brenda_parser_helpers;
mod parser_combinator;

#[derive(Debug, PartialEq)]
struct SpeedDeclaration {
    identifier: String,
    speed: f32,
}

pub trait Parsable: Sized {
    fn parse<'a>(text: ParserInput<'a>) -> ParserResult<'a, Self>;
}

impl Parsable for SpeedDeclaration {
    fn parse<'a>(text: ParserInput<'a>) -> ParserResult<'a, Self> {
        let ParserSuccess { next_input, .. } = parse_speed().parse(text)?;
        let ParserSuccess { next_input, .. } = parse_lparen().parse(next_input)?;
        let ParserSuccess {
            content: identifier,
            next_input,
        } = parse_identifier().parse(next_input)?;
        let ParserSuccess { next_input, .. } = parse_rparen().parse(next_input)?;
        let ParserSuccess { next_input, .. } = parse_equal().parse(next_input)?;
        let ParserSuccess {
            content: speed,
            next_input,
        } = parse_float().parse(next_input)?;
        let ParserSuccess { next_input, .. } = parse_semicolon().parse(next_input)?;
        Ok(ParserSuccess {
            content: SpeedDeclaration { identifier, speed },
            next_input,
        })
    }
}

#[derive(Debug, PartialEq)]
struct DiameterDeclaration {
    identifier: String,
    diameter: f32,
}

impl Parsable for DiameterDeclaration {
    fn parse<'a>(text: ParserInput<'a>) -> ParserResult<'a, Self> {
        let ParserSuccess { next_input, .. } = parse_diameter().parse(text)?;
        let ParserSuccess { next_input, .. } = parse_lparen().parse(next_input)?;
        let ParserSuccess {
            content: identifier,
            next_input,
        } = parse_identifier().parse(next_input)?;
        let ParserSuccess { next_input, .. } = parse_rparen().parse(next_input)?;
        let ParserSuccess { next_input, .. } = parse_equal().parse(next_input)?;
        let ParserSuccess {
            content: diameter,
            next_input,
        } = parse_float_ranged(0., 1.).parse(next_input)?;
        let ParserSuccess { next_input, .. } = parse_semicolon().parse(next_input)?;
        Ok(ParserSuccess {
            content: DiameterDeclaration {
                identifier,
                diameter,
            },
            next_input,
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct InitDeclaration {
    pub identifier: String,
    pub number: u32,
}

impl Parsable for InitDeclaration {
    fn parse<'a>(text: ParserInput<'a>) -> ParserResult<'a, Self> {
        let ParserSuccess { next_input, .. } = parse_init().parse(text)?;
        let ParserSuccess { next_input, .. } = parse_lparen().parse(next_input)?;
        let ParserSuccess {
            content: identifier,
            next_input,
        } = parse_identifier().parse(next_input)?;
        let ParserSuccess { next_input, .. } = parse_rparen().parse(next_input)?;
        let ParserSuccess { next_input, .. } = parse_equal().parse(next_input)?;
        let ParserSuccess {
            content: number,
            next_input,
        } = parse_uint().parse(next_input)?;
        let ParserSuccess { next_input, .. } = parse_semicolon().parse(next_input)?;
        Ok(ParserSuccess {
            content: Self { identifier, number },
            next_input,
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct Reaction {
    pub enzhym: String,
    pub solubes: String,
    pub results: String,
    pub km: f32,
    pub kcat: f32,
}

impl Parsable for Reaction {
    fn parse<'a>(text: ParserInput<'a>) -> ParserResult<'a, Self> {
        let ParserSuccess {
            content: enzhym,
            next_input,
        } = parse_identifier().parse(text)?;
        let ParserSuccess { next_input, .. } = parse_colon().parse(next_input)?;
        let ParserSuccess {
            content: solubes,
            next_input,
        } = parse_identifier().parse(next_input)?;
        let ParserSuccess { next_input, .. } = parse_arrow().parse(next_input)?;
        let ParserSuccess {
            content: results,
            next_input,
        } = parse_identifier().parse(next_input)?;
        let ParserSuccess { next_input, .. } = parse_pipe().parse(next_input)?;
        let ParserSuccess {
            content: km,
            next_input,
        } = parse_float().parse(next_input)?;
        let ParserSuccess { next_input, .. } = parse_mM().parse(next_input)?;
        let ParserSuccess { next_input, .. } = parse_dash().parse(next_input)?;
        let ParserSuccess {
            content: kcat,
            next_input,
        } = parse_float().parse(next_input)?;
        let ParserSuccess { next_input, .. } = parse_mM().parse(next_input)?;
        let ParserSuccess { next_input, .. } = parse_semicolon().parse(next_input)?;
        Ok(ParserSuccess {
            next_input,
            content: Self {
                enzhym,
                solubes,
                results,
                km,
                kcat,
            },
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct Ast(pub Vec<Expression>);

impl Parsable for Ast {
    fn parse<'a>(text: ParserInput<'a>) -> ParserResult<'a, Self> {
        Expression::parse
            .one_or_more()
            .map(Self)
            .skip_next(parse_eof())
            .parse(text)
    }
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    Reaction(Reaction),
    SpeedDeclaration(SpeedDeclaration),
    InitDeclaration(InitDeclaration),
    DiameterDeclaration(DiameterDeclaration),
}

impl Parsable for Expression {
    fn parse<'a>(text: ParserInput<'a>) -> ParserResult<'a, Self> {
        Reaction::parse(text)
            .map(|c| c.map(Expression::Reaction))
            .or(SpeedDeclaration::parse(text).map(|c| c.map(Expression::SpeedDeclaration)))
            .or(InitDeclaration::parse(text).map(|c| c.map(Expression::InitDeclaration)))
            .or(DiameterDeclaration::parse(text).map(|c| c.map(Expression::DiameterDeclaration)))
    }
}

#[cfg(test)]
mod test {
    use crate::{
        parse_eof, Ast, DiameterDeclaration, Parsable, Parser, ParserError, ParserResult,
        ParserSuccess, SpeedDeclaration,
    };

    #[test]
    fn t_speed() {
        let res: ParserResult<SpeedDeclaration> =
            Parsable::parse("vitesse(    E23 ) =           2.12  ; hello".into());
        assert_eq!(
            Err(ParserError {
                error: "Hello".into()
            }),
            res
        )
    }
    #[test]
    fn t_diameter() {
        assert_eq!(
            Err(ParserError {
                error: "Hello".into()
            }),
            DiameterDeclaration::parse("diametre(    E23 ) =           0.7  ; hello".into())
        )
    }

    #[test]
    fn t_eof() {
        assert_eq!(
            parse_eof().parse("".into()),
            Ok(ParserSuccess {
                content: (),
                next_input: "".into()
            })
        )
    }

    #[test]
    fn t_file() {
        let file = include_str!("./test_input.txt");
        assert_eq!(
            Err(ParserError {
                error: "Hello".into()
            }),
            Ast::parse(file.into())
        )
    }
}

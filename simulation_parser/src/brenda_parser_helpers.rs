use crate::{
    between_spaces, identifier, literal, natural_number, nothing, parser_combinator::Parser,
    real_number,
};

// End of File

pub fn parse_eof<'a>() -> impl Parser<'a, ()> {
    between_spaces(literal("").map(|_| ()))
}

// Separators

pub fn parse_uN<'a>() -> impl Parser<'a, &'static str> {
    between_spaces(literal("uN"))
}
pub fn parse_lparen<'a>() -> impl Parser<'a, &'static str> {
    between_spaces(literal("("))
}

pub fn parse_rparen<'a>() -> impl Parser<'a, &'static str> {
    between_spaces(literal(")"))
}

pub fn parse_colon<'a>() -> impl Parser<'a, &'static str> {
    between_spaces(literal(":"))
}

pub fn parse_arrow<'a>() -> impl Parser<'a, &'static str> {
    between_spaces(literal("->"))
}

pub fn parse_pipe<'a>() -> impl Parser<'a, &'static str> {
    between_spaces(literal("|"))
}

pub fn parse_semicolon<'a>() -> impl Parser<'a, &'static str> {
    between_spaces(literal(";"))
}

pub fn parse_dash<'a>() -> impl Parser<'a, &'static str> {
    between_spaces(literal("-"))
}

pub fn parse_equal<'a>() -> impl Parser<'a, &'static str> {
    between_spaces(literal("="))
}

pub fn parse_plus<'a>() -> impl Parser<'a, &'static str> {
    between_spaces(literal("+"))
}

// Symbols

pub fn parse_identifier<'a>() -> impl Parser<'a, String> {
    between_spaces(identifier())
}

pub fn parse_solubes_and_results<'a>() -> impl Parser<'a, Vec<String>> {
    identifier()
        .chain(parse_plus().skip_me(nothing()))
        .map(|(x, _)| x)
        .zero_or_more()
        .chain(identifier())
        .map(|(mut res, v)| {
            res.push(v);
            res
        })
}

// Keywords
pub fn parse_init<'a>() -> impl Parser<'a, ()> {
    between_spaces(literal("init")).map(|_| ())
}
pub fn parse_speed<'a>() -> impl Parser<'a, ()> {
    between_spaces(literal("vitesse")).map(|_| ())
}
pub fn parse_diameter<'a>() -> impl Parser<'a, ()> {
    between_spaces(literal("diametre")).map(|_| ())
}

// Numbers
pub fn parse_float<'a>() -> impl Parser<'a, f32> {
    between_spaces(real_number())
}

pub fn parse_float_ranged<'a>(a: f32, b: f32) -> impl Parser<'a, f32> {
    parse_float().predicate(move |x| *x >= a && *x <= b, "Value out of range")
}

pub fn parse_uint<'a>() -> impl Parser<'a, u32> {
    between_spaces(natural_number())
}

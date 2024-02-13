use parser_combinator::*;
mod parser_combinator;

#[derive(Debug, PartialEq)]
enum Instruction {
    Reaction(String, Vec<String>, Vec<String>, u32, u32),
    Initialization(String, u32),
    Diameter(String, f32),
    Speed(String, f32),
}

fn parse_initialization<'a>(text: ParserInput<'a>) -> ParserResult<'a, Instruction> {
    let result1 = between_spaces(literal("init"))
        .chain(between_spaces(literal("(")))
        .parse(text)?;
    let result2 = between_spaces(identifier()).parse(result1.next_input)?;
    let result3 = between_spaces(literal(")")).parse(result2.next_input)?;
    let result3 = between_spaces(literal("=")).parse(result3.next_input)?;
    let result4 = between_spaces(natural_number()).parse(result3.next_input)?;
    let result5 = between_spaces(literal(";")).parse(result4.next_input)?;
    Ok(ParserSuccess {
        content: Instruction::Initialization(result2.content, result4.content),
        next_input: result5.next_input,
    })
}

fn parse_reaction<'a>(text: ParserInput<'a>) -> ParserResult<'a, Instruction> {
    let result1 = between_spaces(identifier())
        .chain(between_spaces(literal(":")))
        .map(|(id, _)| id)
        .parse(text)?;
    let result2 = zero_or_more(
        between_spaces(identifier())
            .chain(between_spaces(literal("+")))
            .map(|(id, _)| id),
    )
    .chain(between_spaces(identifier()))
    .map(|(mut init, back)| {
        init.push(back);
        init
    })
    .parse(result1.next_input)?;

    let result3 = between_spaces(literal("->"))
        .chain(zero_or_more(
            between_spaces(identifier())
                .chain(between_spaces(literal("+")))
                .map(|(id, _)| id),
        ))
        .map(|(_, resultants)| resultants)
        .parse(result2.next_input)?;

    let result4 = between_spaces(literal("|"))
        .chain(between_spaces(natural_number()))
        .map(|(_, n)| n)
        .parse(result3.next_input)?;

    let result5 = between_spaces(literal("mM"))
        .chain(between_spaces(literal("-")))
        .chain(between_spaces(natural_number()))
        .map(|(_, n)| n)
        .parse(result4.next_input)?;

    let result6 = between_spaces(literal("mM"))
        .chain(between_spaces(literal(";")))
        .parse(result5.next_input)?;

    Ok(ParserSuccess {
        content: Instruction::Reaction(
            result1.content,
            result2.content,
            result3.content,
            result4.content,
            result5.content,
        ),
        next_input: result6.next_input,
    })
}

#[cfg(test)]
mod test {
    use crate::{parse_initialization, Instruction, ParserError};

    #[test]
    fn t_init() {
        assert_eq!(
            Err(ParserError {
                error: "Hello".into()
            }),
            parse_initialization("init(E1  ) =    30  ;   hello".into())
        )
    }
}

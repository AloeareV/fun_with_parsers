struct HelpOutput {
    name: String,
    description: String,
    arguments: Vec<String>,
    responses: Vec<String>,
    examples: String,
}

trait Parser<'a, T>: Fn(&'a str) -> ParseResult<'a, T> + Copy {
    fn parse(self, input: &'a str) -> ParseResult<'a, T>;
}

type ParseResult<'a, T> = Result<(&'a str, T), &'a str>;

impl<'a, T, F> Parser<'a, T> for F
where
    F: Fn(&'a str) -> ParseResult<'a, T> + Copy,
{
    fn parse(self, input: &'a str) -> ParseResult<'a, T> {
        self(input)
    }
}

mod combinators {
    use super::Parser;
    pub(crate) fn maybe<'a, T, P>(parser: P) -> impl Parser<'a, T>
    where
        P: Parser<'a, T>,
        T: Default,
    {
        move |input| parser.parse(input).or(Ok((input, T::default())))
    }
    pub(crate) fn pair<'a, P1, P2, R1, R2>(
        parser1: P1,
        parser2: P2,
    ) -> impl Parser<'a, (R1, R2)>
    where
        P1: Parser<'a, R1>,
        P2: Parser<'a, R2>,
    {
        move |input| {
            parser1.parse(input).and_then(|(next_input, output1)| {
                parser2
                    .parse(next_input)
                    .map(|(remainder, output2)| (remainder, (output1, output2)))
            })
        }
    }
    pub(crate) fn map<'a, P, F, T, U>(
        parser: P,
        map_fn: F,
    ) -> impl Parser<'a, U>
    where
        P: Parser<'a, T>,
        F: Fn(T) -> U + Copy,
    {
        move |input| {
            parser
                .parse(input)
                .map(|(remainder, output)| (remainder, map_fn(output)))
        }
    }

    //pub(crate) fn
}

fn literal_match_parser<'a, 'b: 'a>(
    expected: &'b str,
) -> impl Parser<'a, &'b str> {
    move |input: &'a str| {
        if input.starts_with(expected) {
            Ok((&input[expected.len()..], expected))
        } else {
            Err(input)
        }
    }
}

fn read_word(input: &str) -> ParseResult<'_, &str> {
    let mut i = 0;
    loop {
        if let Some(c) = input.chars().nth(i) {
            if c.is_ascii_alphabetic() {
                i += 1;
                continue;
            }
        }
        break Ok((&input[i..], &input[..i]));
    }
}

fn split<'a, 'b: 'a>(pattern: &'b str) -> impl Parser<'a, &'a str> {
    move |input: &'a str| {
        let mut i = 0;
        loop {
            break if input[i..].len() < pattern.len() {
                Err(input)
            } else if input[i..].starts_with(pattern) {
                Ok((&input[(i + pattern.len())..], &input[..i]))
            } else {
                i += 1;
                continue;
            };
        }
    }
}

fn rpc_name(input: &str) -> ParseResult<'_, String> {
    combinators::map(
        combinators::pair(
            combinators::maybe(literal_match_parser("z_")),
            read_word,
        ),
        |(a, b)| ([a, b].concat()),
    )(input)
}
fn main() {
    let example1 =
        std::fs::read_to_string("quizface_help/z_getoperationresult.txt")
            .unwrap();
    let example2 =
        std::fs::read_to_string("quizface_help/getaddressbalance.txt").unwrap();
    let parse = combinators::pair(rpc_name, split("Arguments:"));
    dbg!(parse(&example1));
    dbg!(parse(&example2));
}

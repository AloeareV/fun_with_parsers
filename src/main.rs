fn main() {}

fn match_literal<'a>(
    expected: &'static str,
) -> Parser<
    impl Fn(&'a str) -> Result<(&'a str, ()), &'a str> + Copy,
    &'a str,
    (),
    &'a str,
> {
    let f = move |input: &'a str| match input.get(0..expected.len()) {
        Some(next) if next == expected => Ok((&input[expected.len()..], ())),
        _ => Err(input),
    };
    Parser::new(f)
}

use std::marker::PhantomData;

struct Parser<F: Copy, I, O, E>
where
    F: Fn(I) -> Result<(I, O), E>,
{
    parser: F,
    phantom: PhantomData<(I, O, E)>,
}

impl<F: Fn(I) -> Result<(I, O), E> + Copy, I, O, E> Clone
    for Parser<F, I, O, E>
{
    fn clone(&self) -> Self {
        Self::new(self.parser.clone())
    }
}
impl<F: Fn(I) -> Result<(I, O), E> + Copy, I, O, E> Copy
    for Parser<F, I, O, E>
{
}

impl<F: Copy, I, O, E> Parser<F, I, O, E>
where
    F: Fn(I) -> Result<(I, O), E>,
{
    fn new(f: F) -> Self {
        Parser {
            parser: f,
            phantom: PhantomData,
        }
    }

    fn parse(&self, input: I) -> Result<(I, O), E> {
        (self.parser)(input)
    }

    fn pair<F2, O2>(
        self,
        other: F2,
    ) -> Parser<impl Fn(I) -> Result<(I, (O, O2)), E> + Copy, I, (O, O2), E>
    where
        F2: Fn(I) -> Result<(I, O2), E> + Copy,
    {
        Parser::new(move |input| match self.parse(input) {
            Ok((remaining_input, left_output)) => {
                match other(remaining_input) {
                    Ok((final_input, right_output)) => {
                        Ok((final_input, (left_output, right_output)))
                    }
                    Err(err) => Err(err),
                }
            }
            Err(err) => Err(err),
        })
    }

    fn map<M, O2>(
        self,
        map_fn: M,
    ) -> Parser<impl Fn(I) -> Result<(I, O2), E> + Copy, I, O2, E>
    where
        M: Fn(O) -> O2 + Copy,
    {
        Parser::new(move |input| self.parse(input).map(|(i, o)| (i, map_fn(o))))
    }
}

impl<F, I, O, E> Parser<F, I, O, E>
where
    F: Fn(I) -> Result<(I, O), E> + Copy,
    I: Clone,
{
    fn zero_or_more(
        self,
    ) -> Parser<impl Fn(I) -> Result<(I, Vec<O>), E> + Copy, I, Vec<O>, E> {
        Parser::new(move |mut input: I| {
            let mut result = Vec::new();
            while let Ok((remaining_input, next_output)) =
                self.parse(input.clone())
            {
                input = remaining_input;
                result.push(next_output);
            }

            Ok((input, result))
        })
    }

    fn one_or_more(
        self,
    ) -> Parser<impl Fn(I) -> Result<(I, Vec<O>), E> + Copy, I, Vec<O>, E> {
        self.pair(self.zero_or_more().parser).map(|(x, mut xs)| {
            xs.insert(0, x);
            xs
        })
    }
}

fn identifier(input: &str) -> Result<(&str, String), &str> {
    let mut matched = String::new();
    let mut chars = input.chars();

    match chars.next() {
        Some(next) if next.is_alphabetic() => matched.push(next),
        _ => return Err(input),
    }

    while let Some(next) = chars.next() {
        if next.is_alphanumeric() || next == '-' {
            matched.push(next);
        } else {
            break;
        }
    }

    let next_index = matched.len();
    Ok((&input[next_index..], matched))
}

#[cfg(test)]
mod unit {
    use super::*;
    #[test]
    fn literal_parser() {
        let parse_joe = match_literal("Hello Joe!");
        assert_eq!(Ok(("", ())), parse_joe.parse("Hello Joe!"));
        assert_eq!(
            Ok((" Hello Robert!", ())),
            parse_joe.parse("Hello Joe! Hello Robert!")
        );
        assert_eq!(Err("Hello Mike!"), parse_joe.parse("Hello Mike!"));
    }
    #[test]
    fn identifier_parser() {
        let parse_idents = Parser::new(identifier);
        assert_eq!(
            Ok(("", "i-am-an-identifier".to_string())),
            parse_idents.parse("i-am-an-identifier")
        );
        assert_eq!(
            Ok((" entirely an identifier", "not".to_string())),
            parse_idents.parse("not entirely an identifier")
        );
        assert_eq!(
            Err("!not at all an identifier"),
            parse_idents.parse("!not at all an identifier")
        );
    }
    #[test]
    fn pair_combinator() {
        let tag_opener = match_literal("<").pair(identifier);
        assert_eq!(
            Ok(("/>", ((), "my-first-element".to_string()))),
            tag_opener.parse("<my-first-element/>")
        );
        assert_eq!(Err("oops"), tag_opener.parse("oops"));
        assert_eq!(Err("!oops"), tag_opener.parse("<!oops"));
    }
    #[test]
    fn one_or_more_combinator() {
        let parser = match_literal("ha").one_or_more();
        assert_eq!(Ok(("", vec![(), (), ()])), parser.parse("hahaha"));
        assert_eq!(Err("ahah"), parser.parse("ahah"));
        assert_eq!(Err(""), parser.parse(""));
    }

    #[test]
    fn zero_or_more_combinator() {
        let parser = match_literal("ha").zero_or_more();
        assert_eq!(Ok(("", vec![(), (), ()])), parser.parse("hahaha"));
        assert_eq!(Ok(("ahah", vec![])), parser.parse("ahah"));
        assert_eq!(Ok(("", vec![])), parser.parse(""));
    }
}

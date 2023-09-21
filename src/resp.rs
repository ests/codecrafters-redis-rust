//allow unused for the whole file
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::u32;
use nom::character::complete::alpha1;
use nom::character::complete::digit1;
use nom::character::complete::line_ending;
use nom::character::complete::{char, one_of};
use nom::combinator::map_res;
use nom::combinator::opt;
use nom::combinator::recognize;
use nom::bytes::complete::take;
use nom::sequence::delimited;
use nom::sequence::terminated;
use nom::sequence::tuple;
use nom::IResult;

#[allow(unused)]
pub fn parse_error(input: &str) -> IResult<&str, &str> {
    delimited(char('-'), alpha1, line_ending)(input)
}

#[allow(unused)]
pub fn parse_string(input: &str) -> IResult<&str, &str> {
    delimited(char('+'), alpha1, line_ending)(input)
}

#[allow(unused)]
pub fn parse_bulk_string(input: &str) -> IResult<&str, &str> {
    let (rest, length) = delimited(char('$'), u32, line_ending)(input)?;
    terminated(take(length), line_ending)(rest)
}

#[allow(unused)]
pub fn digit(input: &str) -> IResult<&str, i64> {
    map_res(recognize(tuple((opt(one_of("+-")), digit1))), |s: &str| {
        i64::from_str_radix(s, 10)
    })(input)
}

#[allow(unused)]
pub fn parse_integer(input: &str) -> IResult<&str, i64> {
    delimited(char(':'), digit, line_ending)(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_string() {
        let (remaining_input, output) = parse_string("+OK\r\n").unwrap();
        assert_eq!(remaining_input, "");
        assert_eq!(output, "OK");
    }

    #[test]
    fn test_parse_bulk_string() {
        let (remaining_input, output) = parse_bulk_string("$7\r\nabc\r\n89\r\n").unwrap();
        assert_eq!(remaining_input, "");
        assert_eq!(output, "abc\r\n89");
    }

    #[test]
    fn test_parse_integer() {
        let (remaining_input, output) = parse_integer(":100\r\n").unwrap();
        assert_eq!(remaining_input, "");
        assert_eq!(output, 100);
        let (remaining_input, output) = parse_integer(":-100\r\n").unwrap();
        assert_eq!(remaining_input, "");
        assert_eq!(output, -100);
    }
}

use std::borrow::Cow;
use std::rc::Rc;

use nom::{
    branch::alt,
    bytes::complete::take,
    character::complete::{alpha1, char, digit1, line_ending, one_of, u32},
    combinator::{map, map_res, opt, recognize},
    multi::count,
    sequence::{delimited, terminated, tuple},
    IResult,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Type<'a> {
    Integer(i64),
    String(Cow<'a, str>, StrType),
    Array(Rc<Vec<Type<'a>>>),
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum StrType {
    Basic,
    Bulk,
}

// #[allow(unused)]
// pub fn parse_simple_string_or_array(input: &str) -> IResult<&str, Type> {
//     alt((
//         map(parse_string, |s| Type::String(Cow::from(s), StrType::Basic)),
//         parse_array,
//     ))(input)
// }

#[allow(unused)]
pub fn parse_resp(input: &str) -> IResult<&str, Vec<Type>> {
    parse_array(input)
}

#[allow(unused)]
pub fn parse_array(input: &str) -> IResult<&str, Vec<Type>> {
    let (rest, arr_len) = delimited(char('*'), u32, line_ending)(input)?;

    // N times array
    let (rest, obj) = count(
        alt((
            map(parse_string, |s| Type::String(Cow::from(s), StrType::Basic)),
            map(parse_bulk_string, |s| {
                Type::String(Cow::from(s), StrType::Bulk)
            }),
            map(parse_integer, Type::Integer),
        )),
        arr_len as usize,
    )(rest)?;

    Ok((rest, obj))
}

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
fn digit(input: &str) -> IResult<&str, i64> {
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
    use Type::*;

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

    #[test]
    fn test_parse_array_of_integers() {
        let (remaining_input, output) = parse_array("*3\r\n:1\r\n:2\r\n:3\r\n").unwrap();
        assert_eq!(remaining_input, "");
        assert_eq!(output, vec![Integer(1), Integer(2), Integer(3)]);
    }

    #[test]
    fn test_parse_resp_echo() {
        let (remaining_input, output) = parse_array("*2\r\n$4\r\nECHO\r\n$3\r\nhey\r\n").unwrap();
        assert_eq!(remaining_input, "");
        assert_eq!(
            output,
            vec![
                Type::String(Cow::from("ECHO"), StrType::Bulk),
                Type::String(Cow::from("hey"), StrType::Bulk)
            ]
        );
    }
}

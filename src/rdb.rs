use std::io::Cursor;
use std::{error::Error, fs::File, io::Read, path::Path};

use crate::{Duration, State};
use nom::bytes::complete::{take, take_while};
use nom::number::complete::be_u8;
use nom::{bytes::complete::tag, combinator::map_res, IResult};

pub fn load_from_rdb(path: &Path, state: State, durations: Duration) -> Result<(), Box<dyn Error>> {
    let mut file = File::open(path)?;
    let mut buf: Vec<u8> = Vec::new();
    file.read_to_end(&mut buf)?;
    let cursor = Cursor::new(buf);

    let (_, (_, version)) = parse_rdb_header(&cursor.get_ref()[0..9]).unwrap();
    let (rest, (hash_size, expiry_size)) = parse_resize_db(&cursor.get_ref()[9..]).unwrap();

    dbg!(&hash_size);
    dbg!(&expiry_size);
    Ok(())
}

fn parse_rdb_header(input: &[u8]) -> IResult<&[u8], (&str, u32)> {
    let (rest, _) = tag(b"REDIS")(input)?;
    let (rest, version_str) = map_res(take(4usize), std::str::from_utf8)(rest)?;
    let version = u32::from_str_radix(version_str, 10).unwrap();

    Ok((rest, ("REDIS", version)))
}

fn parse_resize_db(input: &[u8]) -> IResult<&[u8], (usize, usize)> {
    let (input, _) = take_while(|b| b != 0xFB)(input)?;
    let (input, _) = be_u8(input)?; // Consume the FB byte
    let (input, hash_size) = parse_length_encoded_integer(input)?;
    let (input, expire_size) = parse_length_encoded_integer(input)?;
    Ok((input, (hash_size, expire_size)))
}

fn parse_length_encoded_integer(input: &[u8]) -> IResult<&[u8], usize> {
    let (input, first_byte) = be_u8(input)?;
    match first_byte >> 6 {
        00 => Ok((input, (first_byte & 0b00111111) as usize)),
        01 => {
            let (input, second_byte) = be_u8(input)?;
            Ok((
                input,
                (((first_byte & 0b00111111) as usize) << 8) | second_byte as usize,
            ))
        }
        _ => todo!("not implemented"),
    }
}

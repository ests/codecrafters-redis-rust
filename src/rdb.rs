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

    let (_, (_, _version)) = parse_rdb_header(&cursor.get_ref()[0..9]).unwrap();
    let (rest, (hash_size, _expiry_size)) = parse_resize_db(&cursor.get_ref()[9..]).unwrap();

    for _ in 0..hash_size {
        let (rest, (key, value)) = parse_key_value_pair(rest).unwrap();
        dbg!(&key);
        dbg!(&value);
        // insert into state
    }

    Ok(())
}

fn parse_key_value_pair(input: &[u8]) -> IResult<&[u8], (&str, &str)> {
    let (rest, value_type) = be_u8(input)?;
    if value_type == 0x00 {
        let (rest, first_byte) = be_u8(rest)?;
        let (rest, key) = match first_byte >> 6 {
            00 => {
                let length = (first_byte & 0b00111111) as usize;
                let (rest, string) = take(length)(rest)?;
                (rest, std::str::from_utf8(string).unwrap_or("NULL"))
            }
            // 01 => {
            //     let (input, second_byte) = be_u8(input)?;
            //     Ok((
            //         input,
            //         (((first_byte & 0b00111111) as usize) << 8) | second_byte as usize,
            //     ))
            // }
            _ => {
                todo!("unimplemented")
            }
        };
        let (rest, second_byte) = be_u8(rest)?;
        let (rest, value) = match second_byte >> 6 {
            00 => {
                let length = (second_byte & 0b00111111) as usize;
                let (rest, string) = take(length)(rest)?;
                (rest, std::str::from_utf8(string).unwrap_or("NULL"))
            }
            // 01 => {
            //     let (input, second_byte) = be_u8(input)?;
            //     Ok((
            //         input,
            //         (((first_byte & 0b00111111) as usize) << 8) | second_byte as usize,
            //     ))
            // }
            _ => {
                todo!("unimplemented")
            }
        };
        Ok((rest, (key, value)))
    } else {
        todo!("not implemented");
    }
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

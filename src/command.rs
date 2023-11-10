use std::borrow::Cow;

use crate::resp::Type as RespType;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Command {
    Ping,
    Echo(String),
    Set(String, String, Option<u64>),
    Get(String),
}

impl<'a> TryFrom<Vec<RespType<'a>>> for Command {
    type Error = &'static str;

    fn try_from(t: Vec<RespType<'a>>) -> Result<Self, Self::Error> {
        let mut iter = t.into_iter();

        let cmd = iter.next();

        if let Some(RespType::String(a, _)) = cmd {
            match a.as_ref() {
                "ping" => return Ok(Command::Ping),
                "echo" => {
                    let params = iter.next();
                    match params {
                        Some(RespType::String(key, _)) => {
                            return Ok(Command::Echo(key.to_string()));
                        }
                        _ => return Err("Invalid echo command format"),
                    }
                }
                "get" => {
                    let params = iter.next();
                    match params {
                        Some(RespType::String(key, _)) => {
                            return Ok(Command::Get(key.to_string()));
                        }
                        _ => return Err("Invalid get command format"),
                    }
                }
                "set" => {
                    let params = (iter.next(), iter.next(), iter.next(), iter.next());
                    match params {
                        (
                            Some(RespType::String(key, _)),
                            Some(RespType::String(val, _)),
                            None,
                            None,
                        ) => {
                            return Ok(Command::Set(key.to_string(), val.to_string(), None));
                        }
                        (
                            Some(RespType::String(key, _)),
                            Some(RespType::String(val, _)),
                            Some(RespType::String(Cow::Borrowed("px"), _)),
                            Some(RespType::String(i, _)),
                        ) => {
                            let px = u64::from_str_radix(i.as_ref(), 10).unwrap_or(0);
                            return Ok(Command::Set(key.to_string(), val.to_string(), Some(px)));
                        }
                        _ => return Err("Invalid set command format"),
                    }
                }
                _ => return Err("Unrecognized command"),
            }
        } else {
            return Err("Invalid command");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resp::StrType;
    use std::borrow::Cow;

    #[test]
    fn test_set_invalid() {
        let resp = vec![
            RespType::String(Cow::from("set"), StrType::Bulk),
            RespType::String(Cow::from("test_string"), StrType::Bulk),
        ];
        let command = Command::try_from(resp);
        assert!(command.is_err());
    }

    #[test]
    fn test_set() {
        let resp = vec![
            RespType::String(Cow::from("set"), StrType::Bulk),
            RespType::String(Cow::from("test_string"), StrType::Bulk),
            RespType::String(Cow::from("test_value"), StrType::Bulk),
        ];
        let command = Command::try_from(resp);
        assert!(command.is_ok());
        assert_eq!(
            command.unwrap(),
            Command::Set("test_string".to_string(), "test_value".to_string(), None)
        );
    }

    #[test]
    fn test_set_with_px() {
        let resp = vec![
            RespType::String(Cow::from("set"), StrType::Bulk),
            RespType::String(Cow::from("test_string"), StrType::Bulk),
            RespType::String(Cow::from("test_value"), StrType::Bulk),
            RespType::String(Cow::from("px"), StrType::Bulk),
            RespType::String(Cow::from("142"), StrType::Bulk),
        ];
        let command = Command::try_from(resp);
        assert!(command.is_ok());
        assert_eq!(
            command.unwrap(),
            Command::Set(
                "test_string".to_string(),
                "test_value".to_string(),
                Some(142u64)
            )
        );
    }
}

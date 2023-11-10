pub enum Reply<'a> {
    Simple(String),
    Error(&'a str),
    Pong,
    Echo(String),
    Null,
    NullBulk,
    Bulk(String),
    // TODO: for now it only supports bulk strings
    Array(Vec<String>),
}

impl<'a> Reply<'a> {
    pub fn into_bytes(self) -> Vec<u8> {
        match self {
            Reply::Simple(s) => format!("+{}\r\n", s).into_bytes(),
            Reply::Pong => Reply::Simple("PONG".to_string()).into_bytes(),
            Reply::Echo(s) => Reply::Simple(s).into_bytes(),
            Reply::Error(msg) => format!("-ERR {}\r\n", msg).into_bytes(),
            Reply::Null => String::from("_\r\n").into_bytes(),
            Reply::NullBulk => String::from("$-1\r\n").into_bytes(),
            Reply::Bulk(s) => format!("${}\r\n{}\r\n", s.len(), s).into_bytes(),
            Reply::Array(v) => {
                let mut resp = format!("*{}\r\n", v.len());
                for s in v {
                    resp.push_str(&format!("${}\r\n{}\r\n", s.len(), s));
                }
                resp.into_bytes()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_reply() {
        let expected = b"_\r\n";
        assert_eq!(expected.to_vec(), Reply::Null.into_bytes());
    }

    #[test]
    fn test_err_reply() {
        let expected = b"-ERR you fooed\r\n";
        assert_eq!(expected.to_vec(), Reply::Error("you fooed").into_bytes());
    }

    #[test]
    fn test_bulk_array() {
        let expected = b"*2\r\n$3\r\ndir\r\n$16\r\n/tmp/redis-files\r\n";
        assert_eq!(
            expected.to_vec(),
            Reply::Array(vec!["dir".to_string(), "/tmp/redis-files".to_string()]).into_bytes()
        );
    }
}

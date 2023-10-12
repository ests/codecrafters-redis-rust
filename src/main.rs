mod resp;

use redis_starter_rust::ThreadPool;
use resp::{StrType, Type};
use std::{io::Read, io::Write, net::TcpListener};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    let pool = ThreadPool::new(2);

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                println!("accepted new connection");

                pool.execute(|| {
                    println!("pool.execute");
                    handle_client(s).unwrap();
                })
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_client<T: Write + Read>(mut stream: T) -> std::io::Result<()> {
    loop {
        let mut buf: [u8; 64] = [0; 64];
        let bytes_read = stream.read(&mut buf)?;
        if bytes_read == 0 {
            break;
        }

        let (_, resp_cmd): (&str, Vec<resp::Type>) =
            resp::parse_resp(std::str::from_utf8(&buf).unwrap()).unwrap();

        let mut reply: Option<Vec<u8>> = None;

        if let Type::String(cmd, StrType::Bulk) = &resp_cmd[0] {
            match cmd.clone().to_owned().to_lowercase().as_ref() {
                "echo" => {
                    if let Type::String(attr, StrType::Bulk) = &resp_cmd[1] {
                        reply = Some(format!("+{}\r\n", attr.clone().as_ref()).into_bytes());
                    }
                }
                "ping" => {
                    reply = Some(b"+PONG\r\n".to_vec());
                }
                _ => {
                    eprintln!("Unsupported command");
                }
            }
        }

        if let Some(b) = reply {
            stream.write(&b)?;
        } else {
            stream.write(b"+OK\r\n")?;
        }
    }

    Ok(())
}

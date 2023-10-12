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

        let (_, resp_type): (&str, Vec<resp::Type>) =
            resp::parse_resp(std::str::from_utf8(&buf).unwrap()).unwrap();

        let resp_cmd = &resp_type[..];

        match resp_cmd {
            [Type::String(cmd, StrType::Bulk), Type::String(attr, StrType::Bulk)] => {
                match cmd.to_lowercase().as_ref() {
                    "echo" => {
                        let mut reply = String::from("+");
                        reply.push_str(attr.clone().as_ref());
                        reply.push_str("\r\n");
                        let _ = stream.write(reply.as_bytes())?;
                    }
                    _ => {
                        eprintln!("Unsupported command");
                        let _ = stream.write(b"+OK\r\n")?;
                    }
                }
            }
            _ => {
                eprintln!("Unsupported command");
                let _ = stream.write(b"+OK\r\n")?;
            }
        }
    }

    Ok(())
}

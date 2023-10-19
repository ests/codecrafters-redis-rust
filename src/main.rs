#![allow(unused)]

mod resp;

use redis_starter_rust::ThreadPool;
use resp::{StrType, Type};
use std::time;
use std::{io::Read, io::Write, net::TcpListener};

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

type State = Arc<Mutex<HashMap<String, (String, Option<time::Instant>)>>>;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    let pool = ThreadPool::new(2);

    let state: State = Arc::new(Mutex::new(HashMap::new()));

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                println!("accepted new connection");

                let state_clone = Arc::clone(&state);
                pool.execute(|| {
                    println!("pool.execute");
                    handle_client(s, state_clone).unwrap();
                })
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_client<T: Write + Read>(mut stream: T, state: State) -> std::io::Result<()> {
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
                "get" => {
                    if resp_cmd.len() == 2 {
                        if let Type::String(key, StrType::Bulk) = &resp_cmd[1] {
                            let mut state = state.lock().unwrap();
                            let val = state.get(key.as_ref());
                            if let Some((val, dur)) = val {
                                reply = Some(format!("${}\r\n{}\r\n", val.len(), val).into_bytes());
                                if dur.is_some() {
                                    if dur.unwrap().checked_duration_since(time::Instant::now()).is_none() {
                                        // remove
                                        let _ = state.remove(key.as_ref());
                                        reply = Some(b"-1\r\n".to_vec());
                                    }
                                } 
                            }
                        }
                    }
                }
                "set" => {
                    if resp_cmd.len() >= 3 {
                        if let [Type::String(key, StrType::Bulk), Type::String(val, StrType::Bulk)] =
                            &resp_cmd[1..3]
                        {
                            let mut duration = None;

                            if resp_cmd.len() == 5 {
                                if let [Type::String(px, StrType::Bulk), Type::String(dur_str, StrType::Bulk)] =
                                    &resp_cmd[3..5]
                                {
                                    if px.to_lowercase() == "px" {
                                        let now = time::Instant::now();
                                        if let Ok(ms) = u64::from_str_radix(dur_str, 10) {
                                            let dur = time::Duration::from_millis(ms);
                                            duration = Some(now + dur);
                                        } else {
                                            println!("Failed to convert string to duration.");
                                        }
                                    }
                                }
                            }

                            let mut s = state.lock().unwrap();
                            s.insert(key.clone().into_owned(), (val.clone().into_owned(), duration));
                            reply = Some(b"+OK\r\n".to_vec());
                        }
                    }
                }
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
            // Null reply
            stream.write(b"_\r\n")?;
        }
    }

    Ok(())
}

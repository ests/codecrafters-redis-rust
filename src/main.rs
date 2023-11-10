mod command;
mod resp;

use redis_starter_rust::ThreadPool;
use std::time;
use std::{io::Read, io::Write, net::TcpListener};

use command::Command;
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

        let mut reply: Option<Vec<u8>> = None;

        let (_, resp_cmd): (&str, Vec<resp::Type>) =
            resp::parse_resp(std::str::from_utf8(&buf).unwrap()).unwrap();
        let command = Command::try_from(resp_cmd);
        if command.is_err() {
            let emsg = command.unwrap_err();
            reply = Some(format!("-ERR {}\r\n", emsg).as_bytes().to_vec());
        } else {
            match command.unwrap() {
                Command::Set(key, val, px) => {
                    let duration = if let Some(dur) = px {
                        Some(time::Instant::now() + time::Duration::from_millis(dur))
                    } else {
                        None
                    };
                    let mut s = state.lock().unwrap();
                    s.insert(key, (val, duration));
                    reply = Some(b"+OK\r\n".to_vec());
                }
                Command::Get(key) => {
                    let mut state = state.lock().unwrap();
                    if let Some((val, dur)) = state.get(&key) {
                        if dur.is_some()
                            && dur
                                .unwrap()
                                .checked_duration_since(time::Instant::now())
                                .is_none()
                        {
                            let _ = state.remove(&key);
                            reply = Some(b"$-1\r\n".to_vec());
                        } else {
                            reply = Some(format!("${}\r\n{}\r\n", val.len(), val).into_bytes());
                        }
                    }
                }
                Command::Ping => reply = Some(b"+PONG\r\n".to_vec()),
                Command::Echo(s) => {
                    reply = Some(format!("+{}\r\n", s).into_bytes());
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

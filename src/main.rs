mod command;
mod reply;
mod resp;

use redis_starter_rust::ThreadPool;
use reply::Reply;
use std::time;
use std::{io::Read, io::Write, net::TcpListener};

use command::Command;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

type State = Arc<Mutex<HashMap<String, String>>>;
type Config = Arc<HashMap<String, String>>;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    let pool = ThreadPool::new(2);

    let state: State = Arc::new(Mutex::new(HashMap::new()));

    let args: Vec<String> = std::env::args().collect();
    let mut arg_pairs = HashMap::new();
    let mut args_iter = args.iter().skip(1);
    while let Some(arg) = args_iter.next() {
        match arg.as_str() {
            "--dir" => {
                arg_pairs.insert("dir".to_owned(), args_iter.next().cloned().unwrap());
            }
            "--dbfilename" => {
                arg_pairs.insert("dbfilename".to_owned(), args_iter.next().cloned().unwrap());
            }
            _ => {}
        }
    }
    let shared_args: Config = Arc::new(arg_pairs);

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                let state_clone = Arc::clone(&state);
                let config = Arc::clone(&shared_args);
                pool.execute(|| {
                    handle_client(s, state_clone, config).unwrap();
                })
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_client<T: Write + Read>(mut stream: T, state: State, config: Config) -> std::io::Result<()> {
    loop {
        let mut buf: [u8; 64] = [0; 64];
        let bytes_read = stream.read(&mut buf)?;
        if bytes_read == 0 {
            break;
        }

        let mut reply: Option<Reply> = None;

        let (_, resp_cmd): (&str, Vec<resp::Type>) =
            resp::parse_resp(std::str::from_utf8(&buf).unwrap()).unwrap();

        let command = Command::try_from(resp_cmd);
        if command.is_err() {
            let emsg: &str = command.unwrap_err();
            reply = Some(Reply::Error(emsg));
        } else {
            match command.unwrap() {
                Command::ConfigGet(key) => {
                    if let Some(val) = config.get(&key) {
                        reply = Some(Reply::Array(vec![key, val.to_owned()]));
                    }
                }
                Command::Set(key, val, px) => {
                    let duration = if let Some(dur) = px {
                        Some(time::Instant::now() + time::Duration::from_millis(dur))
                    } else {
                        None
                    };
                    let mut s = state.lock().unwrap();
                    s.insert(key, val);
                    // TODO: handle duration
                    reply = Some(Reply::Simple("OK".to_string()));
                }
                Command::Get(key) => {
                    let mut state = state.lock().unwrap();
                    if let Some(val) = state.get(&key) {
                        // if dur.is_some()
                        //     && dur
                        //         .unwrap()
                        //         .checked_duration_since(time::Instant::now())
                        //         .is_none()
                        // {
                        //     let _ = state.remove(&key);
                        //     reply = Some(Reply::NullBulk);
                        // } else {
                        reply = Some(Reply::Bulk(val.to_owned()));
                        // }
                    }
                }
                Command::Ping => reply = Some(Reply::Pong),
                Command::Echo(s) => {
                    reply = Some(Reply::Echo(s));
                }
            }
        }

        if let Some(b) = reply {
            stream.write(&b.into_bytes())?;
        } else {
            stream.write(&Reply::Null.into_bytes())?;
        }
    }

    Ok(())
}

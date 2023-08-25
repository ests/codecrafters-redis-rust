use redis_starter_rust::ThreadPool;
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

        stream.write_all(b"+PONG\r\n")?;
    }

    Ok(())
}

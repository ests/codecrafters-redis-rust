use std::{net::TcpListener, io::Write, io::Read};

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                handle_client(s).unwrap();
                println!("accepted new connection");
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_client<T: Write + Read>(mut stream: T) -> std::io::Result<()> {
    let mut buf: [u8; 64] = [0; 64];
    stream.read(&mut buf)?;

    stream.write(b"+PONG\r\n")?;
    Ok(())
}

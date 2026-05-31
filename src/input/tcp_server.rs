use std::io::Read;
use std::net::{TcpListener, TcpStream};

pub fn start(addr: &str) -> std::io::Result<()> {
    let listener = TcpListener::bind(addr)?;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("Client connected");

                handle_client(stream);
            }

            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }

    Ok(())
}

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0u8; 4096];

    loop {
        match stream.read(&mut buffer) {
            Ok(0) => {
                println!("Client disconnected");
                break;
            }

            Ok(bytes_read) => {
                let data = &buffer[..bytes_read];

                println!("Received {} bytes", bytes_read);
                println!("{:?}", data);

                if let Ok(text) = std::str::from_utf8(data) {
                    println!("{}", text);
                }
            }

            Err(e) => {
                eprintln!("Read error: {}", e);
                break;
            }
        }
    }
}

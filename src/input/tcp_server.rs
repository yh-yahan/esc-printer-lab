use std::io::Read;
use std::net::{TcpListener, TcpStream};
use crate::parser::parser::Parser;
use crate::receipt::builder::ReceiptBuilder;

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
    let mut parser = Parser::new();
    let mut builder = ReceiptBuilder::new();

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

                let commands = parser.feed(data);

                for cmd in commands {
                    println!("{:?}", cmd);
                    builder.process(&cmd);
                }
            }

            Err(e) => {
                eprintln!("Read error: {}", e);
                break;
            }
        }
    }

    let receipt = builder.build();

    println!("{:#?}", receipt);
}

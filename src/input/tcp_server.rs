use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

use crate::parser::parser::Parser;
use crate::receipt::builder::ReceiptBuilder;
use crate::shared::print_session::PrintSession;

pub fn start(addr: &str, session: Arc<Mutex<PrintSession>>) -> std::io::Result<()> {
    let listener = TcpListener::bind(addr)?;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("Client connected: {:?}", stream.peer_addr());
                handle_client(stream, Arc::clone(&session));
            }

            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }

    Ok(())
}

fn handle_client(mut stream: TcpStream, session: Arc<Mutex<PrintSession>>) {
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
                session.lock().unwrap().push_raw(data);

                let commands = parser.feed(data);

                for cmd in &commands {
                    println!("{:?}", cmd);
                }

                for cmd in commands {
                    session
                        .lock()
                        .unwrap()
                        .push_parser(format!("{:?}", cmd));

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

    session.lock().unwrap().push_receipt(receipt);
}

use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

use eframe::egui::Context;

use crate::parser::command::Command;
use crate::parser::parser::Parser;
use crate::receipt::builder::ReceiptBuilder;
use crate::shared::print_session::{ParsedCommand, PrintSession};

pub fn start(
    addr: &str,
    session: Arc<Mutex<PrintSession>>,
    ctx: Context,
) -> std::io::Result<()> {
    let listener = TcpListener::bind(addr)?;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("Client connected: {:?}", stream.peer_addr());
                handle_client(stream, Arc::clone(&session), ctx.clone());
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }

    Ok(())
}

fn handle_client(mut stream: TcpStream, session: Arc<Mutex<PrintSession>>, ctx: Context) {
    let _ = stream.set_nodelay(true);

    let mut parser = Parser::new();
    let mut builder = ReceiptBuilder::new();
    let mut buffer = [0u8; 4096];
    let mut epoch = session.lock().unwrap().epoch;

    session.lock().unwrap().commit_current();

    loop {
        match stream.read(&mut buffer) {
            Ok(0) => {
                println!("Client disconnected");
                break;
            }
            Ok(bytes_read) => {
                let data = &buffer[..bytes_read];
                println!("Received {} bytes", bytes_read);

                {
                    let mut session = session.lock().unwrap();
                    if session.epoch != epoch {
                        parser = Parser::new();
                        builder = ReceiptBuilder::new();
                        epoch = session.epoch;
                    }
                    session.push_raw(data);
                }

                let commands = parser.feed(data);
                apply_commands(&commands, &mut builder, &parser, &session);
                ctx.request_repaint();
            }
            Err(e) => {
                eprintln!("Read error: {}", e);
                break;
            }
        }
    }

    let remaining_commands = parser.finish();
    apply_commands(&remaining_commands, &mut builder, &parser, &session);

    let mut session = session.lock().unwrap();
    session.update_current(builder.preview(parser.pending_text()));
    session.commit_current();
    ctx.request_repaint();
}

fn apply_commands(
    commands: &[ParsedCommand],
    builder: &mut ReceiptBuilder,
    parser: &Parser,
    session: &Arc<Mutex<PrintSession>>,
) {
    let mut session = session.lock().unwrap();

    for cmd in commands {
        match &cmd.command {
            Command::Initialize => {
                session.update_current(builder.preview(None));
                session.commit_current();
                builder.process(&cmd.command);
            }
            Command::Cut(_) => {
                builder.process(&cmd.command);
                session.update_current(builder.preview(None));
                session.commit_current();
                builder.start_new();
            }
            _ => builder.process(&cmd.command),
        }

        session.push_command(cmd.clone());
    }

    session.update_current(builder.preview(parser.pending_text()));
}

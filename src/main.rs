mod input;
mod parser;
mod receipt;
mod ui;
mod shared;

use std::sync::{Arc, Mutex};
use std::thread;

use crate::shared::print_session::PrintSession;

fn main() -> eframe::Result<()> {
    let session = Arc::new(Mutex::new(PrintSession::new()));

    let session_clone = Arc::clone(&session);

    thread::spawn(move || {
        if let Err(e) = input::tcp_server::start(
            "127.0.0.1:9100",
            session_clone,
        ) {
            eprintln!("TCP server error: {}", e);
        }
    });

    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "ESC Printer Lab",
        options,
        Box::new(|_cc| {
            Ok(Box::new(ui::app::App::new(session)))
        }),
    )
}

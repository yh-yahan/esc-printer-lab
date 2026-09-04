mod input;
mod parser;
mod printer;
mod receipt;
mod shared;
mod ui;

use std::sync::{Arc, Mutex};
use std::thread;

use crate::shared::print_session::PrintSession;

fn main() -> eframe::Result<()> {
    let session = Arc::new(Mutex::new(PrintSession::new()));
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "ESC Printer Lab",
        options,
        Box::new(move |cc| {
            let session_clone = Arc::clone(&session);
            let ctx = cc.egui_ctx.clone();

            thread::spawn(move || {
                if let Err(e) = input::tcp_server::start(
                    "127.0.0.1:9100",
                    session_clone,
                    ctx,
                ) {
                    eprintln!("TCP server error: {}", e);
                }
            });

            Ok(Box::new(ui::app::App::new(session)))
        }),
    )
}

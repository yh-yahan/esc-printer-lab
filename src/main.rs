mod input;
mod parser;
mod receipt;
mod ui;

use std::thread;
use std::sync::mpsc;

fn main() -> eframe::Result<()> {
    let (tx, rx) = mpsc::channel();

    let tx_clone = tx.clone();

    thread::spawn(|| {
        if let Err(e) = input::tcp_server::start("127.0.0.1:9102", tx_clone) {
            eprintln!("TCP server error: {}", e);
        }
    });

    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "ESC Printer Lab",
        options,
        Box::new(|_cc| Ok(Box::new(ui::main_window::MainWindow::new(rx)))),
    )
}

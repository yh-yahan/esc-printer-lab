mod input;
mod parser;
mod receipt;

fn main() -> std::io::Result<()> {
    input::tcp_server::start("0.0.0.0:9100")
}

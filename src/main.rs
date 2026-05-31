mod input;
mod parser;

fn main() -> std::io::Result<()> {
    input::tcp_server::start("0.0.0.0:9100")
}

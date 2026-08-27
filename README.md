# ESC Printer Lab

ESC/POS thermal receipt printer emulator written in Rust.

Listens for ESC/POS data over TCP and shows a live visual preview of the receipt, along with debug information.

## Current Status

Currently supported commands:

| Command | Function |
|---|---|
| `ESC @` | Initialize |
| `ESC a n` | Set text alignment |
| `ESC E n` | Set bold |
| `ESC - n` | Set underline style |
| `GS V n` | Cut paper |
| `GS ! n` | Set character size |
| `LF` (`0x0A`) | Line feed |
| Plain text | Print text |

## How to Run

```bash
git clone https://github.com/yh-yahan/esc-printer-lab.git
cd esc-printer-lab
cargo run
```

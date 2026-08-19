# ESC Printer Lab

ESC/POS thermal receipt printer emulator written in Rust.

It listens for ESC/POS data over TCP and shows a live visual preview of the receipt, along with debug information.

---

## Current Status

Currently supported commands:

- `ESC @` — Initialize
- `ESC a n` — Alignment (Left / Center / Right)
- `ESC E n` — Bold on/off
- `ESC - n` — Underline (Off / Thin / Thick)
- `GS V n` — Paper cut
- `LF` (`0x0A`) — Line feed
- Plain text

Missing (planned):
- Font size / double width & height
- Character code pages
- Images / raster bitmaps
- Barcodes & QR codes

---

## How to Run

```bash
git clone https://github.com/yh-yahan/esc-printer-lab.git
cd esc-printer-lab
cargo run
```

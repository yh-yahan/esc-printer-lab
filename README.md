# ESC Printer Lab

ESC/POS thermal receipt printer emulator written in Rust.

Listens for ESC/POS data over TCP and shows a live visual preview of the receipt, along with a protocol inspector.

<img width="1670" height="1182" alt="image" src="https://github.com/user-attachments/assets/d4a55281-2ff6-4e8b-84c7-4c663d24d86e" />

## Supported Commands

Currently supported commands:

| Command | Bytes | Effect |
| --- | --- | --- |
| Initialize | `ESC @` | Reset printer state |
| Align | `ESC a n` | Left / center / right |
| Bold | `ESC E n` | Emphasized text on/off |
| Underline | `ESC - n` | Off / thin / thick |
| Default line spacing | `ESC 2` | Restore default spacing |
| Set line spacing | `ESC 3 n` | Spacing in dots |
| Print and feed lines | `ESC d n` | Print buffer, feed *n* lines |
| Print and feed dots | `ESC J n` | Print buffer, feed *n* dots |
| Character size | `GS ! n` | Width/height 1x–8x |
| Cut | `GS V n` | Full or partial cut |
| Raster image | `GS v 0` | Raster bit image |
| QR model / size / ECC / store / print | `GS ( k` | QR Code setup and print |
| Line feed | `LF` (`0x0A`) | Print and advance one line |
| Carriage return | `CR` (`0x0D`) | Return to start of line |
| Code page | `ESC t n` | Select character code table |
| International set | `ESC R n` | Replace a few ASCII punctuation chars |
| Text | printable bytes | Print text |
| Unknown | other sequences | Preserved as raw bytes |

## How to Run

```bash
git clone https://github.com/yh-yahan/esc-printer-lab.git
cd esc-printer-lab
cargo run
```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.


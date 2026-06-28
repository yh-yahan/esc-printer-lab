pub struct EscPosFormatter;

impl EscPosFormatter {
    pub fn format(bytes: &[u8]) -> Vec<String> {
        let mut out = Vec::new();
        let mut i = 0;

        while i < bytes.len() {
            let b = bytes[i];

            match b {
                0x1B => {
                    i += 1;
                    if i >= bytes.len() { break; }

                    match bytes[i] {
                        0x40 => out.push("ESC @".into()),
                        0x61 => {
                            i += 1;
                            if i < bytes.len() {
                                out.push(format!("ESC a {}", bytes[i]));
                            }
                        }
                        0x45 => {
                            i += 1;
                            if i < bytes.len() {
                                out.push(format!("ESC E {}", bytes[i]));
                            }
                        }
                        0x2D => {
                            i += 1;
                            if i < bytes.len() {
                                out.push(format!("ESC - {}", bytes[i]));
                            }
                        }
                        _ => out.push(format!("ESC ? {:02X}", bytes[i])),
                    }
                }

                0x0A => out.push("LF".into()),

                printable if printable >= 0x20 && printable <= 0x7E => {
                    let start = i;

                    while i < bytes.len()
                        && bytes[i] >= 0x20
                        && bytes[i] <= 0x7E
                    {
                        i += 1;
                    }

                    let text = String::from_utf8_lossy(&bytes[start..i]).to_string();
                    out.push(format!("\"{}\"", text));

                    continue;
                }

                other => {
                    out.push(format!("0x{:02X}", other));
                }
            }

            i += 1;
        }

        out
    }
}

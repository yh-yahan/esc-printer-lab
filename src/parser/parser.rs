use super::command::{Command, Alignment};

pub fn parser(bytes: &[u8]) -> Vec<Command> {
    let mut commands = Vec::new();
    let mut i = 0;

    while i < bytes.len() {
        let b = bytes[i];

        match b {
            0x1B => {
                if i + 1 < bytes.len() {
                    let cmd = bytes[i + 1];

                    match cmd {
                        0x40 => {
                            commands.push(Command::Initialize);
                            i += 2;
                        }

                        0x45 => {
                            if i + 2 < bytes.len() {
                                let val = bytes[i + 2];
                                commands.push(Command::Bold(val != 0));
                                i += 3;
                            } else {
                                break;
                            }
                        }

                        0x61 => {
                            if i + 2 < bytes.len() {
                                let align = match bytes[i + 2] {
                                    0x00 => Alignment::Left,
                                    0x01 => Alignment::Center,
                                    0x02 => Alignment::Right,
                                    _ => Alignment::Left,
                                };

                                commands.push(Command::Align(align));
                                i += 3;
                            } else {
                                break;
                            }
                        }
                        
                        _ => {
                            i += 2;
                        }
                    }
                } else {
                    break;
                }
            }

            0x0A => {
                commands.push(Command::LineFeed);
                i += 1;
            }

            _ => {
                let start = i;

                while i < bytes.len() && bytes[i] != 0x1B && bytes[i] != 0x0A {
                    if bytes[i] < 0x20 && bytes[i] != 0x09 {
                        break;
                    }

                    i += 1;
                }

                let text = String::from_utf8_lossy(&bytes[start..i]).to_string();
                commands.push(Command::Text(text));
            }
        }
    }

    commands
}

#[derive(Debug, Clone)]
pub enum Command {
    Initialize,
    Text(String),
    LineFeed,
    Bold(bool),
    Align(Alignment),
    Cut,
}

#[derive(Debug, Clone, Copy)]
pub enum Alignment {
    Left,
    Center,
    Right
}

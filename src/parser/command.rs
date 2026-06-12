#[derive(Debug, Clone)]
pub enum Command {
    Initialize,
    Text(String),
    LineFeed,
    Bold(bool),
    Align(Alignment),
    Cut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Left,
    Center,
    Right
}

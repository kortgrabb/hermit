pub enum Colors {
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}
// TODO: Does this work on windows?
impl Colors {
    pub fn to_string(&self) -> String {
        match self {
            Colors::Red => "\x1b[31m",
            Colors::Green => "\x1b[32m",
            Colors::Yellow => "\x1b[33m",
            Colors::Blue => "\x1b[34m",
            Colors::Magenta => "\x1b[35m",
            Colors::Cyan => "\x1b[36m",
            Colors::White => "\x1b[37m",
        }
        .to_string()
    }
}

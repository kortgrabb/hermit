use crate::color::Colors;

pub enum LogLevel {
    Normal,
    Warning,
    Error,
}

pub fn log(to_print: &str, level: LogLevel) {
    match level {
        LogLevel::Normal => {
            println!("[LOG]:\n{to_print}");
        }
        LogLevel::Warning => {
            println!("{}[WARN]:\n{to_print}{}", Colors::Yellow, Colors::Reset)
        }
        LogLevel::Error => {
            println!("{}[ERROR]:\n{to_print}{}", Colors::Red, Colors::Reset);
        }
    }
}

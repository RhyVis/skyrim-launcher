use crossterm::{
    ExecutableCommand,
    style::{Color, ResetColor, SetForegroundColor},
};
use std::io::{self, Write};

pub fn clear_screen() {
    use crossterm::{
        ExecutableCommand,
        cursor::MoveTo,
        terminal::{Clear, ClearType},
    };

    let _ = io::stdout().execute(Clear(ClearType::All));
    let _ = io::stdout().execute(MoveTo(0, 0));
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {{
        use crossterm::style::Color;
        $crate::console::print_colored(Color::Green, "[INFO] ", format!($($arg)*));
    }};
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {{
        use crossterm::style::Color;
        $crate::console::print_colored(Color::Blue, "[DEBUG] ", format!($($arg)*));
    }};
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {{
        use crossterm::style::Color;
        $crate::console::print_colored(Color::Yellow, "[WARN] ", format!($($arg)*));
    }};
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        use crossterm::style::Color;
        $crate::console::print_colored(Color::Red, "[ERROR] ", format!($($arg)*));
    }};
}

pub fn print_colored(color: Color, prefix: &str, message: String) {
    let mut stdout = io::stdout();

    let _ = stdout.execute(SetForegroundColor(color));
    let _ = stdout.write_all(prefix.as_bytes());
    let _ = stdout.execute(ResetColor);
    let _ = stdout.write_all(message.as_bytes());
    let _ = stdout.write_all(b"\n");
    let _ = stdout.flush();
}

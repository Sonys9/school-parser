use core::fmt::Write;

use crate::syscalls::terminal;

#[macro_export]
macro_rules! terminal {
    ($($data:expr),* $(,)?) => {
        {
            use core::fmt::Write;
            let mut writer = crate::log::Terminal;
            let _ = write!(writer, $($data),*);
        }
    };
}

macro_rules! log_type {
    ($name:ident, $title:expr) => {
        macro_rules! $name {
            ($$($$data:expr),* $$(,)?) => {
                terminal!("[{}] {}\n", $title, format_args!($$($$data),*))
            }
        }
    };
}

log_type!(info, "\x1b[32mINFO\x1b[0m");
log_type!(error, "\x1b[31mERROR\x1b[0m");
log_type!(warn, "\x1b[33mWARN\x1b[0m");

pub struct Terminal;

impl Write for Terminal {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        terminal(s);
        Ok(())
    }
}

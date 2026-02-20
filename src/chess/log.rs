use std::env;
use std::sync::LazyLock;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
enum Level {
    Error = 0,
    Warn = 1,
}

impl Level {
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "error" | "err" => Self::Error,
            "warn" | "warning" => Self::Warn,
            _ => Self::Error,
        }
    }
}

static CHESS_LOG: LazyLock<Level> = LazyLock::new(|| {
    env::var("CHESS_LOG")
        .map(|s| Level::from_str(&s))
        .unwrap_or(Level::Error)
});

macro_rules! log {
    ($level:expr, $prefix:expr, $msg:expr) => {
        if *CHESS_LOG >= $level {
            eprintln!(concat!($prefix, ": {}"), $msg.as_ref());
        }
    };
}
pub fn error(msg: impl AsRef<str>) {
    log!(Level::Error, "ERROR", msg);
}
pub fn warn(msg: impl AsRef<str>) {
    log!(Level::Warn, "WARN", msg);
}

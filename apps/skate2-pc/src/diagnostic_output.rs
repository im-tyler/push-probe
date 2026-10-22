//! Routine client output shares the core's quiet flag. Fatal output bypasses it.
macro_rules! eprintln {
    ($($arg:tt)*) => {{
        if !skate2_core::diagnostics::is_quiet() { std::eprintln!($($arg)*); }
    }};
}
macro_rules! println {
    ($($arg:tt)*) => {{
        if !skate2_core::diagnostics::is_quiet() { std::println!($($arg)*); }
    }};
}

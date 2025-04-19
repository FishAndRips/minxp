#[macro_export]
macro_rules! print {
    ($what:literal) => {{
        let _ = $crate::io::Write::write(&mut $crate::io::stdout(), $what.as_bytes());
    }};
}
#[macro_export]
macro_rules! println {
    () => {{
        $crate::print!("\n")
    }};
    ($($arg:tt)*) => {{
        let _ = core::fmt::Write::write_fmt(&mut $crate::io::stdout(), format_args!($($arg)*));
        $crate::println!();
    }};
}
#[macro_export]
macro_rules! eprint {
    ($what:literal) => {{
        let _ = $crate::io::Write::write(&mut $crate::io::stderr(), $what.as_bytes());
    }};
}
#[macro_export]
macro_rules! eprintln {
    () => {{
        $crate::eprint!("\n")
    }};
    ($($arg:tt)*) => {{
        let _ = core::fmt::Write::write_fmt(&mut $crate::io::stderr(), format_args!($($arg)*));
        $crate::eprintln!();
    }};
}

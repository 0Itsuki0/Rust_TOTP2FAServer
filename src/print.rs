#[macro_export]
macro_rules! print_green {
    () => {
        println!()
    };
    ($($arg:tt)*) => {{
        println!("\x1b[32m{}\x1b[m", format!($($arg)*));
    }};
}

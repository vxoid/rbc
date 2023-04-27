#[macro_export]
macro_rules! verbose {
    ($info:expr) => {
        {println!("INFO: {}", $info)}
    };
}

#[macro_export]
macro_rules! warning {
    ($warning:expr) => {
        {println!("\x1b[33mWARNING: {}\x1b[0m", $warning)}
    };
}

#[macro_export]
macro_rules! error {
    ($error:expr) => {
        {println!("\x1b[31mERROR: {}\x1b[0m", $error)}
    };
}
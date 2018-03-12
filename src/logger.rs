use std::env;
use std::str::FromStr;

extern crate time;

#[derive(Debug,Eq,Ord,PartialEq,PartialOrd)]
pub enum Level {
    DEBUG,
    INFO,
    WARN,
    ERROR,
}

impl FromStr for Level {
    type Err = ParseLevelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "DEBUG"|"debug" => Ok(Level::DEBUG),
            "INFO"|"info" => Ok(Level::INFO),
            "WARN"|"warn" => Ok(Level::WARN),
            "ERROR"|"error" => Ok(Level::ERROR),
            _ => Err(ParseLevelError(format!("unknown logger::Level: {}", s))),
        }
    }
}

// TODO: Consider the way to implement ParseLevelError as a normal error handler.
#[derive(Debug)]
pub struct ParseLevelError(String);

pub fn log(level: Level, msg: &str) {
    let min_level = env::var("LOGLEVEL").unwrap_or("WARN".to_string()).parse::<Level>().expect("illegal LOGLEVEL");
    if level >= min_level {
        println!("{:>5} {} {}", format!("{:?}", level), time::now().rfc3339(), msg);
    }
}

#[macro_export]
macro_rules! log {
    ($lv:expr, $($e:tt)+) => ($crate::logger::log($lv, &format!($($e)*)));
}

// NOTICE: We cannot use a higher-order macro for generating macros taking variable arguments :(
#[macro_export]
macro_rules! debug {
    ($($e:tt)+) => (log!($crate::logger::Level::DEBUG, $($e)*));
}

#[macro_export]
macro_rules! info {
    ($($e:tt)+) => (log!($crate::logger::Level::INFO, $($e)*));
}

#[macro_export]
macro_rules! warn {
    ($($e:tt)+) => (log!($crate::logger::Level::WARN, $($e)*));
}

#[macro_export]
macro_rules! error {
    ($($e:tt)+) => (log!($crate::logger::Level::ERROR, $($e)*));
}

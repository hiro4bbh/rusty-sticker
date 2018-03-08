use std::cmp;
use std::env;
use std::str;

extern crate time;

#[repr(usize)]
#[derive(Copy,Debug,Eq)]
pub enum Level {
    DEBUG,
    INFO,
    WARN,
    ERROR,
}

impl Clone for Level {
    fn clone(&self) -> Self {
        *self
    }
}

impl cmp::PartialEq for Level {
    fn eq(&self, other: &Self) -> bool {
        *self as usize == *other as usize
    }
}

impl cmp::PartialOrd for Level {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl cmp::Ord for Level {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        (*self as usize).cmp(&(*other as usize))
    }
}

impl str::FromStr for Level {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &*s.to_string().to_uppercase() {
            "DEBUG" => Ok(Level::DEBUG),
            "INFO" => Ok(Level::INFO),
            "WARN" => Ok(Level::WARN),
            "ERROR" => Ok(Level::ERROR),
            _ => Err(format!("unknown logger::Level: {}", s)),
        }
    }
}

pub fn log(level: Level, msg: &str) {
    let min_level = env::var("LOGLEVEL").unwrap_or("warn".to_string()).parse::<Level>().unwrap_or(Level::WARN);
    if level >= min_level {
        println!("{:>5} {} {}", format!("{:?}", level), time::now().rfc3339(), msg);
    }
}

macro_rules! log {
    ($lv:expr, $($e:tt)+) => ($crate::logger::log($lv, &format!($($e)*)));
}

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

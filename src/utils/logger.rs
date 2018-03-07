extern crate log;

extern crate time;

pub struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Debug
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            println!("{:>5} {} {}", record.level(), time::now().rfc3339(), record.args());
        }
    }

    fn flush(&self) {}
}

pub fn init_logger() {
    log::set_max_level(log::LevelFilter::Debug);
    log::set_logger(&Logger).expect("log::set_logger must succeed");
}

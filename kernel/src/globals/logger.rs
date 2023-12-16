/// The console logger.

static LOGGER: Logger = Logger { filter: log::LevelFilter::Info };

pub fn init() {
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(LOGGER.filter);
}

pub struct Logger {
    pub filter: log::LevelFilter,
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.filter
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            crate::console_println!("[{}] {}", record.level(), record.args());
        }
    }

    fn flush(&self) { }
}
use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;

#[derive(Debug)]
pub enum LogLevel {
    DEBUG,
    INFO,
    WARN,
    ERROR,
}


pub struct RawFunc {
    data: Box<dyn Fn() + Send + 'static>,
}

impl RawFunc {
    pub fn new<T>(data: T) -> Self
        where
            T: Fn() + Send + 'static,
    {
        RawFunc {
            data: Box::new(data),
        }
    }

    fn invoke(self) {
        (self.data)()
    }
}

pub struct LogLineSpec {
    pub level: LogLevel,
    pub fmt: &'static str,
    pub fmt_fn: RawFunc
}

lazy_static! {
    pub static ref LOG_LINE_SPECS: Arc<Mutex<Vec<LogLineSpec>>> = Arc::new(Mutex::new(Vec::new()));
}


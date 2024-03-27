#![feature(type_alias_impl_trait)]
use std::cell::OnceCell;
use std::sync::{Arc, mpsc, Mutex};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use lazy_static::lazy_static;
use core_affinity;
use std::fmt::{Debug, Display};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_derive::{Deserialize, Serialize};

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
    pub log_ident: &'static str,
    pub fmt_fn: RawFunc,
    pub sender: OnceCell<Sender<Msg>>,
}

type Msg = String;

lazy_static! {
    pub static ref LOG_LINE_SPECS: Arc<Mutex<Vec<LogLineSpec>>> = Arc::new(Mutex::new(Vec::new()));
}


pub fn add_log_line_spec(spec: LogLineSpec) -> usize {
    let mut locked_vec = LOG_LINE_SPECS.lock().unwrap();
    locked_vec.push(spec);
    locked_vec.len()-1
}

pub fn init_logger(id: usize) {
    println!("init_logger called with cpu {}", id);
    let core_id = core_affinity::CoreId { id };
    let (tx, rx): (mpsc::Sender<String>, Receiver<String>) = mpsc::channel();

    // TODO can we find a way to share the sender so that each log line doesn't need the 8 bytes to do it?
    for spec in LOG_LINE_SPECS.lock().unwrap().iter_mut() {
        spec.sender.set(tx.clone()).unwrap();
    }

    thread::spawn(move || {
        core_affinity::set_for_current(core_id);

        for received in rx {
            println!("Received: {}", received);
        }
    });
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Loggable {
    I64(i64),
    F64(f64),
}

impl From<f64> for Loggable {
    fn from(item: f64) -> Self {
        Loggable::F64(item)
    }
}

impl From<i64> for Loggable {
    fn from(item: i64) -> Self {
        Loggable::I64(item)
    }
}

impl Display for Loggable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Loggable::I64(i) => write!(f, "{}", i),
            Loggable::F64(i) => write!(f, "{}", i),
        }
    }
}

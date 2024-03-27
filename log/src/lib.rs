#![feature(type_alias_impl_trait)]
use std::cell::OnceCell;
use std::sync::{Arc, mpsc, Mutex};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use lazy_static::lazy_static;
use core_affinity;
use std::fmt::Debug;
use serde::de::DeserializeOwned;
use serde::Serialize;

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

pub trait RemoteDebug: Serialize + DeserializeOwned + Debug {}
impl<T> RemoteDebug for T where T: Serialize + DeserializeOwned + Debug {}


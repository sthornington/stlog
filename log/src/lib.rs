#![feature(type_alias_impl_trait)]
use std::cell::{OnceCell, RefCell};
use std::sync::{Arc, Condvar, mpsc, Mutex};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use lazy_static::lazy_static;
use core_affinity;
use std::fmt::{Debug, Display, LowerExp};
use std::sync::atomic::Ordering;
use atomic_enum::atomic_enum;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_derive::{Deserialize, Serialize};

#[atomic_enum]
#[derive(PartialEq,PartialOrd)]
pub enum LogLevel {
    DEBUG = 0,
    INFO,
    WARN,
    ERROR,
}

// TODO: switch to channel that can do variable size messages
#[derive(Debug)]
pub struct Msg {
    pub data: [u8; MAX_SIZE],
}

impl Msg {
    pub fn new() -> Self {
        Msg { data: [0; MAX_SIZE] }
    }
}


pub struct RawFunc {
    data: Box<dyn Fn(Msg) + Send + 'static>,
}

impl RawFunc {
    pub fn new<T>(data: T) -> Self
        where
            T: Fn(Msg) + Send + 'static,
    {
        RawFunc {
            data: Box::new(data),
        }
    }

    fn invoke(&self, x: Msg) {
        (&self.data)(x)
    }
}

pub struct LogLineSpec {
    pub level: LogLevel,
    pub fmt: &'static str,
    pub log_ident: &'static str,
    pub fmt_fn: Option<RawFunc>,
}

pub const MAX_SIZE: usize = 256;

lazy_static! {
    pub static ref LOG_LINE_SPECS: Arc<Mutex<Vec<LogLineSpec>>> = Arc::new(Mutex::new(Vec::new()));
    pub static ref SENDER: Arc<Mutex<Option<Sender<Msg>>>> = Arc::new(Mutex::new(None));
}

thread_local! {
    pub static THREAD_LOCAL_SENDER: RefCell<Option<Sender<Msg>>> = RefCell::new(None);
}

pub fn add_log_line_spec(spec: LogLineSpec) -> usize {
    let mut locked_vec = LOG_LINE_SPECS.lock().unwrap();
    locked_vec.push(spec);
    locked_vec.len()-1
}

pub fn init_logger(id: usize) {
    let pair = Arc::new((Mutex::new(false), Condvar::new()));
    let pair_clone = pair.clone();
    // TODO guard against multiple calls
    println!("init_logger called with cpu {}", id);
    let core_id = core_affinity::CoreId { id };
    let (tx, rx): (mpsc::Sender<Msg>, Receiver<Msg>) = mpsc::channel();

    SENDER.lock().unwrap().replace(tx);

    thread::spawn(move || {
        core_affinity::set_for_current(core_id);
        // peel out all the formatting closures
        let fns = LOG_LINE_SPECS.lock().unwrap().iter_mut().map(|spec| spec.fmt_fn.take().unwrap()).collect::<Vec<RawFunc>>();

        let (lock, cvar) = &*pair_clone;
        *lock.lock().unwrap() = true;
        cvar.notify_one();

        for msg in rx {
            let idx: i32 = bincode::deserialize(&msg.data).unwrap();
            fns[idx as usize].invoke(msg);
        }
        eprintln!("Logger thread exiting");

    });

    let (lock, cvar) = &*pair;
    let mut started = lock.lock().unwrap();
    while !*started {
        started = cvar.wait(started).unwrap();
    }
}

static LOG_LEVEL: AtomicLogLevel = AtomicLogLevel::new(LogLevel::DEBUG);

pub fn set_log_level(level: LogLevel) {
    LOG_LEVEL.store(level, Ordering::Relaxed);
}

pub fn get_log_level() -> LogLevel {
    LOG_LEVEL.load(Ordering::Relaxed)
}

// TODO: shutdown logger/flush/poison pill ??

#[derive(Serialize, Deserialize, Debug)]
pub enum Loggable<'a> {
    I64(i64),
    F64(f64),
    Str(&'a str),
}

impl <'a> From<f64> for Loggable<'a> {
    fn from(item: f64) -> Self {
        Loggable::F64(item)
    }
}

impl <'a> From<i64> for Loggable<'a> {
    fn from(item: i64) -> Self {
        Loggable::I64(item)
    }
}

impl <'a> From<&'a str> for Loggable<'a> {
    fn from(item: &'a str) -> Self {
        Loggable::Str(item)
    }
}

impl <'a> Display for Loggable<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Loggable::I64(x) => Display::fmt(x, f),
            Loggable::F64(x) => Display::fmt(x, f),
            Loggable::Str(x) => Display::fmt(x, f),
        }
    }
}

impl <'a> LowerExp for Loggable<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Loggable::I64(x) => LowerExp::fmt(x, f),
            Loggable::F64(x) => LowerExp::fmt(x, f),
            Loggable::Str(x) => panic!("Not implemented"), // not ideal, not sure how to fix
        }
    }
}
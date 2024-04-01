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

#[atomic_enum]
#[derive(PartialEq,PartialOrd)]
pub enum LogLevel {
    DEBUG = 0,
    INFO,
    WARN,
    ERROR,
}

pub struct RawFunc {
    data: Box<dyn Fn(&[u8]) -> usize + Send + 'static>,
}

impl RawFunc {
    pub fn new<T>(data: T) -> Self
        where
            T: Fn(&[u8]) -> usize + Send + 'static,
    {
        RawFunc {
            data: Box::new(data),
        }
    }

    fn invoke(&self, x: &[u8]) -> usize {
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
    pub static ref SENDER: Arc<Mutex<Option<Sender<ByteCueueReader>>>> = Arc::new(Mutex::new(None));
}

type ByteCueueWriter = cueue::Writer<u8>;
type ByteCueueReader = cueue::Reader<u8>;

pub struct TLSWrapper {
    pub sender: ByteCueueWriter,
}

thread_local! {
    pub static THREAD_LOCAL: RefCell<Option<TLSWrapper>> = RefCell::new(None);
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
    let (tx, rx): (mpsc::Sender<ByteCueueReader>, Receiver<ByteCueueReader>) = mpsc::channel();

    SENDER.lock().unwrap().replace(tx);

    thread::spawn(move || {
        core_affinity::set_for_current(core_id);
        let mut log_streams: Vec<ByteCueueReader> = Vec::new();
        // peel out all the formatting closures
        let fns = LOG_LINE_SPECS.lock().unwrap().iter_mut().map(|spec| spec.fmt_fn.take().unwrap()).collect::<Vec<RawFunc>>();

        let (lock, cvar) = &*pair_clone;
        *lock.lock().unwrap() = true;
        cvar.notify_one();

        for i in 0.. {
            let mut j = 0;
            while j < log_streams.len() {
                let mut dead_and_drained = false;
                {
                    {
                        let chunk = log_streams[j].read_chunk();
                        let mut chunk_idx = 0;

                        while chunk_idx < chunk.len() {
                            //println!("READ chunk {:?}", chunk);
                            let (msg_idx, _) = bincode::decode_from_slice::<i32, _>(&chunk[chunk_idx..], bincode::config::legacy()).unwrap();
                            //println!("Got a message of size {} with index {}", size, msg_idx);
                            let read = fns[msg_idx as usize].invoke(&chunk[chunk_idx..]);
                            chunk_idx += read as usize;
                        }
                        assert!(chunk_idx == chunk.len());
                        dead_and_drained = chunk.len() == 0 && log_streams[j].is_abandoned();
                    }
                    log_streams[j].commit();
                }
                if dead_and_drained {
                    println!("Cleaning up a reader");
                    log_streams.swap_remove(j);
                } else {
                    j += 1;
                }
            }
            if i % 64 == 0 {
                for reader in rx.try_iter() {
                    println!("Got a new reader");
                    log_streams.push(reader);
                }
            }
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

#[derive(bincode::Encode, bincode::BorrowDecode, Debug)]
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
            Loggable::Str(_) => panic!("Not implemented"), // not ideal, not sure how to fix
        }
    }
}
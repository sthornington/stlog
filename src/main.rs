#![feature(type_alias_impl_trait)]

mod another;

#[macro_use]
extern crate log_macro;

use std::fmt::Debug;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::Duration;

use log::Loggable;
use crate::another::woob;

fn main() {
    log::init_logger(8);
    log::set_log_level(log::LogLevel::INFO);

    log_data!(INFO, "XXX {:+e}", 5.5);

    const N: i64 = 10000;


    let pair = Arc::new((Mutex::new(false), Condvar::new()));
    let pair_clone = pair.clone();
    thread::spawn(move || {
        log_data!(INFO, "XXX Notifying we are starting");
        let (lock, cvar) = &*pair_clone;
        *lock.lock().unwrap() = true;
        cvar.notify_one();

        for i in 0..1000 {
            log_data!(INFO, "XXX foo {}", i);
            //sleep(std::time::Duration::from_micros(1));
        }
        log_data!(INFO, "XXX Exiting thread");
    });

    let (lock, cvar) = &*pair;
    let mut started = lock.lock().unwrap();
    while !*started {
        started = cvar.wait(started).unwrap();
    }
    log_data!(INFO, "XXX We've started");

    let mut elapsed: Duration = Duration::new(0, 0);

    for i in 0..N {
        let f = i as f64;
        let x = f * f;
        let y = x * x;
        let z = y * y;
        let start = std::time::Instant::now();
        log_data!(INFO, "THIS IS A VERY LONG BIT OF NONSENSE TEXT ON EVERY LOG LINE ({}) i: {} f: {:.1} x: {:.1} y: {:.1} z: {:+e}", if i % 2 == 0 { "EVEN" } else { "ODD" }, i, f, x, y, z);
        //println!("THIS IS A VERY LONG BIT OF NONSENSE TEXT ON EVERY LOG LINE ({}) i: {} f: {:.1} x: {:.1} y: {:.1} z: {:+e}", if i % 2 == 0 { "EVEN" } else { "ODD" }, i, f, x, y, z);
        //log_data!(INFO, "THIS IS A VERY LONG BIT OF NONSENSE TEXT ON EVERY LOG LINE");
        elapsed += start.elapsed();
        woob();
        //sleep(std::time::Duration::from_micros(1));

    }

    println!("XXX log cost {:?}", elapsed / N as u32);

    sleep(std::time::Duration::from_secs(1));
}
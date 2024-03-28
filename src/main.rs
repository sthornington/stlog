#![feature(type_alias_impl_trait)]

mod another;

#[macro_use]
extern crate log_macro;

use std::fmt::Debug;
use std::thread::sleep;
use std::time::Duration;
use serde::de::DeserializeOwned;
use serde::Serialize;
use log::Loggable;
use crate::another::woob;

fn main() {
    log::init_logger(8);

    let mut elapsed: Duration = Duration::new(0, 0);
    log_data!(INFO, "{:+e}", 5.5);

    const N: i64 = 100000;
    for i in 0..N {
        let f = i as f64;
        let x = f * f;
        let y = x * x;
        let z = y * y;
        let start = std::time::Instant::now();
        log_data!(INFO, "THIS IS A VERY LONG BIT OF NONSENSE TEXT ON EVERY LOG LINE ({}) i: {} f: {:.1} x: {:.1} y: {:.1} z: {:+e}", if i % 2 == 0 { "EVENT" } else { "ODD" }, i, f, x, y, z);
        elapsed += start.elapsed();
        woob();
    }

    println!("log cost {:?}", elapsed / N as u32);
    sleep(std::time::Duration::from_secs(1));
}
#![feature(type_alias_impl_trait)]
#[macro_use]
extern crate log_macro;

use std::fmt::Debug;
use std::thread::sleep;
use serde::de::DeserializeOwned;
use serde::Serialize;
use log::Loggable;

fn main() {
    log::init_logger(8);
//    let (a,b) = gen_tuple();
//    log_data!(INFO, "a: {:?}, b: {:?}", a, b);
//    let a: i64 = 1;
//    let b: f64 = 2.0;
//    let t2: (Loggable, Loggable) = (a.into(), b.into());

//    println!("t2: {:?}", t2);
    log_data!(INFO, "FOO 1: {}, 2: {}", 1, 2.1);
    let a = 2.0;
    let b = a * a;
    let c = b * b;

    log_data!(INFO, "BAR {} {} {}", a, b, c);
    sleep(std::time::Duration::from_secs(1));
}
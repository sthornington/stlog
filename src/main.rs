#![feature(type_alias_impl_trait)]
#[macro_use]
extern crate log_macro;

use std::fmt::Debug;
use serde::de::DeserializeOwned;
use serde::Serialize;
use log::Loggable;

fn main() {
//    let (a,b) = gen_tuple();
//    log_data!(INFO, "a: {:?}, b: {:?}", a, b);
    let a: i64 = 1;
    let b: f64 = 2.0;
    let t2: (Loggable, Loggable) = (a.into(), b.into());

    println!("t2: {:?}", t2);
    log_data!(INFO, "FOO 1: {:?}, 2: {:?}", 1, 2.0);
}
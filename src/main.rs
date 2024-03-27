#![feature(type_alias_impl_trait)]
#[macro_use]
extern crate log_macro;

use std::fmt::Debug;
use serde::de::DeserializeOwned;
use serde::Serialize;

fn get_tuple(a: impl log::RemoteDebug, b: impl log::RemoteDebug) -> (impl log::RemoteDebug, impl log::RemoteDebug) {
    (a,b)
}

fn main() {
//    let (a,b) = gen_tuple();
//    log_data!(INFO, "a: {:?}, b: {:?}", a, b);
    let a = 1;
    let b = 2;
    let t = get_tuple(a,b);
    println!("t: {:?}", t);
//    log_data!(INFO, "1: {:?}, 2: {:?}", 1, 2);

}
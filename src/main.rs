#[macro_use]
extern crate log_macro;

use std::sync::{Arc, Mutex};
use constructor::constructor;
use lazy_static::lazy_static;

extern fn init1() {
    LOG_LINE_SPECS.lock().unwrap().push(LogLineSpec { level: 0, fmt: "init1" });
}

extern fn init2() {
    LOG_LINE_SPECS.lock().unwrap().push(LogLineSpec { level: 0, fmt: "init2" });
}

fn main() {
    {
        let specs = LOG_LINE_SPECS.lock().unwrap();
        for spec in specs.iter() {
            println!("{:?}", spec);
        }
    }
    log_data!("INFO", "User logged in", user_id=42);
    constructor!(init1);
    constructor!(init2);
}
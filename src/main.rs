#[macro_use]
extern crate log_macro;

use std::sync::{Arc, Mutex};
use constructor::constructor;
use lazy_static::lazy_static;

pub extern fn init1() {
    println!("first init1")
}

macro_rules! log_data {
  ($($arg:tt)*) => {
    log_data_impl!(file!(), line!(), col!(), $($arg)*)
  };
}


fn main() {
/*
    {
        pub mod init2 {
            #![allow(non_snake_case)]
            #![allow(dead_code)]
            #![allow(non_upper_case_globals)]
            #![deny(private_no_mangle_statics /* >>> constructor must be used from a pub mod <<< */)]

            pub extern fn init2f() {
                println!("init2f {} {}", file!(), line!());
            }

            #[cfg(target_os = "macos")]
            #[link_section = "__DATA,__mod_init_func"]
            #[no_mangle]
            pub static init2: extern fn() = init2f;
        }
    }

    constructor!(init1);
 */
    log_data!(INFO, "hi there");
}
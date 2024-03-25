#[macro_use]
extern crate log_macro;

fn main() {
    for i in 0..5 {
        log_data!(INFO, "hi there {}", i);
        log_data!(INFO, "how you doin {}", i);
    }
}
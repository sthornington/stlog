# stlog

Place to mess around with Rust, trying to implement a fast async logging system where just the variadic format
arguments are passed on a queue to a background logging thread, and the format text and all formatting is done
there. Oh, and also, with no dynamic allocation.  Wish me luck.

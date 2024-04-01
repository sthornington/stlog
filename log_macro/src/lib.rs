#![feature(proc_macro_span)]
#![feature(type_alias_impl_trait)]
extern crate proc_macro;

use proc_macro::{Literal, Span, TokenStream};
use quote::{quote, format_ident};
use syn::{parse_macro_input, LitStr, parse::Parse, parse::ParseStream, Token, Expr, Ident, LitInt};
use constructor::constructor;
use std::io::Cursor;

struct LogMacroInput {
    level: Ident,
    format_str: LitStr,
    args: Vec<Expr>,
}

impl syn::parse::Parse for LogMacroInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let level: Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let format_str: LitStr = input.parse()?;
        let mut args = Vec::new();

        while !input.is_empty() {
            input.parse::<Token![,]>()?;
            let arg: Expr = input.parse()?;
            args.push(arg);
        }

        Ok(LogMacroInput { level, format_str, args })
    }
}

/*
    * This macro generates the code to log data, by packing the args onto a queue and
    * registering a closure to deserialize and log the data. Inspired by:
    * https://www.reddit.com/r/rust/comments/15cm4ug/comment/jtxfttd/
    *
    * # Examples
    *
    * ```
    * log_data!(INFO, "hi there {}", 5);
    * ```
    */
#[proc_macro]
pub fn log_data(input: TokenStream) -> TokenStream {
    fn fnify(s: &str) -> String {
        s.chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect()
    }

    let LogMacroInput { level, format_str, args } = parse_macro_input!(input as LogMacroInput);
    let span = Span::call_site();
    let source = span.source_file();
    let path = source.path().to_str().unwrap().to_string();
    let file = fnify(&path);
    let line = span.start().line();
    let col = span.start().column();
    let log_ident = format_ident!("logident_{}_{}_{}", file, line, col);
    let log_ident_impl = format_ident!("logident_impl_{}_{}_{}", file, line, col);
    let log_ident_str = format!("logident_{}_{}_{}", file, line, col);

    // Map the `Ident` to a LogLevel variant
    let level = match level.to_string().as_str() {
        "Debug" => quote! { log::LogLevel::DEBUG },
        "INFO" => quote! { log::LogLevel::INFO },
        "WARN" => quote! { log::LogLevel::WARN },
        "ERROR" => quote! { log::LogLevel::ERROR },
        _ => panic!("Unsupported log level"),
    };

    let tuple_types = args.iter().map(|_| {quote! { log::Loggable }}).collect::<Vec<_>>();
    let vars = (0..args.len()).map(|i| format_ident!("var{}", i)).collect::<Vec<_>>();
    let tuple_args_into = args.iter().map(|x| {
        quote! { #x.into() }
    });

    // Register static stuff
    let register = quote! {
        use std::fmt::Debug;

        // taken from constructor crate, but without the super:: since our function is local
        pub mod #log_ident {
            #![allow(non_snake_case)]
            #![allow(dead_code)]
            #![allow(non_upper_case_globals)]
            #![deny(private_no_mangle_statics /* >>> constructor must be used from a pub mod <<< */)]
            use std::sync::atomic::{AtomicI32, Ordering};
            use std::cell::OnceCell;
            use std::sync::mpsc::{Receiver, Sender};
            use std::io::Cursor;
            use bincode::BorrowDecode;

            pub static idx: AtomicI32 = AtomicI32::new(-1);
            pub static level: log::LogLevel = #level;

            pub extern "C" fn #log_ident_impl() {
                let fmt_str_copy = #format_str.clone();
                let raw_func = log::RawFunc::new(move |data| {
                    // Deserialize the tuple
                    let ((msg_idx, #(#vars),*), bytes_read) : ((i32, #(#tuple_types),*), usize) = bincode::borrow_decode_from_slice(&data, bincode::config::legacy()).unwrap();
                    assert!(msg_idx == idx.load(Ordering::Relaxed));
                    println!(#format_str, #(#vars),* );
                    bytes_read
                } );
                let id = log::add_log_line_spec(log::LogLineSpec { level: #level, fmt: fmt_str_copy, log_ident: #log_ident_str, fmt_fn: Some(raw_func) });
                if let Err(prev) = idx.compare_exchange(-1, id as i32, Ordering::Acquire, Ordering::Relaxed) {
                    panic!("log call site at {}:{}:{} has already been initialized to {}!", #path, #line, #col, prev);
                }
            }
            #[cfg(target_os = "linux")]
            #[link_section = ".ctors"]
            #[no_mangle]
            pub static #log_ident: extern fn() = #log_ident_impl;
            #[cfg(target_os = "macos")]
            #[link_section = "__DATA,__mod_init_func"]
            #[no_mangle]
            pub static #log_ident: extern fn() = #log_ident_impl;
        }
    };

    // Generate serialization code
    let serialize = quote! {
        if false {
            // put this here just to force the compile time format type checking
            println!(#format_str, #(#args),* );
        }
        if log::get_log_level() <= #level {
            let idx = #log_ident::idx.load(std::sync::atomic::Ordering::Relaxed);
            assert!(idx >= 0);
            let t: ( i32, #( #tuple_types ),* ) = ( idx, #( #tuple_args_into ),* );
            log::THREAD_LOCAL.with(|maybe_tls| {
                if maybe_tls.borrow().is_none() {
                    use cueue;
                    // create this thread's TLS
                    let (mut w, mut r) = cueue::cueue(1 << 20).unwrap();

                    *maybe_tls.borrow_mut() = Some(log::TLSWrapper { sender: w });
                    // send our reader to the logger thread
                    log::SENDER.lock().unwrap().as_ref().expect("stlog not initialized").send(r).unwrap();
                }
                if let Some(ref mut tls) = *maybe_tls.borrow_mut() {
                    let mut chunk = tls.sender.write_chunk();

                    let written = bincode::encode_into_slice(&t, chunk, bincode::config::legacy()).expect("Serialization failed");
                    //println!("Serializing required_size {} size_size {} bytes idx {} WRITE CHUNK {:?}.", required_size, size_size, idx, &chunk[..required_size as usize]);
                    tls.sender.commit(written);

                }
            });
        }

    };

    let output = quote! {
        {
            #register
            #serialize
        }
    };

    output.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
    }
}

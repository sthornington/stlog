#![feature(proc_macro_span)]
#![feature(type_alias_impl_trait)]
extern crate proc_macro;

use proc_macro::{Literal, Span, TokenStream};
use quote::{quote, format_ident};
use syn::{parse_macro_input, LitStr, parse::Parse, parse::ParseStream, Token, Expr, Ident, LitInt};
use constructor::constructor;

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

    // Generate deserialize and log closure
    let deserialize_and_log = quote! {
        println!("TODO: deserialize and log closure");
    };
    println!("COMPILE {}", log_ident);
    println!("COMPILE args len {:?}", args.len());

    let tuple_args = args.iter().enumerate().map(|(i, _)| {
        quote! { impl log::RemoteDebug }
    }).collect::<Vec<_>>();
    println!("COMPILE tuple_args len {:?}", tuple_args.len());

    // Register static stuff
    let register = quote! {
        use std::fmt::Debug;
        use serde::de::DeserializeOwned;
        use serde::Serialize;


        // taken from constructor crate, but without the super:: since our function is local
        pub mod #log_ident {
            #![allow(non_snake_case)]
            #![allow(dead_code)]
            #![allow(non_upper_case_globals)]
            #![deny(private_no_mangle_statics /* >>> constructor must be used from a pub mod <<< */)]
            use std::sync::atomic::{AtomicI32, Ordering};
            use std::cell::OnceCell;

            pub type tuple_type = ( #( #tuple_args ),* );

            pub static idx: AtomicI32 = AtomicI32::new(-1);

            pub extern "C" fn #log_ident_impl() {
//                println!("{} was called, original fmt: \"{}\"", #log_ident_str, #format_str);
                // TODO: Lock the specs vec, the index will be this log's dense id. Store that
                // TODO: in the serialization code SOMEHOW so it can push the args with a good
                // TODO: dense id.
                let fmt_str_copy = #format_str.clone();
                let raw_func = log::RawFunc::new(move || { println!("TODO: deserialize and log args here with fmt {}", #format_str); } );
                let id = log::add_log_line_spec(log::LogLineSpec { level: #level, fmt: fmt_str_copy, log_ident: #log_ident_str, fmt_fn: raw_func, sender: OnceCell::new() });
                if let Err(prev) = idx.compare_exchange(-1, id as i32, Ordering::Acquire, Ordering::Relaxed) {
                    panic!("log call site at {}:{}:{} has already been initialized to {}!", #path, #line, #col, prev);
                } else {
                    println!("log call site at {}:{}:{} successfully initialized with id {}!", #path, #line, #col, id);
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
        println!("TODO: serialize id {} and args here", #log_ident::idx.load(std::sync::atomic::Ordering::Relaxed));
        let t = #log_ident::tuple_type ( #(#args),* );
        println!("{:?}", t);
        println!(#format_str, #(#args),*);
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

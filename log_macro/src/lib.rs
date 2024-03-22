extern crate proc_macro;

use proc_macro::{TokenStream};
use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;
use quote::{quote, format_ident};
use syn::{parse_macro_input, LitStr, parse::Parse, parse::ParseStream, Token, Expr, Ident};

enum LogLevel {
    DEBUG,
    INFO,
    WARN,
    ERROR,
}

#[derive(Debug)]
struct LogLineSpec {
    level: LogLevel,
    fmt: &'static str,
}

lazy_static! {
    static ref LOG_LINE_SPECS: Arc<Mutex<Vec<LogLineSpec>>> = Arc::new(Mutex::new(Vec::new()));
}



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

#[proc_macro]
pub fn log_data(input: TokenStream) -> TokenStream {
    let LogMacroInput { level, message, key, value } = parse_macro_input!(input as LogMacroInput);

    // Generate serialization code
    let serialize = quote! {
        let serialized = format!("{}={}", stringify!(#key), #value);
    };

    // Generate deserialization and formatting code
    let deserialize_and_log = quote! {
        println!("[{}] {}: {}", #level, #message, serialized);
    };

    let output = quote! {
        {
            #serialize
            #deserialize_and_log
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

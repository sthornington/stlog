extern crate proc_macro;

use proc_macro::{Literal, TokenStream};

use quote::{quote, format_ident};
use syn::{parse_macro_input, LitStr, parse::Parse, parse::ParseStream, Token, Expr, Ident, LitInt};
use constructor::constructor;

struct LogMacroInput {
    file: LitStr,
    line: LitInt,
    col: LitInt,
    level: Ident,
    format_str: LitStr,
    args: Vec<Expr>,
}

impl syn::parse::Parse for LogMacroInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let file: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;
        let line: LitInt = input.parse()?;
        input.parse::<Token![,]>()?;
        let col: LitInt = input.parse()?;
        input.parse::<Token![,]>()?;
        println!("PARSE {} {} {}", file.value(), line, col);
        let level: Ident = input.parse()?;
        let format_str: LitStr = input.parse()?;
        let mut args = Vec::new();

        while !input.is_empty() {
            input.parse::<Token![,]>()?;
            let arg: Expr = input.parse()?;
            args.push(arg);
        }

        Ok(LogMacroInput { file, line, col, level, format_str, args })
    }
}


#[proc_macro]
pub fn log_data_impl(input: TokenStream) -> TokenStream {
    let LogMacroInput { file, line, col, level, format_str, args } = parse_macro_input!(input as LogMacroInput);

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
    println!("COMPILE {} {}", file.value(), line);
    // Register static stuff
    let register = quote! {
        let (file, line) = (file!(), line!());
        println!("RUN {} {}", file, line);
            /*
        // TODO this is a function not a closure so it doesn't have format_str etc?
        extern fn inner_register() {
            let raw_func = log::RawFunc::new(move || {
                println!("TODO: deserialize and log args here with fmt {}", #format_str);
            });
            log::LOG_LINE_SPECS.lock().unwrap().push(log::LogLineSpec { level: #level, fmt: #format_str, fmt_fn: raw_func } );
        }
        constructor!(inner_register);
*/
    };
    // Generate serialization code
    let serialize = quote! {
        println!("TODO: serialize args here")
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

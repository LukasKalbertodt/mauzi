//! This crate defines the proc-macro `dict!`.
//!
//! You shouldn't use this crate directly, but use `mauzi` instead. The macro
//! is reexported there.

#![feature(proc_macro)]

extern crate literalext;
extern crate proc_macro;
extern crate mauzi_runtime;

use std::result::Result as StdResult;

mod ast;
mod gen;
mod parse;

/// Right now the only way to report errors to the user from within a proc-
/// macro is to panic, which prints a simple string. So right now our error
/// type for everything contains a string as error value.
type Result<T> = StdResult<T, String>;

/// Generates a dictonary hosting translations in different languages.
///
/// **TODO**: documentation
#[proc_macro]
pub fn dict(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    parse::parse(input)
        .and_then(gen::gen)
        .unwrap_or_else(|e| panic!(e))
}

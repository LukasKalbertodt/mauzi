//! This crate defines the proc-macro `mauzi!`.
//!
//! You shouldn't use this crate directly, but use `mauzi` instead. The macro
//! is reexported there.

#![feature(proc_macro)]

extern crate literalext;
extern crate proc_macro;

use std::result::Result as StdResult;

mod ast;
mod check;
mod gen;
mod parse;
mod util;

/// Right now the only way to report errors to the user from within a proc-
/// macro is to panic, which prints a simple string. So right now our error
/// type for everything contains a string as error value.
type Result<T> = StdResult<T, String>;

/// Generates a dictionary hosting translations in different languages.
///
/// **TODO**: documentation
#[proc_macro]
pub fn mauzi(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    use check::check;
    use gen::gen;
    use parse::parse;

    parse(input)
        .and_then(|ast| {
            check(&ast).map(|_| ast)
        })
        .and_then(gen)
        .unwrap_or_else(|e| panic!(e))
}

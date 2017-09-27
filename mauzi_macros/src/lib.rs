//! This crate defines the proc-macro `mauzi!`.
//!
//! You shouldn't use this crate directly, but use `mauzi` instead. The macro
//! is reexported there.

#![feature(proc_macro, catch_expr)]

extern crate literalext;
extern crate proc_macro;


#[macro_use]
mod util;

mod ast;
mod check;
mod gen;
mod parse;


use proc_macro::{Diagnostic, TokenStream};
use std::result::Result as StdResult;

/// Right now the only way to report errors to the user from within a proc-
/// macro is to panic, which prints a simple string. So right now our error
/// type for everything contains a string as error value.
type Result<T> = StdResult<T, Diagnostic>;

/// Generates a dictionary hosting translations in different languages.
///
/// **TODO**: documentation
#[proc_macro]
pub fn mauzi(input: TokenStream) -> TokenStream {
    use check::check;
    use gen::gen;
    use parse::parse;

    do catch {
        let ast = parse(input)?;
        check(&ast)?;
        gen(ast)
    }.unwrap_or_else(|e| {
        e.emit();
        TokenStream::empty()
    })
}

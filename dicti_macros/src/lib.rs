#![feature(proc_macro)]

extern crate literalext;
extern crate proc_macro;

use std::result::Result as StdResult;


mod ast;
mod gen;
mod parse;

type Result<T> = StdResult<T, String>;

#[proc_macro]
pub fn dict(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    parse::parse(input)
        .and_then(|ast| gen::gen(&ast))
        .unwrap_or_else(|e| panic!(e))
}

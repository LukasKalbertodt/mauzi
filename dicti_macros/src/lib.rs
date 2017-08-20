#![feature(proc_macro)]

extern crate literalext;
extern crate proc_macro;
extern crate proc_macro2;


mod ast;
mod parse;

type ParseResult<T> = Result<T, String>;

#[proc_macro]
pub fn dict(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: proc_macro2::TokenStream = input.into();

    let ast = match parse::parse(input) {
        Ok(ast) => ast,
        Err(e) => panic!("{}", e),
    };

    panic!("ah, we got an ast: {:#?}", ast)

    // output.into()
}

#[macro_use]
extern crate dicti_macros;

// #[macro_use]
// pub mod makro;

#[derive(Debug)]
pub enum Loc {
    De,
    En,
    Other(String),
}

// hello_world {
//     En => : "Hello World";
//     De => : "Hallo Welt";
// }
dict! {
    hello_world {
        En => "Hello World",
        De => "Hallo Welt",
    }
    greet(name: &str) {
        En => "Hi {name}",
        De => "Hallo {name}",
    }
}

#![feature(proc_macro)]

extern crate dicti;

use dicti::Locale;


mod dicto {
    use dicti::dict;

    dict! {
        hello_world {
            En => "Hello World",
            De(Ch) => "Hallo Welt",
            _ => "no idea...",
        }
        greet(name: &str, age: u8) {
            En => "Hi {name} with age {age}",
            De => {
                (2 + 4).to_string()
            }
        }
    }
}


fn main() {
    let dict = dicto::Dict::new(Locale::de());

    println!("{}", dict.hello_world());
    println!("{}", dict.greet("Lukas", 23));
}

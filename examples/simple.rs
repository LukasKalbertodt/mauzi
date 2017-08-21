#![feature(proc_macro)]

extern crate dicti;

use dicti::{Locale, EnRegion};


mod dicto {
    use dicti::dict;

    dict! {
        hello_world {
            En(Us) => "Hello USA",
            En(Gb) => "Bye Europe",
            De => "Hallo Welt",
        }
        drink {
            En(Gb) => "Tea",
            _ => "different kinds of things",
        }
        greet(name: &str, age: u8) {
            En => "Hi {name} with age {age}",
            De => {
                let cool_greeting = match age {
                    0...18 => "Junge",
                    19...25 => "Alta",
                    _ => "Mann",
                };
                format!("Hallo {}, {}!", name, cool_greeting)
            }
        }
    }
}


fn main() {
    use dicto::Dict;

    println!("{:?}", Locale::en().with_region_variant_str("Gb"));

    let locales = [
        Locale::de(),
        Locale::En(EnRegion::Gb),
        Locale::En(EnRegion::Us),
    ];

    for &locale in &locales {
        println!("--- for {:?} ---", locale);
        let dict = Dict::new(locale);

        println!("hello_world => {}", dict.hello_world());
        println!("greet       => {}", dict.greet("Lukas", 23));
        println!("drink       => {}", dict.drink());
    }
}

#![feature(proc_macro)]

extern crate mauzi;

use mauzi::{dict, Locale, EnRegion};


dict! {
    unit fav_color {
        De => "Was ist deine Lieblingsfarbe?",
        En(Gb) => "What is your favourite colour?",
        En(Us) => "What is your favorite color?",
    }
    unit greet(name: &str) {
        En(Gb) => "Hi {name}! Are you all right, mate?",
        En(Us) => "Hi {name}! How are you, buddy?",
        De => "Hallo {name}, wie geht's dir?",
    }
    unit new_emails(count: u32) {
        En => {
            match count {
                1 => "You have one new email".to_string(),
                _ => format!("You have {} new emails", count),
            }
        }
        De => {
            match count {
                1 => "Sie haben eine neue E-Mail".to_string(),
                _ => format!("Sie haben {} neue E-Mails", count),
            }
        }
    }
}

fn main() {
    let locales = [
        Locale::de(),
        Locale::En(EnRegion::Gb),
        Locale::En(EnRegion::Us),
    ];

    for &locale in &locales {
        println!("--- for {:?} ---", locale);
        let dict = Dict::new(locale);

        println!("greet       => {}", dict.greet("Ferris"));
        println!("fav_color   => {}", dict.fav_color());
        println!("new_emails  => {}", dict.new_emails(3));
    }
}

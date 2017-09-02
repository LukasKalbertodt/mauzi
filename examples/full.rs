#![feature(proc_macro)]

extern crate mauzi;


// The `mauzi` macro us usually invocated in a `dict` submodule. This submodule
// should live in its own file, but for this example, it's an inline module.
mod dict {
    use mauzi::mauzi;

    mauzi! {
        // The first thing in the macro invocation is the Locale definition.
        // Here you define which languages and regions your dictionary
        // supports.
        enum Locale {
            // You can have languages without distinguishing between regions...
            De,
            // ... but you can have regions for a given language, too.
            En { Gb, Us },
        }

        // A simple translation unit: it returns a string depending on the
        // locale.
        unit fav_color {
            De => "Was ist deine Lieblingsfarbe?",
            En(Gb) => "What is your favourite colour?",
            En(Us) => "What is your favorite color?",
        }

        // Translation units can take parameters. Those are declared in a pair
        // of parenthesis, just like parameters for a Rust function.
        //
        // You can then use the parameter in the string with the `{param}`
        // syntax.
        unit greet(name: &str) {
            En(Gb) => "Hi {name}! Are you all right, mate?",
            En(Us) => "Hi {name}! How are you, buddy?",
            De => "Hallo {name}, wie geht's dir?",
        }

        // Instead of simple strings, you can specify your own Rust code which
        // will generate a string instead. Note that you can't use the fancy
        // `{param}` syntax as above.
        unit new_emails(count: u32) {
            // Note that the region is omitted here. You can do that if the
            // region doesn't matter. This is equivalent to `En(_)`.
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

        // You can also specify custom return types. However, this requires you
        // to specify raw bodies. Custom return types are mostly useful for
        // preformatted HTML, like the `maud::Markup` type.
        unit number_of_umlauts -> u32 {
            De => { 3 },
            En => { 0 },
        }
    }
}

fn main() {
    use dict::{Locale, EnRegion};

    let locales = [
        Locale::De,
        Locale::En(EnRegion::Gb),
        Locale::En(EnRegion::Us),
    ];

    for &locale in &locales {
        println!("--- for {:?} ---", locale);
        let dict = dict::new(locale);

        // All translation keys are simple functions. You can access it like
        // calling a function.
        println!("greet       => {}", dict.greet("Ferris"));
        println!("fav_color   => {}", dict.fav_color());
        println!("new_emails  => {}", dict.new_emails(3));
        println!("umlauts     => {}", dict.number_of_umlauts());
    }
}

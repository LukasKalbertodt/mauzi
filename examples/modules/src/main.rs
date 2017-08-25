#![feature(proc_macro)]

extern crate mauzi;


mod dict;

fn main() {
    use dict::Locale;

    let locales = [
        Locale::De,
        Locale::En,
    ];

    for &locale in &locales {
        println!("--- for {:?} ---", locale);
        let dict = dict::new(locale);

        println!("cat              => {}", dict.cat());
        println!("foo::greet       => {}", dict.foo.greet("Lukas"));
        println!("bar::hello_world => {}", dict.bar.hello_world());
        println!("baz::bye_world   => {}", dict.bar.baz.bye_world());
    }
}

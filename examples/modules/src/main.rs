#![feature(proc_macro)]

extern crate mauzi;


mod dict;

fn main() {
    use dict::Locale;

    let dict = dict::new(Locale::De);
    println!("{}", dict.cat());
}

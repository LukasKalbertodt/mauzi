#[macro_use] extern crate dicti;

dict! {
    hello_world() {
        En => : "Hello World";
        De => : "Hallo Welt";
    }
    greet(name: &str) {
        En =>
            : "Hi ";
            : name;
        De =>
            : "Hallo ";
            : name;
    }
}

fn main() {
    use dicti::Loc;

    let dict = Dict::new(
        if std::env::args().count() == 2 { Loc::De } else { Loc::En }
    );

    println!("{}", dict.hello_world());
    println!("{}", dict.greet("Lukas"));
}

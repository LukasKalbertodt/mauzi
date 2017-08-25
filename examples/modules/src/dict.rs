use mauzi::mauzi;


mauzi! {
    enum Locale {
        De,
        En,
    }

    mod foo;
    mod bar;

    unit cat {
        De => "Katze",
        En => "cat",
    }
}

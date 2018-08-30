Mauzi
=====

Experimental library to help with internationalization -- using `proc_macro` macros. **This was an experiment of mine; the crate is not developed anymore.**

The idea behind this crate was the following: i18n is usually done by writing text files in a special format. Translators can also use some special functionality to ease pluralization and the like. Instead of having external files with strange syntax, I think Rust would benefit from having something more type-safe. You can see something similar in the domain of templating libraries: there are a few ones that work with strings as input (handlebars, tera, ...). And then there is [`maud`](https://github.com/lfairy/maud): here, the template is written in a syntax inside a `proc_macro`. This enables way greater type-safety and has a couple of other benefits. 

This library wanted to be the `maud` of i18n. To get an idea what this crate looks like: take a look at [this example](https://github.com/LukasKalbertodt/mauzi/blob/master/examples/full.rs):

```rust
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
```


## Status

This library was just an experiment. The project I used this for is no more, 
so I don't have an direct motivation to work on this. Maybe I will revisit 
this crate again in the future.


## Prepare for trouble, make it double (list of dirty hacks)

Right now this library is as unstable as a house of cards made from flerovium
in the pre-Rust-1.0 era. Partly due to my lack of understanding of certain
things, partly due to the incomplete and unstable nature of the `proc_macro`
feature. Here, I want to list all evil hacks I used right now.


### Emulate module system

I was unable to map mauzi-modules to Rust modules. The problem is that, from
the submodules, I need to use `Dict` which is defined in the root module.
Sounds easy in theory, but due to macro hygiene (I think?) it is complicated.
Hardly any `use` statements work as you would expect. Maybe it works once
[this][1] lands, maybe not. I made another comment about something similar
[here][2]. Maybe it is already possible but I'm unable to find the solution.

The current solution is to build long names for types in submodules. So
instead of:

```
dict::bar::baz::Dict
```
... the type is:

```
dict::bar___this_is_a_bad_solution___baz___this_is_a_bad_solution___Dict
```

[1]: https://github.com/rust-lang/rfcs/issues/959
[2]: https://github.com/rust-lang/rust/issues/38356#issuecomment-323734922


### Loading of sub module files

The proc macro can't find out in which file it was called. This means we don't
know where to look for submodule files! This is discussed [here][3].

The current solution uses `CARGO_MANIFEST_PATH` and adds `src/`. This means:

- You have to call the `mauzi!` macro in a file which lives directly in the
  `src/` folder.
- Any project not living in `src/` won't work at all (e.g. examples in
  `examples/`)

[3]: https://github.com/rust-lang/rust/issues/38546


---

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

Mauzi
=====

Library to help with internationalization. WIP. Nightly only: uses `proc_macros`.


## Status

This library is very young and very unfinished. Please note that I don't know
anything about internationalization and that I never used an i18n library
before. I developed this library because I needed something like this for a
web project of mine. This web project will drive the development of this
library: experience from using the library in a real project helps a lot to
shape the API of this library.


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
[2]: https://github.com/rust-
lang/rust/issues/38356#issuecomment-323734922


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

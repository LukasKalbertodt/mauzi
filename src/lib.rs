//! **TODO:** add documentation
#![feature(proc_macro)]

extern crate mauzi_macros;


// Currently, proc-macros can't be defined in a crate together with non-proc-
// macros things. Thus the `mauzi!` macro is defined in the seperate crate
// `mauzi_macros`. But since `mauzi_macros` depends on types like `Locale`, and
// because we can't have circular crate dependencies, we put all the normal
// stuff into `mauzi_runtime`.
//
// In this main crate, we just reexport everything from those crates.
pub use mauzi_macros::mauzi;

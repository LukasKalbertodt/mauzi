#![feature(proc_macro)]

extern crate dicti_macros;

pub use dicti_macros::dict;



#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Locale {
    De(DeRegion),
    En(EnRegion),
}

impl Locale {
    pub fn de() -> Self {
        Locale::De(DeRegion::None)
    }

    pub fn en() -> Self {
        Locale::En(EnRegion::None)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DeRegion { None, De, Ch, At }
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum EnRegion { None, Us, Gb }


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

    pub fn from_variant_str(s: &str) -> Option<Self> {
        match s {
            "De" => Some(Self::de()),
            "En" => Some(Self::en()),
            _ => None,
        }
    }

    pub fn with_region_variant_str(&self, s: &str) -> Option<Self> {
        use self::Locale::*;

        macro_rules! make_match {
            (
                $(
                    $lang:ident, $region:ident, $region_ty:ident;
                )*
            ) => {
                match (*self, s) {
                    $(
                        ($lang(_), stringify!($region)) => Some(Locale::$lang($region_ty::$region)),
                    )*
                    _ => None,
                }
            }
        }

        make_match! {
            De, At, DeRegion;
            De, Ch, DeRegion;
            De, De, DeRegion;
            En, Gb, EnRegion;
            En, Us, EnRegion;
        }
    }

    pub fn region_type_str(&self) -> &'static str {
        use self::Locale::*;

        match *self {
            De(_) => "DeRegion",
            En(_) => "EnRegion",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DeRegion { None, At, Ch, De }
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum EnRegion { None, Gb, Us }

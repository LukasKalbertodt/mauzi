
#[macro_export]
macro_rules! dict {
    (
        $(
            $fn_name:ident
            ( $($arg_name:ident : $arg_type:ty),* )
            {
                $(
                    $dic:ident
                    =>
                    $(: $fragment:expr ;)+
                )+
            }
        )*
    ) => {
        pub struct Dict {
            loc: $crate::Loc,
        }

        impl Dict {
            pub fn new(loc: $crate::Loc) -> Self {
                Self { loc }
            }

            $(
                pub fn $fn_name
                    (&self, $($arg_name : $arg_type),* )
                    -> String
                {
                    use ::std::fmt::Write;

                    let mut buffer = String::new();
                    match self.loc {
                        $(
                            $crate::Loc::$dic => {
                                $(
                                    write!(buffer, "{}", $fragment).unwrap();
                                )+
                            }
                        )+
                        ref loc => panic!("missing dict entry for {:?}", loc),
                    }

                    buffer
                }
            )*
        }
    }
}

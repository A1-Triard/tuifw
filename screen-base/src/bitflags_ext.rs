macro_rules! bitflags_display {
    (
        $vis:vis struct $flags:ident : $ty:ty {
            $($(
                $name:ident = $value:expr
            ),+ $(,)?)?
        }
    ) => {
        bitflags! {
            #[derive(Default)]
            $vis struct $flags: $ty {
                $(const $name = $value;)*
            }
        }

        impl ::core::fmt::Display for $flags {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                let mut start = true;
                $(
                    if self.contains($flags::$name) {
                        if start {
                            #[allow(unused_assignments)]
                            start = false;
                        } else {
                            write!(f, " ")?;
                        }
                        write!(f, stringify!($name))?;
                    }
                )*
                Ok(())
            }
        }
        
        impl ::core::str::FromStr for $flags {
            type Err = ();
        
            fn from_str(s: &str) -> Result<$flags, Self::Err> {
                let mut flags = $flags::empty();
                for f in s.split(char::is_whitespace) {
                    match f {
                        "" => { },
                        $(
                            stringify!($name) => flags |= $flags::$name,
                        )*
                        _ => return Err(())
                    }
                }
                Ok(flags)
            }
        }
    }
}

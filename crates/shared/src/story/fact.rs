//! `fact!`, which declares a story fact: an enum of values, `ALL` of them in
//! order, and the `lang::Selector` that names each value in the language files
//! (`<fact>-<key>`). Many facts are plain lists of values, and writing the same
//! three blocks for each by hand would hide what the values are.
//!
//! Rust note: `macro_rules!` writes code from a pattern at compile time. This
//! one only expands to an ordinary enum and two `impl` blocks, so reading the
//! values written where it is used tells you everything about the type.

/// `fact! { /// docs  Name as "fact-key" { Value = "value-key", ... } }`
macro_rules! fact {
    (
        $(#[$meta:meta])*
        $name:ident as $fact:literal {
            $( $(#[$value_meta:meta])* $value:ident = $key:literal, )+
        }
    ) => {
        $(#[$meta])*
        #[derive(
            Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash,
            serde::Serialize, serde::Deserialize,
        )]
        pub enum $name {
            $( $(#[$value_meta])* $value, )+
            // New values go at the end: messages carry one by its place.
        }

        impl $name {
            /// Every value, in order.
            pub const ALL: &'static [Self] = &[ $( Self::$value, )+ ];
        }

        impl crate::lang::Selector for $name {
            fn fact(&self) -> &'static str {
                $fact
            }

            fn key(&self) -> &'static str {
                match self {
                    $( Self::$value => $key, )+
                }
            }
        }
    };
}

pub(super) use fact;

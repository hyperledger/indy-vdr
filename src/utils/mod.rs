pub use std::sync::atomic::{AtomicUsize, Ordering};

#[macro_use]
pub mod qualifier;

pub mod base58;
pub mod crypto;
pub mod environment;
pub mod error;
pub mod hash;
pub mod merkletree;
pub mod validation;

#[macro_use]
#[allow(unused_macros)]
pub mod test;

pub type HandleType = usize;

#[macro_export]
macro_rules! new_handle_type (($newtype:ident, $counter:ident) => (

    lazy_static! {
        static ref $counter: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub struct $newtype(pub usize);

    impl $newtype {
        pub fn invalid() -> $newtype {
            $newtype(0)
        }
        pub fn next() -> $newtype {
            $newtype($counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1)
        }
        pub fn value(&self) -> usize {
            return self.0
        }
    }

    impl std::fmt::Display for $newtype {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}({})", stringify!($newtype), self.0)
        }
    }
));

#[macro_use]
mod macros;
#[macro_use]
pub mod qualifier;

pub mod base58;
pub mod crypto;
pub mod environment;
pub mod hash;
pub mod signature;
pub mod validation;

#[macro_use]
#[allow(unused_macros)]
pub mod test;

#[macro_export]
macro_rules! new_handle_type (($newtype:ident, $counter:ident) => (

    lazy_static! {
        static ref $counter: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub struct $newtype(pub usize);

    impl $newtype {
        #[allow(dead_code)]
        pub fn invalid() -> $newtype {
            $newtype(0)
        }
        #[allow(dead_code)]
        pub fn next() -> $newtype {
            $newtype($counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1)
        }
    }

    impl std::fmt::Display for $newtype {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}({})", stringify!($newtype), self.0)
        }
    }

    impl std::ops::Deref for $newtype {
        type Target = usize;
        fn deref(&self) -> &usize {
            &self.0
        }
    }
));

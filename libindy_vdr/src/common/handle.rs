pub trait ResourceHandle: Copy + Ord + From<i64> {
    fn invalid() -> Self {
        Self::from(0)
    }

    fn next() -> Self;
}

#[cfg(feature = "ffi")]
/// Derive a new handle type having an atomically increasing sequence number
macro_rules! impl_sequence_handle (($newtype:ident, $counter:ident) => (
    static $counter: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(0);

    impl $crate::common::handle::ResourceHandle for $newtype {
        fn next() -> $newtype {
            $newtype($counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1)
        }
    }

    impl std::fmt::Display for $newtype {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}({})", stringify!($newtype), self.0)
        }
    }

    impl std::ops::Deref for $newtype {
        type Target = i64;
        fn deref(&self) -> &i64 {
            &self.0
        }
    }

    impl From<i64> for $newtype {
        fn from(val: i64) -> Self {
            Self(val)
        }
    }

    impl PartialEq<i64> for $newtype {
        fn eq(&self, other: &i64) -> bool {
            self.0 == *other
        }
    }
));

//#![allow(dead_code, unused_macros)]
pub mod pool;
pub mod crypto;
pub mod constants;
pub mod helpers;
pub mod fixtures;

macro_rules! inject_dependencies {
    () => {
        extern crate indy_vdr;
        extern crate ursa;
        #[allow(unused_imports)]
        #[macro_use]
        extern crate serde_json;
        #[macro_use]
        extern crate rstest;
    }
}

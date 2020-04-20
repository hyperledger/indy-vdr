#![allow(dead_code, unused_macros)]
pub mod constants;
pub mod crypto;
pub mod fixtures;
pub mod helpers;
pub mod pool;

macro_rules! inject_dependencies {
    () => {
        extern crate indy_vdr;
        extern crate ursa;
        #[allow(unused_imports)]
        #[macro_use]
        extern crate serde_json;
        #[macro_use]
        extern crate rstest;
        #[macro_use]
        extern crate lazy_static;
    };
}

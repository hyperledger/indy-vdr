#![cfg_attr(feature = "fatal_warnings", deny(warnings))]
#![allow(dead_code)]

extern crate failure;
// #[macro_use]
extern crate failure_derive;

#[macro_use]
extern crate lazy_static;

extern crate named_type;
#[macro_use]
extern crate named_type_derive;

#[macro_use]
extern crate log;
extern crate log_derive;

extern crate rmp_serde;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate base64;
extern crate byteorder;
extern crate hex;
extern crate rand;
extern crate rlp;
extern crate sha2;
extern crate sha3;
extern crate time;
extern crate ursa;

macro_rules! unwrap_opt_or_return {
    ($opt:expr, $err:expr) => {
        match $opt {
            Some(val) => val,
            None => return $err,
        };
    };
}

macro_rules! unwrap_or_return {
    ($result:expr, $err:expr) => {
        match $result {
            Ok(res) => res,
            Err(_) => return $err,
        };
    };
}

pub mod api;
mod domain;
mod services;
mod utils;

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn dummy() {
        assert!(true, "Dummy check!");
    }
}

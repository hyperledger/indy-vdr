use indy_blssignatures::Generator;
use once_cell::sync::Lazy;

use crate::pool::ProtocolVersion;

pub const DEFAULT_ACK_TIMEOUT: i64 = 5;
pub const DEFAULT_REPLY_TIMEOUT: i64 = 30;
pub const DEFAULT_CONN_ACTIVE_TIMEOUT: i64 = 5;
pub const DEFAULT_CONN_REQUEST_LIMIT: usize = 10;
pub const DEFAULT_REQUEST_READ_NODES: usize = 2;
pub const DEFAULT_FRESHNESS_TIMEOUT: u64 = 300;
pub const DEFAULT_PROTOCOL_VERSION: ProtocolVersion = ProtocolVersion::Node1_4;

pub static DEFAULT_GENERATOR: Lazy<Generator> = Lazy::new(|| {
    Generator::from_bytes(
        &hex::decode(
            "16cb6e1f1b7803f30ab2c661196fe199af17d8ed193d98a3d0fa17638a3a1b831df541918f0e5acd\
            0576998bfdb839318349b8acbb4106fe93e6a3d35a3f008107e2c4a7c9a5049f2cc9f9d7ced5049f\
            4336f67843c5dc32ad940e397e252df7176a8f76fd15d536bc8d294ac7040f6cc8d560dad13de88c\
            3dfa7260ec363452",
        )
        .unwrap(),
    )
    .unwrap()
});

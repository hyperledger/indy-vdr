use std::fmt;

pub const DEFAULT_PROTOCOL_VERSION: ProtocolVersion = ProtocolVersion::Node1_4;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ProtocolVersion {
    Node1_3 = 1,
    Node1_4 = 2,
}

impl ProtocolVersion {
    pub fn display_version(&self) -> String {
        match self {
            Self::Node1_3 => "1.3".to_owned(),
            Self::Node1_4 => "1.4".to_owned(),
        }
    }

    pub fn to_id(&self) -> usize {
        *self as usize
    }
}

impl PartialEq<usize> for ProtocolVersion {
    fn eq(&self, other: &usize) -> bool {
        (*self as usize) == *other
    }
}

impl Default for ProtocolVersion {
    fn default() -> Self {
        DEFAULT_PROTOCOL_VERSION
    }
}

impl fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.display_version())
    }
}

//! Retail 12.1.5 legacy PerformanceBar protocol input.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProtocolCode {
    Unsupported,
    One,
    Two,
}

impl ProtocolCode {
    pub fn value(self) -> u8 {
        match self {
            Self::Unsupported => 0,
            Self::One => 1,
            Self::Two => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PerformanceProtocolTypes {
    pub home: ProtocolCode,
    pub world: ProtocolCode,
}

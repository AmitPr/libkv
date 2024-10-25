use std::{convert::Infallible, fmt::Display, str::FromStr};

use super::serialization::{Decodable, Encodable, Encoding};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayEncoding;
impl Encoding for DisplayEncoding {
    type EncodeError = Infallible;
    type DecodeError = Infallible;
}

impl<T: ToString + FromStr> Encodable<DisplayEncoding> for T {
    fn encode(&self) -> Result<Vec<u8>, Infallible> {
        Ok(self.to_string().into_bytes())
    }
}

impl<T: Display + FromStr<Err: Into<Box<dyn std::error::Error>>>> Decodable<DisplayEncoding> for T {
    fn decode(bytes: &[u8]) -> Result<Self, Infallible> {
        let s = std::str::from_utf8(bytes).unwrap();
        Ok(Self::from_str(s).unwrap_or_else(|_| panic!("Failed to parse {}", s)))
    }
}

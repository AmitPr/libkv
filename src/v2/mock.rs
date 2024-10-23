use std::{convert::Infallible, fmt::Display, str::FromStr};

use super::serialization::{Decodable, Encodable, Encoding};

pub struct DisplayEncoding;
impl Encoding for DisplayEncoding {
    type EncodeError = Infallible;
    type DecodeError = Box<dyn std::error::Error>;
}

impl<T: ToString + FromStr> Encodable<DisplayEncoding> for T {
    fn encode(&self) -> Result<Vec<u8>, Infallible> {
        Ok(self.to_string().into_bytes())
    }
}

impl<T: Display + FromStr<Err: Into<Box<dyn std::error::Error>>>> Decodable<DisplayEncoding> for T {
    fn decode(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let s = std::str::from_utf8(bytes).unwrap();
        Self::from_str(s).map_err(|e| e.into())
    }
}

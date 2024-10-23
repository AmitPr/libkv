use std::{fmt::Display, str::FromStr};

trait Encoding {
    type EncodeError;
    type DecodeError;
}

trait Encodable<E: Encoding> {
    fn encode(&self) -> Result<Vec<u8>, E::EncodeError>;
}

trait Decodable<E: Encoding>: Sized {
    fn decode(bytes: &[u8]) -> Result<Self, E::DecodeError>;
}

struct DisplayEncoding<T: Display + FromStr>(std::marker::PhantomData<T>);
impl<T: Display + FromStr<Err = E>, E> Encoding for DisplayEncoding<T> {
    type EncodeError = std::fmt::Error;
    type DecodeError = E;
}

impl<T: Display + FromStr> Encodable<DisplayEncoding<T>> for T {
    fn encode(&self) -> Result<Vec<u8>, std::fmt::Error> {
        Ok(format!("{}", self).into_bytes())
    }
}

impl<T: Display + FromStr> Decodable<DisplayEncoding<T>> for T {
    fn decode(bytes: &[u8]) -> Result<Self, <DisplayEncoding<T> as Encoding>::DecodeError> {
        let s = std::str::from_utf8(bytes).unwrap();
        Self::from_str(s)
    }
}

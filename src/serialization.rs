pub trait Encoding {
    type EncodeError;
    type DecodeError;
}

pub trait Encodable<E: Encoding> {
    fn encode(&self) -> Result<Vec<u8>, E::EncodeError>;
}

pub trait Decodable<E: Encoding>: Sized {
    fn decode(bytes: &[u8]) -> Result<Self, E::DecodeError>;
}

use crate::serialization::Encoding;

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum KeySerializeError {}

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum KeyDeserializeError {
    #[error("Not enough bytes to decode key: expected {0}, got {1}")]
    NotEnoughBytes(usize, usize),
    #[error("Invalid key length: expected {0}, got {1}")]
    InvalidLength(usize, usize),
    #[error("Error decoding UTF8 key: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum StorageError<Enc: Encoding> {
    #[error("Error serializing key: {0}")]
    KeySerialize(#[from] KeySerializeError),
    #[error("Error deserializing key: {0}")]
    KeyDeserialize(#[from] KeyDeserializeError),
    #[error("Error serializing value: {0}")]
    ValueSerialize(Enc::EncodeError),
    #[error("Error deserializing value: {0}")]
    ValueDeserialize(Enc::DecodeError),
}

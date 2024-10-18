use num_traits::ToBytes;

pub(crate) fn to_length_prefixed_bytes(data: impl AsRef<[u8]>) -> Vec<u8> {
    let data = data.as_ref();
    let mut buf = Vec::with_capacity(std::mem::size_of::<usize>() + data.len());
    buf.extend_from_slice(&data.len().to_be_bytes());
    buf.extend_from_slice(data);
    buf
}

pub(crate) fn from_length_prefixed_bytes(bytes: &mut &[u8]) -> Option<Vec<u8>> {
    let first = bytes.first_chunk::<{ usize::BITS as usize / 8 }>()?;
    let len = usize::from_be_bytes(*first);
    *bytes = &bytes[std::mem::size_of::<usize>()..];
    if bytes.len() < len {
        return None;
    }
    Some(bytes.to_vec())
}

pub(crate) fn to_lex_bytes<N>(num: N) -> Vec<u8>
where
    N: ToBytes<Bytes: AsRef<[u8]>>,
{
    num.to_be_bytes().as_ref().to_vec()
}

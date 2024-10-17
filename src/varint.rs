use num_traits::{FromPrimitive, PrimInt, Unsigned};

pub(crate) fn to_varint<N>(mut n: N) -> Vec<u8>
where
    N: Unsigned + PrimInt,
{
    // Varint encoding:
    // First bit: 0 if no more bytes follow, 1 if more bytes follow.
    // Next 7 bits: 7 bits of the number.

    // Total Size: at most ceil(bits(N)/7) bytes
    let varint_bytes = std::mem::size_of::<N>() * 8 / 7 + 1;
    let mut buf = Vec::with_capacity(varint_bytes);
    for _ in 0..varint_bytes {
        // Get next 7 bits of n, safe because we're going byte by byte
        let byte = unsafe { *(&mut n as *mut N as *mut u8) };
        let mut byte = (byte & 0x7f) | 0x80;
        n = n >> 7;
        buf.push(byte);
        // if n.is_zero() {
        //     break;
        // }
    }
    // unset last byte's high bit
    buf[0] &= 0x7f;
    buf.reverse();
    buf
}

pub(crate) fn from_varint<N>(buf: &[u8]) -> N
where
    N: Unsigned + PrimInt + FromPrimitive,
{
    let mut n = N::zero();
    for byte in buf {
        let mut byte = *byte;
        n = (n << 7) | N::from_u8(byte & 0x7f).unwrap();
        if byte & 0x80 == 0 {
            break;
        }
    }
    n
}

pub fn to_varint_u128(n: u128) -> Vec<u8> {
    to_varint(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_encode_decode {
        ($n:expr, $expected:expr) => {
            let buf = to_varint($n);
            assert_eq!(buf, $expected);
            let decoded = from_varint(&buf);
            assert_eq!($n, decoded);
        };
    }

    #[test]
    fn test_to_varint_u8() {
        test_encode_decode!(0u8, vec![0]);
        test_encode_decode!(1u8, vec![1]);
        test_encode_decode!(127u8, vec![127]);
        test_encode_decode!(128u8, vec![128, 1]);
        test_encode_decode!(u8::MAX, vec![255, 1]);
        assert_eq!(to_varint(u8::MAX).len(), u8::BITS as usize / 7 + 1);
    }

    #[test]
    fn test_to_varint_u16() {
        test_encode_decode!(0u16, vec![0]);
        test_encode_decode!(1u16, vec![1]);
        test_encode_decode!(127u16, vec![127]);
        test_encode_decode!(128u16, vec![128, 1]);
        test_encode_decode!(255u16, vec![255, 1]);
        test_encode_decode!(256u16, vec![128, 2]);
        test_encode_decode!(0x2000u16, vec![128, 64]);
        test_encode_decode!(u16::MAX, vec![255, 255, 3]);
        assert_eq!(to_varint(u16::MAX).len(), u16::BITS as usize / 7 + 1);
    }

    #[test]
    fn test_to_varint_u32() {
        test_encode_decode!(0u32, vec![0]);
        test_encode_decode!(1u32, vec![1]);
        test_encode_decode!(127u32, vec![127]);
        test_encode_decode!(128u32, vec![128, 1]);
        test_encode_decode!(255u32, vec![255, 1]);
        test_encode_decode!(256u32, vec![128, 2]);
        test_encode_decode!(0x2000u32, vec![128, 64]);
        test_encode_decode!(0xFFFFu32, vec![255, 255, 3]);
        test_encode_decode!(0x100000u32, vec![128, 128, 64]);
        test_encode_decode!(u32::MAX, vec![255, 255, 255, 255, 15]);
        assert_eq!(to_varint(u32::MAX).len(), u32::BITS as usize / 7 + 1);
    }

    #[test]
    fn test_to_varint_u64() {
        test_encode_decode!(0u64, vec![0]);
        test_encode_decode!(1u64, vec![1]);
        test_encode_decode!(127u64, vec![127]);
        test_encode_decode!(128u64, vec![128, 1]);
        test_encode_decode!(255u64, vec![255, 1]);
        test_encode_decode!(256u64, vec![128, 2]);
        test_encode_decode!(0x2000u64, vec![128, 64]);
        test_encode_decode!(0xFFFFu64, vec![255, 255, 3]);
        test_encode_decode!(0x100000u64, vec![128, 128, 64]);
        test_encode_decode!(0xFFFFFFFFu64, vec![255, 255, 255, 255, 15]);
        test_encode_decode!(0x100000000u64, vec![128, 128, 128, 128, 16]);
        test_encode_decode!(
            u64::MAX,
            vec![255, 255, 255, 255, 255, 255, 255, 255, 255, 1]
        );

        assert_eq!(to_varint(u64::MAX).len(), u64::BITS as usize / 7 + 1);
    }

    #[test]
    fn test_to_varint_u128() {
        test_encode_decode!(0u128, vec![0]);
        test_encode_decode!(1u128, vec![1]);
        test_encode_decode!(127u128, vec![127]);
        test_encode_decode!(128u128, vec![128, 1]);
        test_encode_decode!(255u128, vec![255, 1]);
        test_encode_decode!(256u128, vec![128, 2]);
        test_encode_decode!(0x2000u128, vec![128, 64]);
        test_encode_decode!(0xFFFFu128, vec![255, 255, 3]);
        test_encode_decode!(0x100000u128, vec![128, 128, 64]);
        test_encode_decode!(0xFFFFFFFFu128, vec![255, 255, 255, 255, 15]);
        test_encode_decode!(0x100000000u128, vec![128, 128, 128, 128, 16]);
        test_encode_decode!(
            0xFFFFFFFFFFFFFFFFu128,
            vec![255, 255, 255, 255, 255, 255, 255, 255, 255, 1]
        );
        test_encode_decode!(
            0x100000000000000000u128,
            vec![128, 128, 128, 128, 128, 128, 128, 128, 128, 0x20]
        );
        test_encode_decode!(
            u128::MAX,
            vec![
                255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
                255, 255, 3
            ]
        );

        assert_eq!(to_varint(u128::MAX).len(), u128::BITS as usize / 7 + 1);
    }

    #[test]
    fn test_lexicographic_order() {
        let mut last = vec![0];
        for i in 1..=u32::MAX {
            let buf = to_varint(i);
            assert!(buf > last);
            last = buf;
        }
    }

    #[test]
    fn debug_print_reprs() {
        let n = 0x0123_4567_89AB_CDEFu128;
        let buf = to_varint(n);
        assert_eq!(from_varint::<u128>(&buf), n);
        let chars = buf
            .iter()
            .map(|b| format!("{:08b}", b))
            .collect::<String>()
            .chars()
            .collect::<Vec<_>>();
        let hex_buf = chars
            .chunks(8)
            .map(|c| format!("{} {}", c[0], c[1..].iter().collect::<String>()))
            .collect::<Vec<_>>()
            .join(" ");
        let n_bin = n
            .to_be_bytes()
            .iter()
            .map(|b| format!("{:08b}", b))
            .collect::<String>();
        let n_buf = n_bin
            .chars()
            .collect::<Vec<_>>()
            .chunks(7)
            .map(|c| c.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join("   ");
        println!("  {}\n{}", n_buf, hex_buf);
    }
}

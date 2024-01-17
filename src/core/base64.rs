use bitstream_io::{BigEndian, BitWrite, BitWriter};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DecodeError {
    #[error("invalid character {0}")]
    InvalidCharacter(u8),
}

/// Custom base64 implementation, 6-bits aligned, no padding,
/// using the URL Safe Base64 dictionary.
pub fn decode(s: &str) -> Result<Vec<u8>, DecodeError> {
    // output buffer should not be larger than input string, so we pre-allocate enough bytes as to avoid realloc
    // which is slow, and could cause allocation of a bigger capacity than needed (x2 or more)
    let mut buffer = Vec::with_capacity(s.len());
    let mut bw = BitWriter::endian(&mut buffer, BigEndian);

    // write 6 bits for every decoded character
    for b in s.bytes() {
        let value = base64_value(b).ok_or(DecodeError::InvalidCharacter(b))?;
        bw.write(6, value).expect("write into vec should not fail");
    }

    // write remaining value if we're not 8-bit aligned at this point
    let (n, value) = bw.into_unwritten();
    if n > 0 {
        let n = 8 - n;
        let value = value << n;
        buffer.push(value);
    }

    Ok(buffer)
}

fn base64_value(b: u8) -> Option<u8> {
    match b {
        b'A'..=b'Z' => Some(b - b'A'),
        b'a'..=b'z' => Some(b - b'a' + 26),
        b'0'..=b'9' => Some(b - b'0' + 52),
        b'-' => Some(62),
        b'_' => Some(63),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(b'A' => Some(0))]
    #[test_case(b'Z' => Some(25))]
    #[test_case(b'a' => Some(26))]
    #[test_case(b'z' => Some(51))]
    #[test_case(b'0' => Some(52))]
    #[test_case(b'9' => Some(61))]
    #[test_case(b'=' => None ; "equal")]
    #[test_case(b'#' => None ; "sharp")]
    fn base64_value_map(b: u8) -> Option<u8> {
        base64_value(b)
    }

    #[test_case("DBABM" => vec![12, 16, 1, 48] ; "simple header")]
    #[test_case("" => is empty ; "empty string")]
    fn test_decode_base64(s: &str) -> Vec<u8> {
        decode(s).unwrap()
    }

    #[test_case("===" => matches DecodeError::InvalidCharacter(_) ; "equal signs")]
    #[test_case("   " => matches DecodeError::InvalidCharacter(_) ; "whitespaces")]
    fn error(s: &str) -> DecodeError {
        decode(s).unwrap_err()
    }
}

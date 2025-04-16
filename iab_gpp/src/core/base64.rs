use std::io;
use std::io::Read;
use thiserror::Error;

/// The error type that describes failures to decode Base64 encoded strings.
#[derive(Error, Debug)]
pub enum DecodeError {
    /// An invalid byte was found in the input. The offset and offending byte are provided.
    #[error("invalid byte {1} at offset {0}")]
    InvalidByte(usize, u8),
}

pub struct Base64Reader<R>
where
    R: Read,
{
    inner_reader: R,
    inner_reader_pos: usize,
    partial_byte: u8,
    partial_byte_index: usize,
}

impl<R> Base64Reader<R>
where
    R: Read,
{
    pub fn new(r: R) -> Self {
        Self {
            inner_reader: r,
            inner_reader_pos: 0,
            partial_byte: 0,
            partial_byte_index: 0,
        }
    }
}

impl<R> Read for Base64Reader<R>
where
    R: Read,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut bytes_written = 0;
        let mut bit_buf = [0];
        let mut output_byte_index = 0;

        'bytes: for b in buf.iter_mut() {
            output_byte_index = 0;

            // 1. write any remaining bits into output
            if self.partial_byte_index != 0 {
                let copied_bits = copy_bits(self.partial_byte, self.partial_byte_index, b, 0);

                // 2. update partial byte index, if >= 6, all is written, go back to 0
                self.partial_byte_index += copied_bits;
                if self.partial_byte_index >= 6 {
                    self.partial_byte_index = 0;
                }

                // 3. update output byte index, if we've completely written a byte, skip to next
                output_byte_index += copied_bits;
                if output_byte_index >= 8 {
                    bytes_written += 1;
                    continue;
                }
            }

            while output_byte_index < 8 {
                // 4. read next byte from input
                let read = self.inner_reader.read(&mut bit_buf)?;
                if read == 0 {
                    break 'bytes;
                }
                self.inner_reader_pos += read;

                // 5. decode into 6 bits value
                let val = bit_buf[0];
                self.partial_byte = base64_value(val).ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        DecodeError::InvalidByte(self.inner_reader_pos - 1, val),
                    )
                })?;

                // 6. copy bits to output
                let copied_bits = copy_bits(self.partial_byte, 0, b, output_byte_index);

                self.partial_byte_index += copied_bits;
                if self.partial_byte_index >= 6 {
                    self.partial_byte_index = 0;
                }
                output_byte_index += copied_bits;
                if output_byte_index >= 8 {
                    bytes_written += 1;
                }
            }
        }

        // 7. pad if needed
        if output_byte_index > 0 && output_byte_index < 8 {
            bytes_written += 1;
        }

        Ok(bytes_written)
    }
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

fn copy_bits(input: u8, input_offset: usize, output: &mut u8, output_offset: usize) -> usize {
    let input_size = 6 - input_offset;
    let mut copied_bits = 0;
    let mut current_output_offset = 7 - output_offset;

    for i in (0..input_size).rev() {
        let bit = (input >> i) & 1;
        let bit = bit << current_output_offset;

        *output |= bit;
        copied_bits += 1;

        if current_output_offset == 0 {
            break;
        }
        current_output_offset -= 1;
    }

    copied_bits
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
    fn test_base64_reader(s: &str) -> Vec<u8> {
        let mut r = Base64Reader::new(s.as_bytes());
        let mut buf = vec![0; 32];
        let n = r.read(&mut buf).unwrap();
        buf.truncate(n);

        buf
    }

    #[test_case("===" => matches DecodeError::InvalidByte(0, b'=') ; "equal signs")]
    #[test_case("a  " => matches DecodeError::InvalidByte(1, b' ') ; "whitespaces")]
    fn test_base64_reader_error(s: &str) -> DecodeError {
        let mut r = Base64Reader::new(s.as_bytes());
        let mut buf = vec![0; 32];
        r.read(&mut buf).unwrap_err().downcast().unwrap()
    }
}

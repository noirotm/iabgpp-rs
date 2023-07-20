use crate::core::fibonacci::fibonacci_iterator;
use base64::{DecodeError, Engine};
use bitstream_io::{BigEndian, BitRead, BitReader, Numeric};
use std::io;
use std::iter::repeat_with;

mod fibonacci;

pub trait DecodeExt {
    fn decode_base64_url(&self) -> Result<Vec<u8>, DecodeError>;
}

impl DecodeExt for &str {
    fn decode_base64_url(&self) -> Result<Vec<u8>, DecodeError> {
        base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(self)
    }
}

pub struct DataReader<'a> {
    bit_reader: BitReader<&'a [u8], BigEndian>,
}

impl<'a> DataReader<'a> {
    pub fn new(bytes: &'a [u8]) -> DataReader {
        DataReader {
            bit_reader: BitReader::endian(bytes, BigEndian),
        }
    }

    pub fn read_bool(&mut self) -> io::Result<bool> {
        self.bit_reader.read_bit()
    }

    pub fn read_fixed_integer<N: Numeric>(&mut self, bits: u32) -> io::Result<N> {
        self.bit_reader.read(bits)
    }

    pub fn read_fibonacci_integer(&mut self) -> io::Result<u64> {
        let mut fib = fibonacci_iterator();
        let mut total = 0;
        let mut last_bit = false;

        loop {
            let bit = self.read_bool()?;

            // two consecutive 1's signal the end of the value
            if last_bit && bit {
                break;
            }

            let fib_value = fib.next().expect("next fibonacci number");
            if bit {
                total += fib_value;
            }
            last_bit = bit;
        }

        Ok(total)
    }

    pub fn read_string(&mut self, chars: usize) -> io::Result<String> {
        repeat_with(|| self.read_fixed_integer::<u8>(6))
            .take(chars)
            .map(|r| r.map(|n| (n + 65) as char))
            .collect::<Result<String, _>>()
    }

    pub fn read_datetime_as_unix_timestamp(&mut self) -> io::Result<i64> {
        Ok(self.read_fixed_integer::<i64>(36)? / 10) // seconds
    }

    pub fn read_fixed_bitfield(&mut self, bits: usize) -> io::Result<Vec<bool>> {
        repeat_with(|| self.read_bool())
            .take(bits)
            .collect::<Result<Vec<_>, _>>()
    }

    pub fn read_variable_bitfield(&mut self) -> io::Result<Vec<bool>> {
        let n = self.read_fixed_integer::<u16>(16)? as usize;
        repeat_with(|| self.read_bool())
            .take(n)
            .collect::<Result<Vec<_>, _>>()
    }

    pub fn read_integer_range(&mut self) -> io::Result<Vec<u64>> {
        let n = self.bit_reader.read::<u16>(12)? as usize;
        let mut range = vec![];

        for _ in 0..n {
            let is_group = self.read_bool()?;
            if is_group {
                let start = self.read_fixed_integer::<u64>(16)?;
                let end = self.read_fixed_integer::<u64>(16)?;

                for id in start..=end {
                    range.push(id);
                }
            } else {
                let id = self.read_fixed_integer::<u64>(16)?;
                range.push(id);
            }
        }

        Ok(range)
    }

    pub fn read_fibonacci_range(&mut self) -> io::Result<Vec<u64>> {
        let n = self.bit_reader.read::<u16>(12)? as usize;
        let mut range = vec![];
        let mut last_id = 0u64;

        for _ in 0..n {
            let is_group = self.read_bool()?;
            if is_group {
                let offset = self.read_fibonacci_integer()?;
                let count = self.read_fibonacci_integer()?;

                for id in (last_id + offset)..=(last_id + offset + count) {
                    range.push(id);
                    last_id = id;
                }
            } else {
                let id = self.read_fibonacci_integer()?;
                range.push(last_id + id);
                last_id = id;
            }
        }

        Ok(range)
    }

    pub fn read_optimized_range(&mut self) -> io::Result<Vec<u64>> {
        let is_fibo = self.read_bool()?;
        if is_fibo {
            self.read_fibonacci_range()
        } else {
            Ok(self
                .read_variable_bitfield()?
                .iter()
                .map(|&b| b as u64)
                .collect())
        }
    }

    pub fn read_optimized_integer_range(&mut self) -> io::Result<Vec<u64>> {
        let len = self.read_fixed_integer::<u16>(16)?;
        let is_int_range = self.read_bool()?;
        if is_int_range {
            self.read_integer_range()
        } else {
            Ok(self
                .read_fixed_bitfield(len as usize)?
                .iter()
                .map(|&b| b as u64)
                .collect())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Transform a string of literal binary digits into a vector of bytes.
    /// Zeroes will be appended to fill missing bits.
    fn b(s: &str) -> Vec<u8> {
        let chars = s
            .chars()
            .filter(|&c| c == '1' || c == '0')
            .collect::<Vec<_>>();
        chars
            .chunks(8)
            .map(|c| (8 - c.len(), String::from_iter(c)))
            .map(|(l, s)| u8::from_str_radix(&s, 2).map(|n| n << l))
            .collect::<Result<Vec<_>, _>>()
            .unwrap_or(vec![])
    }

    #[test]
    fn test_bytes() {
        assert_eq!(b("00000001 00000010 00000011"), vec![1, 2, 3]);
        assert_eq!(b("000000 010000 001000 000011"), vec![1, 2, 3]);
        assert_eq!(b("000000 010000 001000 000011 1000"), vec![1, 2, 3, 128]);
        assert_eq!(b("000000 010000 001000 000011 100"), vec![1, 2, 3, 128]);
    }

    #[test]
    fn test_read_int() {
        let test_cases = [(b("000101"), 6, 5), (b("101010"), 6, 42)];

        for (buf, bits, expected_value) in test_cases {
            let mut reader = DataReader::new(&buf);

            assert_eq!(
                reader.read_fixed_integer::<u32>(bits).unwrap(),
                expected_value
            );
        }
    }

    #[test]
    fn test_read_fibonacci() {
        let test_cases = [
            (b("11"), 1),
            (b("011"), 2),
            (b("0011"), 3),
            (b("1011"), 4),
            (b("00011"), 5),
            (b("10011"), 6),
            (b("01011"), 7),
        ];

        for (buf, expected_value) in test_cases {
            let mut reader = DataReader::new(&buf);

            assert_eq!(reader.read_fibonacci_integer().unwrap(), expected_value);
        }
    }

    #[test]
    fn test_read_string() {
        let test_cases = [(b("101010"), 1, "k"), (b("101010 101011"), 2, "kl")];

        for (buf, chars, expected_value) in test_cases {
            let mut reader = DataReader::new(&buf);

            assert_eq!(reader.read_string(chars).unwrap(), expected_value);
        }
    }

    #[test]
    fn test_read_datetime_as_unix_timestamp() {
        let test_cases = [(b("001111101100100110001110010001011101"), 1685434479)];

        for (buf, expected_value) in test_cases {
            let mut reader = DataReader::new(&buf);

            assert_eq!(
                reader.read_datetime_as_unix_timestamp().unwrap(),
                expected_value
            );
        }
    }

    #[test]
    fn test_read_fixed_bitfield() {
        let test_cases = [(b("10101"), 5, vec![true, false, true, false, true])];

        for (buf, bits, expected_value) in test_cases {
            let mut reader = DataReader::new(&buf);

            assert_eq!(reader.read_fixed_bitfield(bits).unwrap(), expected_value);
        }
    }

    #[test]
    fn test_read_variable_bitfield() {
        let test_cases = [(
            b("0000000000000101 10101"),
            vec![true, false, true, false, true],
        )];

        for (buf, expected_value) in test_cases {
            let mut reader = DataReader::new(&buf);

            assert_eq!(reader.read_variable_bitfield().unwrap(), expected_value);
        }
    }

    #[test]
    fn test_read_integer_range() {
        let test_cases = [(
            b("000000000010 0 0000000000000011 1 0000000000000101 0000000000001000"),
            vec![3, 5, 6, 7, 8],
        )];

        for (buf, expected_value) in test_cases {
            let mut reader = DataReader::new(&buf);

            assert_eq!(reader.read_integer_range().unwrap(), expected_value);
        }
    }

    #[test]
    fn test_read_fibonacci_range() {
        let test_cases = [
            (b("000000000010 0 0011 1 011 0011"), vec![3, 5, 6, 7, 8]),
            (b("000000000010 0 011 0 1011"), vec![2, 6]),
        ];

        for (buf, expected_value) in test_cases {
            let mut reader = DataReader::new(&buf);

            assert_eq!(reader.read_fibonacci_range().unwrap(), expected_value);
        }
    }

    #[test]
    fn test_read_optimized_range() {
        let test_cases = [
            (b("1 000000000010 0 0011 1 011 0011"), vec![3, 5, 6, 7, 8]),
            (b("0 0000000000000101 10101"), vec![1, 0, 1, 0, 1]),
        ];

        for (buf, expected_value) in test_cases {
            let mut reader = DataReader::new(&buf);

            assert_eq!(reader.read_optimized_range().unwrap(), expected_value);
        }
    }

    #[test]
    fn test_read_optimized_int_range() {
        let test_cases = [
            (b("0000000000000000 1 000000000010 0 0000000000000011 1 0000000000000101 0000000000001000"), vec![3, 5, 6, 7, 8]),
            (b("0000000000000101 0 10101"), vec![1, 0, 1, 0, 1]),
        ];

        for (buf, expected_value) in test_cases {
            let mut reader = DataReader::new(&buf);

            assert_eq!(
                reader.read_optimized_integer_range().unwrap(),
                expected_value
            );
        }
    }
}

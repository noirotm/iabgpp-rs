use crate::core::fibonacci::fibonacci_iterator;
use base64::{DecodeError, Engine};
use bitstream_io::{BigEndian, BitRead, BitReader, Numeric};
use num_iter::range_inclusive;
use num_traits::{CheckedAdd, Num, NumAssignOps, ToPrimitive};
use std::collections::BTreeSet;
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

pub trait FromDataReader: Sized {
    type Err;

    fn from_data_reader(r: &mut DataReader) -> Result<Self, Self::Err>;
}

pub struct DataReader<'a> {
    bit_reader: BitReader<&'a [u8], BigEndian>,
}

impl<'a> DataReader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bit_reader: BitReader::endian(bytes, BigEndian),
        }
    }

    pub fn parse<F>(&mut self) -> Result<F, <F as FromDataReader>::Err>
    where
        F: FromDataReader,
    {
        FromDataReader::from_data_reader(self)
    }

    pub fn read_bool(&mut self) -> io::Result<bool> {
        self.bit_reader.read_bit()
    }

    pub fn read_fixed_integer<N: Numeric>(&mut self, bits: u32) -> io::Result<N> {
        self.bit_reader.read(bits)
    }

    pub fn read_fibonacci_integer<T>(&mut self) -> io::Result<T>
    where
        T: CheckedAdd + Copy + Num + NumAssignOps,
    {
        let mut fib = fibonacci_iterator::<T>();
        let mut total = T::zero();
        let mut last_bit = false;

        loop {
            let bit = self.read_bool()?;

            // two consecutive 1's signal the end of the value
            if last_bit && bit {
                break;
            }

            let fib_value = fib.next().unwrap_or(T::zero());
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

    pub fn read_fixed_bitfield(&mut self, bits: usize) -> io::Result<BTreeSet<u16>> {
        let mut result = BTreeSet::new();
        for i in 1..=bits {
            let b = self.read_bool()?;
            if b {
                result.insert(i as u16);
            }
        }

        Ok(result)
    }

    pub fn read_variable_bitfield(&mut self) -> io::Result<BTreeSet<u16>> {
        let n = self.read_fixed_integer::<u16>(16)? as usize;
        self.read_fixed_bitfield(n)
    }

    pub fn read_integer_range(&mut self) -> io::Result<Vec<u16>> {
        let n = self.bit_reader.read::<u16>(12)? as usize;
        let mut range = vec![];

        for _ in 0..n {
            let is_group = self.read_bool()?;
            if is_group {
                let start = self.read_fixed_integer::<u16>(16)?;
                let end = self.read_fixed_integer::<u16>(16)?;

                for id in start..=end {
                    range.push(id);
                }
            } else {
                let id = self.read_fixed_integer::<u16>(16)?;
                range.push(id);
            }
        }

        Ok(range)
    }

    pub fn read_fibonacci_range<T>(&mut self) -> io::Result<Vec<T>>
    where
        T: CheckedAdd + Copy + Num + NumAssignOps + PartialOrd + ToPrimitive,
    {
        let n = self.bit_reader.read::<u16>(12)? as usize;
        let mut range = vec![];
        let mut last_id = T::zero();

        for _ in 0..n {
            let is_group = self.read_bool()?;
            if is_group {
                let offset = self.read_fibonacci_integer()?;
                let count = self.read_fibonacci_integer()?;

                for id in range_inclusive(last_id + offset, last_id + offset + count) {
                    range.push(id);
                    last_id = id;
                }
            } else {
                let id = self.read_fibonacci_integer::<T>()?;
                range.push(last_id + id);
                last_id = id;
            }
        }

        Ok(range)
    }

    pub fn read_optimized_range(&mut self) -> io::Result<BTreeSet<u16>> {
        let is_fibo = self.read_bool()?;
        if is_fibo {
            Ok(self.read_fibonacci_range::<u16>()?.into_iter().collect())
        } else {
            self.read_variable_bitfield()
        }
    }

    pub fn read_optimized_integer_range(&mut self) -> io::Result<BTreeSet<u16>> {
        let len = self.read_fixed_integer::<u16>(16)?;
        let is_int_range = self.read_bool()?;
        if is_int_range {
            self.read_integer_range().map(|r| r.into_iter().collect())
        } else {
            self.read_fixed_bitfield(len as usize)
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
    fn bytes() {
        assert_eq!(b("00000001 00000010 00000011"), vec![1, 2, 3]);
        assert_eq!(b("000000 010000 001000 000011"), vec![1, 2, 3]);
        assert_eq!(b("000000 010000 001000 000011 1000"), vec![1, 2, 3, 128]);
        assert_eq!(b("000000 010000 001000 000011 100"), vec![1, 2, 3, 128]);
    }

    #[test]
    fn read_int() {
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
    fn read_fibonacci() {
        let test_cases = [
            (b("11"), 1),
            (b("011"), 2),
            (b("0011"), 3),
            (b("1011"), 4),
            (b("00011"), 5),
            (b("10011"), 6),
            (b("01011"), 7),
            (b("0100000000001011"), 2), // overflow for u8, we ignore bits we can't encode
        ];

        for (buf, expected_value) in test_cases {
            let mut reader = DataReader::new(&buf);

            assert_eq!(
                reader.read_fibonacci_integer::<u8>().unwrap(),
                expected_value
            );
        }
    }

    #[test]
    fn read_string() {
        let test_cases = [(b("101010"), 1, "k"), (b("101010 101011"), 2, "kl")];

        for (buf, chars, expected_value) in test_cases {
            let mut reader = DataReader::new(&buf);

            assert_eq!(reader.read_string(chars).unwrap(), expected_value);
        }
    }

    #[test]
    fn read_datetime_as_unix_timestamp() {
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
    fn read_fixed_bitfield() {
        let test_cases = [(b("10101"), 5, BTreeSet::from_iter([1, 3, 5]))];

        for (buf, bits, expected_value) in test_cases {
            let mut reader = DataReader::new(&buf);

            assert_eq!(reader.read_fixed_bitfield(bits).unwrap(), expected_value);
        }
    }

    #[test]
    fn read_variable_bitfield() {
        let test_cases = [(b("0000000000000101 10101"), BTreeSet::from_iter([1, 3, 5]))];

        for (buf, expected_value) in test_cases {
            let mut reader = DataReader::new(&buf);

            assert_eq!(reader.read_variable_bitfield().unwrap(), expected_value);
        }
    }

    #[test]
    fn read_integer_range() {
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
    fn read_fibonacci_range() {
        let test_cases = [
            (b("000000000010 0 0011 1 011 0011"), vec![3, 5, 6, 7, 8]),
            (b("000000000010 0 011 0 1011"), vec![2, 6]),
        ];

        for (buf, expected_value) in test_cases {
            let mut reader = DataReader::new(&buf);

            assert_eq!(reader.read_fibonacci_range::<u8>().unwrap(), expected_value);
        }
    }

    #[test]
    fn read_optimized_range() {
        let test_cases = [
            (
                b("1 000000000010 0 0011 1 011 0011"),
                BTreeSet::from_iter([3, 5, 6, 7, 8]),
            ),
            (
                b("0 0000000000000101 10101"),
                BTreeSet::from_iter([1, 3, 5]),
            ),
        ];

        for (buf, expected_value) in test_cases {
            let mut reader = DataReader::new(&buf);

            assert_eq!(reader.read_optimized_range().unwrap(), expected_value);
        }
    }

    #[test]
    fn read_optimized_int_range() {
        let test_cases = [
            (
                b("0000000000000000 1 000000000010 0 0000000000000011 1 0000000000000101 0000000000001000"),
                BTreeSet::from_iter([3, 5, 6, 7, 8])
            ),
            (
                b("0000000000000101 0 10101"),
                BTreeSet::from_iter([1, 3, 5])
            ),
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

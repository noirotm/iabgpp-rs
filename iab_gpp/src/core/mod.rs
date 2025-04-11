use crate::core::fibonacci::fibonacci_iterator;
use base64::DecodeError;
use bitstream_io::{BigEndian, BitRead, BitReader, UnsignedInteger};
use num_iter::range_inclusive;
use num_traits::{CheckedAdd, Num, NumAssignOps, ToPrimitive};
use std::collections::BTreeSet;
use std::io;
use std::iter::repeat_with;

pub mod base64;
mod fibonacci;

pub trait DecodeExt {
    fn decode_base64_url(&self) -> Result<Vec<u8>, DecodeError>;
}

impl DecodeExt for &str {
    fn decode_base64_url(&self) -> Result<Vec<u8>, DecodeError> {
        base64::decode(self)
    }
}

pub trait FromDataReader: Sized {
    type Err;

    fn from_data_reader(r: &mut DataReader) -> Result<Self, Self::Err>;
}

impl FromDataReader for bool {
    type Err = io::Error;

    fn from_data_reader(r: &mut DataReader) -> Result<Self, Self::Err> {
        r.read_bool()
    }
}

impl FromDataReader for u8 {
    type Err = io::Error;

    fn from_data_reader(r: &mut DataReader) -> Result<Self, Self::Err> {
        r.read_fixed_integer(6)
    }
}

impl FromDataReader for u16 {
    type Err = io::Error;

    fn from_data_reader(r: &mut DataReader) -> Result<Self, Self::Err> {
        r.read_fixed_integer(12)
    }
}

pub struct DataReader<'a> {
    bit_reader: BitReader<&'a [u8], BigEndian>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct GenericRange<X, Y> {
    pub key: X,
    pub range_type: Y,
    pub ids: BTreeSet<u16>,
}

pub type Range = GenericRange<u8, u8>;

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

    pub fn read_fixed_integer<N: UnsignedInteger>(&mut self, bits: u32) -> io::Result<N> {
        self.bit_reader.read_unsigned_var(bits)
    }

    pub fn read_fibonacci_integer<T>(&mut self) -> io::Result<T>
    where
        T: CheckedAdd + Copy + Num + NumAssignOps,
    {
        let mut fib = fibonacci_iterator();
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

    pub fn read_datetime_as_unix_timestamp(&mut self) -> io::Result<u64> {
        Ok(self.read_fixed_integer::<u64>(36)? / 10) // seconds
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
        let n = self.read_fixed_integer::<u16>(12)? as usize;
        let mut range = vec![];

        for _ in 0..n {
            let is_group = self.read_bool()?;
            if is_group {
                let start = self.read_fixed_integer(16)?;
                let end = self.read_fixed_integer(16)?;

                for id in start..=end {
                    range.push(id);
                }
            } else {
                let id = self.read_fixed_integer(16)?;
                range.push(id);
            }
        }

        Ok(range)
    }

    pub fn read_fibonacci_range<T>(&mut self) -> io::Result<Vec<T>>
    where
        T: CheckedAdd + Copy + Num + NumAssignOps + PartialOrd + ToPrimitive,
    {
        let n = self.read_fixed_integer::<u16>(12)? as usize;
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
        let n = self.read_fixed_integer::<u16>(16)? as usize;
        let is_int_range = self.read_bool()?;
        if is_int_range {
            self.read_integer_range().map(|r| r.into_iter().collect())
        } else {
            self.read_fixed_bitfield(n)
        }
    }

    pub fn read_array_of_ranges(&mut self) -> io::Result<Vec<Range>> {
        let n = self.read_fixed_integer::<u16>(12)? as usize;
        repeat_with(|| {
            Ok(Range {
                key: self.read_fixed_integer(6)?,
                range_type: self.read_fixed_integer(2)?,
                ids: self.read_optimized_integer_range()?,
            })
        })
        .take(n)
        .collect()
    }

    pub fn read_n_array_of_ranges<X, Y>(
        &mut self,
        x: u32,
        y: u32,
    ) -> io::Result<Vec<GenericRange<X, Y>>>
    where
        X: UnsignedInteger,
        Y: UnsignedInteger,
    {
        let n = self.read_fixed_integer::<u16>(12)? as usize;
        repeat_with(|| {
            Ok(GenericRange {
                key: self.read_fixed_integer::<X>(x)?,
                range_type: self.read_fixed_integer::<Y>(y)?,
                ids: self.read_optimized_range()?,
            })
        })
        .take(n)
        .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

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

    #[test_case("00000001 00000010 00000011" => vec![1, 2, 3])]
    #[test_case("000000 010000 001000 000011" => vec![1, 2, 3])]
    #[test_case("000000 010000 001000 000011 1000" => vec![1, 2, 3, 128])]
    #[test_case("000000 010000 001000 000011 100" => vec![1, 2, 3, 128])]
    #[test_case("000000 010000 001000 000011 1001" => vec![1, 2, 3, 144])]
    fn bytes(s: &str) -> Vec<u8> {
        b(s)
    }

    #[test_case("000101", 6 => 5)]
    #[test_case("101010", 6 => 42)]
    fn read_int(s: &str, bits: u32) -> u32 {
        DataReader::new(&b(s)).read_fixed_integer(bits).unwrap()
    }

    #[test_case("11" => 1)]
    #[test_case("011" => 2)]
    #[test_case("0011" => 3)]
    #[test_case("1011" => 4)]
    #[test_case("00011" => 5)]
    #[test_case("10011" => 6)]
    #[test_case("01011" => 7)]
    #[test_case("0100000000001011" => 2 ; "overflow for u8")] // ignore bits we can't encode
    fn read_fibonacci(s: &str) -> u8 {
        DataReader::new(&b(s)).read_fibonacci_integer().unwrap()
    }

    #[test_case("101010", 1 => "k")]
    #[test_case("101010 101011", 2 => "kl")]
    fn read_string(s: &str, chars: usize) -> String {
        DataReader::new(&b(s)).read_string(chars).unwrap()
    }

    #[test_case("001111101100100110001110010001011101" => 1685434479)]
    #[test_case("000000000000000000000000000000000000" => 0)]
    fn read_datetime_as_unix_timestamp(s: &str) -> u64 {
        DataReader::new(&b(s))
            .read_datetime_as_unix_timestamp()
            .unwrap()
    }

    #[test_case("10101", 5 => BTreeSet::from_iter([1, 3, 5]))]
    #[test_case("101010", 6 => BTreeSet::from_iter([1, 3, 5]))]
    #[test_case("101010", 0 => BTreeSet::from_iter([]))]
    fn read_fixed_bitfield(s: &str, bits: usize) -> BTreeSet<u16> {
        DataReader::new(&b(s)).read_fixed_bitfield(bits).unwrap()
    }

    #[test_case("0000000000000101 10101" => BTreeSet::from_iter([1, 3, 5]))]
    fn read_variable_bitfield(s: &str) -> BTreeSet<u16> {
        DataReader::new(&b(s)).read_variable_bitfield().unwrap()
    }

    #[test_case("000000000010 0 0000000000000011 1 0000000000000101 0000000000001000" => vec![3, 5, 6, 7, 8] ; "test1")]
    fn read_integer_range(s: &str) -> Vec<u16> {
        DataReader::new(&b(s)).read_integer_range().unwrap()
    }

    #[test_case("000000000010 0 0011 1 011 0011" => vec![3, 5, 6, 7, 8])]
    #[test_case("000000000010 0 011 0 1011" => vec![2, 6])]
    fn read_fibonacci_range(s: &str) -> Vec<u8> {
        DataReader::new(&b(s)).read_fibonacci_range().unwrap()
    }

    #[test_case("1 000000000010 0 0011 1 011 0011" => BTreeSet::from_iter([3, 5, 6, 7, 8]))]
    #[test_case("0 0000000000000101 10101" => BTreeSet::from_iter([1, 3, 5]))]
    fn read_optimized_range(s: &str) -> BTreeSet<u16> {
        DataReader::new(&b(s)).read_optimized_range().unwrap()
    }

    #[test_case("0000000000000000 1 000000000010 0 0000000000000011 1 0000000000000101 0000000000001000" => BTreeSet::from_iter([3, 5, 6, 7, 8]) ; "test1")]
    #[test_case("0000000000000101 0 10101" => BTreeSet::from_iter([1, 3, 5]) ; "test2")]
    fn read_optimized_int_range(s: &str) -> BTreeSet<u16> {
        DataReader::new(&b(s))
            .read_optimized_integer_range()
            .unwrap()
    }

    #[test_case("000000000000" => Vec::<Range>::new() ; "empty")]
    #[test_case("000000000001 000011 01 0000000000000101 0 10101" => vec![
        Range {
            key: 3,
            range_type: 1,
            ids: BTreeSet::from_iter([1, 3, 5]),
        },
    ] ; "1 element")]
    #[test_case("000000000010 000011 01 0000000000000101 0 10101 000010 10 0000000000000000 1 000000000010 0 0000000000000011 1 0000000000000101 0000000000001000" => vec![
        Range {
            key: 3,
            range_type: 1,
            ids: BTreeSet::from_iter([1, 3, 5]),
        },
        Range {
            key: 2,
            range_type: 2,
            ids: BTreeSet::from_iter([3, 5, 6, 7, 8]),
        },
    ] ; "2 elements")]
    fn read_array_of_ranges(s: &str) -> Vec<Range> {
        DataReader::new(&b(s)).read_array_of_ranges().unwrap()
    }

    #[test_case("000000000000" => Vec::<GenericRange<u8, u8>>::new() ; "empty")]
    #[test_case("000000000001 000011 01 0 0000000000000101 10101" => vec![
        Range {
            key: 3,
            range_type: 1,
            ids: BTreeSet::from_iter([1, 3, 5]),
        },
    ] ; "1 element")]
    #[test_case("000000000010 000011 01 0 0000000000000101 10101 000010 10 1 000000000010 0 0011 1 011 0011" => vec![
        Range {
            key: 3,
            range_type: 1,
            ids: BTreeSet::from_iter([1, 3, 5]),
        },
        Range {
            key: 2,
            range_type: 2,
            ids: BTreeSet::from_iter([3, 5, 6, 7, 8]),
        },
    ] ; "2 elements")]
    fn read_n_array_of_ranges(s: &str) -> Vec<GenericRange<u8, u8>> {
        DataReader::new(&b(s))
            .read_n_array_of_ranges::<u8, u8>(6, 2)
            .unwrap()
    }
}

use core::marker::PhantomData;

use crate::{
    helper::data_capacity,
    types::{EccLevel, EncodeMode, QrError},
    version::{DataEccBuf, Version},
};

#[derive(Debug, Clone, Copy)]
pub(crate) struct EncodedData<T: Version> {
    pub data_len: usize,
    _marker: PhantomData<T>,
}

impl<T: Version> EncodedData<T> {
    pub(crate) fn encode(
        data: &[u8],
        mode: EncodeMode,
        level: EccLevel,
        codewords: &mut DataEccBuf<T>,
    ) -> Result<Self, QrError> {
        let data_len = data_capacity::<T>(level);
        let mut writer = BitWriter::new(&mut codewords.as_mut()[..data_len]);
        let count_bits = mode.counter_bits::<T>();

        writer.push_bits(mode.mode_bits(), 4)?;
        writer.push_bits(data.len() as u16, count_bits)?;
        mode.encode(data, &mut writer)?;

        let capacity_bits = data_len * 8;
        let terminator_bits = capacity_bits.saturating_sub(writer.bit_len()).min(4);
        writer.push_bits(0, terminator_bits)?;

        let remainder = writer.bit_len() % 8;
        if remainder != 0 {
            writer.push_bits(0, 8 - remainder)?;
        }

        let mut pad_byte = 0xECu8;
        while writer.byte_len() < data_len {
            writer.push_bits(pad_byte as u16, 8)?;
            pad_byte = if pad_byte == 0xEC { 0x11 } else { 0xEC };
        }

        Ok(Self {
            data_len,
            _marker: PhantomData,
        })
    }
}

impl EncodeMode {
    pub(crate) fn choose(data: &[u8]) -> Self {
        if data.iter().all(|byte| byte.is_ascii_digit()) {
            Self::Numeric
        } else if data
            .iter()
            .all(|byte| Self::alphanumeric_value(*byte).is_some())
        {
            Self::Alphanumeric
        } else {
            Self::Bytes
        }
    }

    fn encode(self, data: &[u8], writer: &mut BitWriter<'_>) -> Result<(), QrError> {
        match self {
            Self::Numeric => self.encode_numeric(data, writer),
            Self::Alphanumeric => self.encode_alphanumeric(data, writer),
            Self::Bytes => self.encode_bytes(data, writer),
        }
    }

    fn encode_numeric(self, data: &[u8], writer: &mut BitWriter<'_>) -> Result<(), QrError> {
        let mut index = 0;
        while index < data.len() {
            let remaining = data.len() - index;
            let group_len = remaining.min(3);
            let mut value = 0u16;

            for offset in 0..group_len {
                let byte = data[index + offset];
                if !byte.is_ascii_digit() {
                    return Err(QrError::DataInvalid);
                }
                value = (value * 10) + u16::from(byte - b'0');
            }

            let bit_count = match group_len {
                3 => 10,
                2 => 7,
                _ => 4,
            };

            writer.push_bits(value, bit_count)?;
            index += group_len;
        }

        Ok(())
    }

    fn encode_alphanumeric(self, data: &[u8], writer: &mut BitWriter<'_>) -> Result<(), QrError> {
        let mut index = 0;
        while index + 1 < data.len() {
            let left = Self::alphanumeric_value(data[index]).ok_or(QrError::DataInvalid)?;
            let right = Self::alphanumeric_value(data[index + 1]).ok_or(QrError::DataInvalid)?;
            writer.push_bits((left * 45) + right, 11)?;
            index += 2;
        }

        if index < data.len() {
            let value = Self::alphanumeric_value(data[index]).ok_or(QrError::DataInvalid)?;
            writer.push_bits(value, 6)?;
        }

        Ok(())
    }

    fn encode_bytes(self, data: &[u8], writer: &mut BitWriter<'_>) -> Result<(), QrError> {
        for &byte in data {
            writer.push_bits(u16::from(byte), 8)?;
        }

        Ok(())
    }

    fn alphanumeric_value(byte: u8) -> Option<u16> {
        match byte {
            b'0'..=b'9' => Some(u16::from(byte - b'0')),
            b'A'..=b'Z' => Some(u16::from(byte - b'A') + 10),
            b' ' => Some(36),
            b'$' => Some(37),
            b'%' => Some(38),
            b'*' => Some(39),
            b'+' => Some(40),
            b'-' => Some(41),
            b'.' => Some(42),
            b'/' => Some(43),
            b':' => Some(44),
            _ => None,
        }
    }
}

struct BitWriter<'a> {
    buffer: &'a mut [u8],
    bit_len: usize,
}

impl<'a> BitWriter<'a> {
    fn new(buffer: &'a mut [u8]) -> Self {
        buffer.fill(0);
        Self { buffer, bit_len: 0 }
    }

    fn push_bits(&mut self, value: u16, count: usize) -> Result<(), QrError> {
        if count > 16 || self.bit_len + count > self.buffer.len() * 8 {
            return Err(QrError::Overflow);
        }

        let mut shift = count;
        while shift > 0 {
            shift -= 1;
            let bit = ((value >> shift) & 1) as u8;
            let index = self.bit_len >> 3;
            let offset = 7 - (self.bit_len & 0x07);
            self.buffer[index] |= bit << offset;
            self.bit_len += 1;
        }

        Ok(())
    }

    fn bit_len(&self) -> usize {
        self.bit_len
    }

    fn byte_len(&self) -> usize {
        self.bit_len / 8
    }
}

#[cfg(test)]
mod tests {
    use super::EncodedData;
    use crate::{
        types::{EccLevel, EncodeMode},
        version::{Version1, new_data_ecc_buf},
    };

    #[test]
    fn choose_mode_prefers_numeric() {
        assert_eq!(EncodeMode::choose(b"8675309"), EncodeMode::Numeric);
    }

    #[test]
    fn choose_mode_prefers_alphanumeric_over_bytes() {
        assert_eq!(EncodeMode::choose(b"HELLO WORLD"), EncodeMode::Alphanumeric);
    }

    #[test]
    fn choose_mode_falls_back_to_bytes() {
        assert_eq!(EncodeMode::choose("hello".as_bytes()), EncodeMode::Bytes);
    }

    #[test]
    fn counter_bits_follow_version_ranges() {
        assert_eq!(EncodeMode::Numeric.counter_bits::<Version1>(), 10);
        assert_eq!(EncodeMode::Alphanumeric.counter_bits::<Version1>(), 9);
        assert_eq!(EncodeMode::Bytes.counter_bits::<Version1>(), 8);
    }

    #[test]
    fn numeric_encoding_pads_to_version_capacity() {
        let mut codewords = new_data_ecc_buf::<Version1>();
        let encoded = EncodedData::<Version1>::encode(
            b"01234567",
            EncodeMode::Numeric,
            EccLevel::M,
            &mut codewords,
        )
        .unwrap();
        assert_eq!(EncodeMode::choose(b"01234567"), EncodeMode::Numeric);
        assert_eq!(encoded.data_len, 16);
        assert_eq!(
            &codewords.as_ref()[..16],
            &[
                0x10, 0x20, 0x0C, 0x56, 0x61, 0x80, 0xEC, 0x11, 0xEC, 0x11, 0xEC, 0x11, 0xEC, 0x11,
                0xEC, 0x11
            ]
        );
    }

    #[test]
    fn byte_encoding_rejects_overflow() {
        let mut codewords = new_data_ecc_buf::<Version1>();
        let result = EncodedData::<Version1>::encode(
            &[0xAA; 15],
            EncodeMode::Bytes,
            EccLevel::H,
            &mut codewords,
        );
        assert!(result.is_err());
    }
}

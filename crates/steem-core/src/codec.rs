use crate::error::{Error, Result};

/// A little-endian binary writer matching Steem's Graphene wire format.
#[derive(Clone, Debug, Default)]
pub struct Encoder {
    buffer: Vec<u8>,
}

impl Encoder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.buffer
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.buffer
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn write_u8(&mut self, value: u8) {
        self.buffer.push(value);
    }

    pub fn write_bool(&mut self, value: bool) {
        self.write_u8(u8::from(value));
    }

    pub fn write_u16_le(&mut self, value: u16) {
        self.buffer.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_u32_le(&mut self, value: u32) {
        self.buffer.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_u64_le(&mut self, value: u64) {
        self.buffer.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_i16_le(&mut self, value: i16) {
        self.buffer.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_i64_le(&mut self, value: i64) {
        self.buffer.extend_from_slice(&value.to_le_bytes());
    }

    /// Unsigned LEB128 varint used by Graphene for lengths and operation ids.
    pub fn write_varint32(&mut self, mut value: u32) {
        loop {
            let mut byte = (value & 0x7f) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            self.buffer.push(byte);
            if value == 0 {
                break;
            }
        }
    }

    pub fn write_vstring(&mut self, value: &str) {
        self.write_varint32(value.len() as u32);
        self.buffer.extend_from_slice(value.as_bytes());
    }

    pub fn write_bytes(&mut self, value: &[u8]) {
        self.buffer.extend_from_slice(value);
    }

    pub fn write_vbytes(&mut self, value: &[u8]) {
        self.write_varint32(value.len() as u32);
        self.buffer.extend_from_slice(value);
    }
}

/// A little-endian binary reader matching Steem's Graphene wire format.
#[derive(Clone, Debug)]
pub struct Decoder<'a> {
    data: &'a [u8],
    position: usize,
}

impl<'a> Decoder<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, position: 0 }
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.position)
    }

    fn take(&mut self, length: usize) -> Result<&'a [u8]> {
        let end = self
            .position
            .checked_add(length)
            .ok_or_else(|| Error::Serialization("length overflow".to_string()))?;
        if end > self.data.len() {
            return Err(Error::Serialization(format!(
                "unexpected end of input at byte {}",
                self.position
            )));
        }
        let slice = &self.data[self.position..end];
        self.position = end;
        Ok(slice)
    }

    pub fn read_u8(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }

    pub fn read_bool(&mut self) -> Result<bool> {
        Ok(self.read_u8()? != 0)
    }

    pub fn read_u16_le(&mut self) -> Result<u16> {
        let bytes: [u8; 2] = self.take(2)?.try_into().unwrap();
        Ok(u16::from_le_bytes(bytes))
    }

    pub fn read_u32_le(&mut self) -> Result<u32> {
        let bytes: [u8; 4] = self.take(4)?.try_into().unwrap();
        Ok(u32::from_le_bytes(bytes))
    }

    pub fn read_u64_le(&mut self) -> Result<u64> {
        let bytes: [u8; 8] = self.take(8)?.try_into().unwrap();
        Ok(u64::from_le_bytes(bytes))
    }

    pub fn read_i16_le(&mut self) -> Result<i16> {
        let bytes: [u8; 2] = self.take(2)?.try_into().unwrap();
        Ok(i16::from_le_bytes(bytes))
    }

    pub fn read_i64_le(&mut self) -> Result<i64> {
        let bytes: [u8; 8] = self.take(8)?.try_into().unwrap();
        Ok(i64::from_le_bytes(bytes))
    }

    pub fn read_varint32(&mut self) -> Result<u32> {
        let mut result = 0u32;
        let mut shift = 0u32;
        loop {
            let byte = self.read_u8()?;
            if shift >= 32 {
                return Err(Error::Serialization("varint overflow".to_string()));
            }
            result |= ((byte & 0x7f) as u32) << shift;
            if byte & 0x80 == 0 {
                return Ok(result);
            }
            shift += 7;
        }
    }

    pub fn read_vstring(&mut self) -> Result<String> {
        let length = self.read_varint32()? as usize;
        let bytes = self.take(length)?;
        String::from_utf8(bytes.to_vec())
            .map_err(|_| Error::Serialization("invalid UTF-8 string".to_string()))
    }

    pub fn read_bytes(&mut self, length: usize) -> Result<&'a [u8]> {
        self.take(length)
    }

    pub fn read_vbytes(&mut self) -> Result<&'a [u8]> {
        let length = self.read_varint32()? as usize;
        self.take(length)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn varint_round_trip() {
        for value in [0, 1, 127, 128, 300, 16_384, u32::MAX] {
            let mut encoder = Encoder::new();
            encoder.write_varint32(value);
            let bytes = encoder.into_bytes();
            let mut decoder = Decoder::new(&bytes);
            assert_eq!(decoder.read_varint32().unwrap(), value);
            assert_eq!(decoder.remaining(), 0);
        }
    }

    #[test]
    fn vstring_round_trip() {
        let mut encoder = Encoder::new();
        encoder.write_vstring("hello");
        let bytes = encoder.into_bytes();
        let mut decoder = Decoder::new(&bytes);
        assert_eq!(decoder.read_vstring().unwrap(), "hello");
    }
}

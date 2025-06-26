use crate::error::Error;

#[derive(Debug, PartialEq, Eq)]
pub struct EncryptionKey([u8; 16]);
impl EncryptionKey {
    pub fn new(inner: [u8; 16]) -> Self {
        Self(inner)
    }

    pub fn from_hex_str(s: &str) -> Result<Self, Error> {
        if s.len() != 32 {
            let msg = format!("Invalid encryption key length: {s}");
            return Err(Error::Key(msg));
        }

        let mut key = [0u8; 16];
        for i in (0..s.len()).step_by(2) {
            let byte = &s[i..=i + 1];
            let hex = u8::from_str_radix(byte, 16)
                .map_err(|e| format!("Invalid encryption key data: {e}"))
                .map_err(Error::Key)?;
            key[i / 2] = hex;
        }

        Ok(Self(key))
    }

    pub fn xor<I, T>(&self, data: I) -> Vec<u8>
    where
        I: IntoIterator<Item = T>,
        T: std::borrow::Borrow<u8>,
    {
        data.into_iter()
            .enumerate()
            .map(|(i, x)| x.borrow() ^ self.0[i % 16])
            .collect()
    }
}

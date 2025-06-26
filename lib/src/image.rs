use crate::{encryption_key::EncryptionKey, error::Error};
use std::io::{BufReader, Read};

const DEFAULT_PNG_HEADER: [u8; 16] = [
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52,
];

pub fn decrypt<R>(key: Option<EncryptionKey>, data: R) -> Result<Vec<u8>, Error>
where
    R: Read,
{
    let reader = BufReader::new(data);
    let bytes = reader.bytes().collect::<Result<Vec<_>, _>>()?;

    let key = match key {
        Some(key) => key,
        None => {
            let key = EncryptionKey::new(DEFAULT_PNG_HEADER).xor(&bytes[16..32]);

            EncryptionKey::new(std::array::from_fn(|i| key[i]))
        }
    };

    let decrypted_header = key.xor(&bytes[16..32]);
    let body = &bytes[32..];

    let mut output = Vec::with_capacity(decrypted_header.len() + body.len());

    output.extend(decrypted_header);
    output.extend(body);

    Ok(output)
}

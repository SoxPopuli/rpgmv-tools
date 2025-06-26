use crate::error::{
    Error::{self, *},
    LzErrorKind,
};

pub fn decompress(data: &str) -> Result<String, Error> {
    let decompressed =
        lz_str::decompress_from_base64(data).ok_or(Lz(LzErrorKind::Decompression))?;
    String::from_utf16(&decompressed).map_err(|_| Lz(LzErrorKind::Decompression))
}

pub fn compress(data: &str) -> String {
    lz_str::compress_to_base64(data)
}

pub fn decompress_json_pretty(data: &str) -> Result<String, Error> {
    let decompressed = decompress(data)?;
    let json_str: serde_json::Value = serde_json::from_str(&decompressed)?;

    serde_json::to_string_pretty(&json_str).map_err(|e| e.into())
}

pub fn compress_json(data: &str) -> Result<String, Error> {
    let json: serde_json::Value = serde_json::from_str(data)?;
    let json_str = serde_json::to_string(&json)?;
    Ok(compress(&json_str))
}

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

#[derive(Debug)]
#[repr(transparent)]
pub struct Json(serde_json::Value);
impl Json {
    pub fn decompress(data: &str) -> Result<Self, Error> {
        let decompressed = decompress(data)?;
        serde_json::from_str(&decompressed)
            .map(Json)
            .map_err(|e| e.into())
    }

    pub fn from_string(s: &str) -> Result<Self, Error> {
        serde_json::from_str(s).map(Self).map_err(|e| e.into())
    }

    pub fn compress(&self) -> Result<String, Error> {
        let s = self.to_string();
        s.map(|x| compress(&x))
    }

    pub fn inner(self) -> serde_json::Value {
        self.0
    }

    pub fn to_string(&self) -> Result<String, Error> {
        serde_json::to_string(&self.0).map_err(|e| e.into())
    }

    pub fn to_string_pretty(&self) -> Result<String, Error> {
        serde_json::to_string_pretty(&self.0).map_err(|e| e.into())
    }
}

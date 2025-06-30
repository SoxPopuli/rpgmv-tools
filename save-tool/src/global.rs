use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct SpriteInner(String, usize);

#[derive(Debug, PartialEq, Eq)]
pub struct SpriteInfo {
    pub file_name: String,
    pub sprite_index: usize,
}
impl<'de> Deserialize<'de> for SpriteInfo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let SpriteInner(file_name, sprite_index) = SpriteInner::deserialize(deserializer)?;
        Ok(Self {
            file_name,
            sprite_index,
        })
    }
}
impl Serialize for SpriteInfo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        SpriteInner(self.file_name.clone(), self.sprite_index).serialize(serializer)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct DateTime(pub chrono::DateTime<chrono::Utc>);
impl<'de> Deserialize<'de> for DateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = i64::deserialize(deserializer)?;
        let dt = chrono::DateTime::from_timestamp_millis(value).expect("Invalid timestamp");
        Ok(Self(dt))
    }
}
impl Serialize for DateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let timestamp = self.0.timestamp_millis();
        timestamp.serialize(serializer)
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GlobalEntry {
    pub characters: Vec<SpriteInfo>,
    pub faces: Vec<SpriteInfo>,
    pub global_id: String,
    pub playtime: String,
    pub timestamp: DateTime,
    pub title: String
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Global(pub Vec<Option<GlobalEntry>>);

use crate::{
    error::Error, global::GlobalEntry, save_entry::SaveEntry, widgets::save_widget::SaveWidget,
};
use lib::save::Json as SaveJson;
use std::{collections::HashMap, path::Path};

#[derive(Debug, Default)]
pub enum SavesState {
    #[default]
    NotLoaded,
    Loaded {
        entries: Vec<SaveWidget>,
    },
    Error(Error),
}
impl SavesState {
    pub fn load_from_global(save_dir: &Path, global_path: &Path) -> SavesState {
        let save_json = {
            let file = std::fs::read_to_string(global_path);
            file.ok().and_then(|f| SaveJson::decompress(&f).ok())
        };

        let mut file_cache = HashMap::default();

        let save_json = match save_json {
            None => return SavesState::NotLoaded,
            Some(x) => x,
        };

        let mut from_values = |values: Vec<serde_json::Value>| -> SavesState {
            let values = values
                .into_iter()
                .map(serde_json::from_value::<Option<GlobalEntry>>)
                .enumerate()
                .map(|(i, entry)| match entry {
                    Ok(Some(entry)) => SaveEntry::new(entry, save_dir, &mut file_cache)
                        .map(|e| SaveWidget::new(i, e)),
                    Ok(None) => Ok(SaveWidget::empty(i)),
                    Err(e) => Err(Error::Io(e.to_string())),
                })
                .collect::<Result<Vec<_>, _>>();

            match values {
                Ok(v) => {
                    SavesState::Loaded { entries: v }
                }
                Err(e) => SavesState::Error(Error::global_error(e.to_string())),
            }
        };

        match save_json.inner() {
            serde_json::Value::Array(values) => from_values(values),
            _ => SavesState::Error(Error::global_error("Invalid global file")),
        }
    }
}

use crate::vec2::{IVec2, UVec2};
use crate::Try;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SudokuModel {
    pub size: UVec2,
    pub numbers: Vec<IVec2>,
}

impl SudokuModel {
    pub fn from_file(path: &PathBuf) -> Try<Self> {
        let file = std::fs::read_to_string(path)?;
        let model: SudokuModel = serde_json::from_str(&file)?;
        Ok(model)
    }

    pub fn to_file(&self, path: &PathBuf, pretty: bool) -> Try<()> {
        let file = if pretty {
            serde_json::to_string_pretty(self)?
        } else {
            serde_json::to_string(self)?
        };
        std::fs::write(path, file)?;
        Ok(())
    }
}

use crate::Try;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::path::PathBuf;
use crate::vec2::Vec2;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SudokuModel {
    pub size: Vec2,
    pub numbers: Vec<Vec2>,
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

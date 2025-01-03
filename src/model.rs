pub mod constraint;
pub mod region;

use crate::model::constraint::{Constraint, ConstraintSpecifier};
use crate::vec2::{IVec2, UVec2};
use crate::Try;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SudokuSpecifier {
    pub size: UVec2,
    pub numbers: Vec<IVec2>,
    pub constraints: Vec<ConstraintSpecifier>,
}

impl SudokuSpecifier {
    pub fn from_file(path: &PathBuf) -> Try<Self> {
        let file = std::fs::read_to_string(path)?;
        let model: SudokuSpecifier = serde_json::from_str(&file)?;
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

    pub fn build_model(&self) -> SudokuModel {
        let number_set: HashSet<isize> = self.numbers.iter().flat_map(|v| v.x..=v.y).collect();
        let mut numbers = number_set.into_iter().collect::<Vec<isize>>();
        numbers.sort_unstable();
        let constraints = self
            .constraints
            .iter()
            .map(|c| c.build_constraint())
            .collect();
        SudokuModel {
            size: self.size.clone(),
            numbers,
            constraints,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SudokuModel {
    pub size: UVec2,
    pub numbers: Vec<isize>,
    pub constraints: Vec<Constraint>,
}

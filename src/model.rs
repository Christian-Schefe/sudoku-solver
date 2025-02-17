pub mod constraint;
pub mod region;

use crate::model::constraint::{Constraint, ConstraintSpecifier};
use glam::IVec2;
use crate::Try;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SudokuSpecifier {
    pub size: IVec2,
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
        let number_set: HashSet<i32> = self.numbers.iter().flat_map(|v| v.x..=v.y).collect();
        let mut numbers = number_set.into_iter().collect::<Vec<i32>>();
        numbers.sort_unstable();
        let constraints = self
            .constraints
            .iter()
            .map(|c| c.build_constraint())
            .collect();
        let number_indices = numbers.iter().enumerate().map(|(i, &n)| (n, i)).collect();
        SudokuModel {
            size: self.size.clone(),
            numbers,
            number_indices,
            constraints,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SudokuModel {
    pub size: IVec2,
    pub numbers: Vec<i32>,
    pub number_indices: HashMap<i32, usize>,
    pub constraints: Vec<Constraint>,
}

impl SudokuModel {
    pub fn from_file(path: &PathBuf) -> Try<Self> {
        let specifier = SudokuSpecifier::from_file(path)?;
        specifier.to_file(path, true)?;
        Ok(specifier.build_model())
    }
}

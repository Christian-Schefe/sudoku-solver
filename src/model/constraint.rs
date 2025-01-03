use crate::model::region::{Line, LineSpecifier, Region, RegionSpecifier};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "constraint")]
pub enum ConstraintSpecifier {
    Unique(RegionSpecifier),
    Thermometer(LineSpecifier),
    Killer { region: RegionSpecifier, sum: isize },
}

#[derive(Debug, Clone)]
pub enum Constraint {
    Unique(Region),
    Thermometer(Line),
    Killer { region: Region, sum: isize },
}

impl ConstraintSpecifier {
    pub fn build_constraint(&self) -> Constraint {
        match self {
            ConstraintSpecifier::Unique(region) => Constraint::Unique(region.build_region()),
            ConstraintSpecifier::Thermometer(line) => Constraint::Thermometer(line.build_line()),
            ConstraintSpecifier::Killer { region, sum } => Constraint::Killer {
                region: region.build_region(),
                sum: *sum,
            },
        }
    }
}
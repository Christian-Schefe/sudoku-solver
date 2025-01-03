use crate::model::region::{Line, LineSpecifier, Region, RegionSpecifier};
use crate::vec2::UVec2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "constraint_type")]
pub enum ConstraintSpecifier {
    Unique {
        region: RegionSpecifier,
    },
    Thermometer {
        line: LineSpecifier,
    },
    Killer {
        region: RegionSpecifier,
        sum: isize,
    },
    Arrow {
        region: RegionSpecifier,
        tail: UVec2,
    },
    Relationship {
        first: UVec2,
        second: UVec2,
        relationship: Relationship,
    },
    Property {
        region: RegionSpecifier,
        property: Property,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Relationship {
    Greater,
    Less,
    Equal,
    NotEqual,
    Consecutive,
    Double,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Property {
    Even,
    Odd,
    Given(isize),
}

#[derive(Debug, Clone)]
pub enum Constraint {
    Unique(Region),
    Thermometer(Line),
    Killer {
        region: Region,
        sum: isize,
    },
    Arrow {
        region: Region,
        tail: UVec2,
    },
    Relationship {
        first: UVec2,
        second: UVec2,
        relationship: Relationship,
    },
    Property {
        region: Region,
        property: Property,
    },
}

impl ConstraintSpecifier {
    pub fn build_constraint(&self) -> Constraint {
        match self {
            ConstraintSpecifier::Unique { region } => Constraint::Unique(region.build_region()),
            ConstraintSpecifier::Thermometer { line } => Constraint::Thermometer(line.build_line()),
            ConstraintSpecifier::Killer { region, sum } => Constraint::Killer {
                region: region.build_region(),
                sum: *sum,
            },
            ConstraintSpecifier::Arrow { region, tail } => Constraint::Arrow {
                region: region.build_region(),
                tail: tail.clone(),
            },
            ConstraintSpecifier::Relationship {
                first,
                second,
                relationship,
            } => {
                if first == second {
                    panic!("Relationship cannot be between the same cell");
                }
                Constraint::Relationship {
                    first: first.clone(),
                    second: second.clone(),
                    relationship: relationship.clone(),
                }
            }
            ConstraintSpecifier::Property { region, property } => Constraint::Property {
                region: region.build_region(),
                property: property.clone(),
            },
        }
    }
}

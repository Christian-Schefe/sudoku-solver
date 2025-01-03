use crate::vec2::UVec2;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct Region {
    pub cells: HashSet<UVec2>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "region")]
pub enum RegionSpecifier {
    Box {
        start: UVec2,
        end: UVec2,
    },
    Line {
        points: Vec<UVec2>,
    },
    Combination {
        op: SetOperation,
        a: Box<RegionSpecifier>,
        b: Box<RegionSpecifier>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SetOperation {
    Union,
    Intersection,
    Difference,
}

impl RegionSpecifier {
    pub fn build_region(&self) -> Region {
        match self {
            RegionSpecifier::Box { start, end } => {
                let cells = UVec2::loop_box(start, end).collect();
                Region { cells }
            }
            RegionSpecifier::Line { points } => {
                let line = build_line(points);
                Region {
                    cells: line.into_iter().collect(),
                }
            }
            RegionSpecifier::Combination { op, a, b } => {
                let a_cells = a.build_region().cells;
                let b_cells = b.build_region().cells;
                let union = match op {
                    SetOperation::Union => a_cells.union(&b_cells).cloned().collect(),
                    SetOperation::Intersection => a_cells.intersection(&b_cells).cloned().collect(),
                    SetOperation::Difference => a_cells.difference(&b_cells).cloned().collect(),
                };
                Region { cells: union }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Line {
    pub cells: Vec<UVec2>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineSpecifier {
    pub points: Vec<UVec2>,
}

impl LineSpecifier {
    pub fn build_line(&self) -> Line {
        let cells = build_line(&self.points);
        Line { cells }
    }
}

fn build_line(points: &[UVec2]) -> Vec<UVec2> {
    let mut cells = Vec::new();
    for (index, endpoints) in points.windows(2).enumerate() {
        let start = &endpoints[0];
        let end = &endpoints[1];
        let include_end = index + 2 == points.len();
        let line = UVec2::loop_line(start, end, include_end).unwrap();
        cells.extend(line);
    }
    cells
}

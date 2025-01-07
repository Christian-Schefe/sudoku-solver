use glam::IVec2;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::Try;

#[derive(Debug, Clone)]
pub struct Region {
    pub cells: HashSet<IVec2>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "region_type")]
pub enum RegionSpecifier {
    Many {
        cells: Vec<IVec2>,
    },
    Box {
        start: IVec2,
        end: IVec2,
    },
    ManyBox {
        boxes: Vec<(IVec2, IVec2)>,
    },
    Line {
        points: Vec<IVec2>,
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
            RegionSpecifier::Many { cells } => Region {
                cells: cells.clone().into_iter().collect(),
            },
            RegionSpecifier::Box { start, end } => {
                let cells = loop_box(start, end).collect();
                Region { cells }
            }
            RegionSpecifier::ManyBox { boxes } => {
                let cells = boxes
                    .iter()
                    .flat_map(|(start, end)| loop_box(start, end))
                    .collect();
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
    pub cells: Vec<IVec2>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineSpecifier {
    pub points: Vec<IVec2>,
}

impl LineSpecifier {
    pub fn build_line(&self) -> Line {
        let cells = build_line(&self.points);
        Line { cells }
    }
}

fn build_line(points: &[IVec2]) -> Vec<IVec2> {
    let mut cells = Vec::new();
    for (index, endpoints) in points.windows(2).enumerate() {
        let start = &endpoints[0];
        let end = &endpoints[1];
        let include_end = index + 2 == points.len();
        let line = loop_line(start, end, include_end).unwrap();
        cells.extend(line);
    }
    cells
}

fn loop_box(start: &IVec2, end: &IVec2) -> impl Iterator<Item = IVec2> {
    let start = IVec2::new(start.x.min(end.x), start.y.min(end.y));
    let end = IVec2::new(start.x.max(end.x), start.y.max(end.y));
    (start.y..=end.y).flat_map(move |y| {
        (start.x..=end.x).map(move |x| IVec2::new(x, y))
    })
}

fn loop_line(start: &IVec2, end: &IVec2, include_end: bool) -> Try<impl Iterator<Item = IVec2>> {
    let start_i32 = *start;
    let dx = end.x - start_i32.x;
    let dy = end.y - start_i32.y;
    if dx == 0 && dy == 0 {
        return Err(anyhow::anyhow!("Start and end are the same"));
    } else if dx != 0 && dy != 0 && dx.abs() != dy.abs() {
        return Err(anyhow::anyhow!("Start and end are not aligned"));
    }
    let steps = dx.abs().max(dy.abs());
    let dx = dx.signum();
    let dy = dy.signum();
    let end = if include_end { steps } else { steps - 1 };
    Ok((0..=end).map(move |i| {
        let x = start_i32.x + dx * i;
        let y = start_i32.y + dy * i;
        IVec2::new(x, y)
    }))
}

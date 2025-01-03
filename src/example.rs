use crate::model::constraint::{ConstraintSpecifier, Property};
use crate::model::region::{LineSpecifier, RegionSpecifier};
use crate::model::SudokuSpecifier;
use crate::vec2::{IVec2, UVec2};
use std::path::PathBuf;

fn sudoku_constraints() -> Vec<ConstraintSpecifier> {
    let mut constraints = Vec::new();
    for i in 0..9 {
        constraints.push(ConstraintSpecifier::Unique {
            region: RegionSpecifier::Line {
                points: vec![UVec2::new(0, i), UVec2::new(8, i)],
            },
        });
        constraints.push(ConstraintSpecifier::Unique {
            region: RegionSpecifier::Line {
                points: vec![UVec2::new(i, 0), UVec2::new(i, 8)],
            },
        });
        let box_start = UVec2::new(i % 3 * 3, i / 3 * 3);
        constraints.push(ConstraintSpecifier::Unique {
            region: RegionSpecifier::Box {
                start: box_start.clone(),
                end: box_start + UVec2::new(2, 2),
            },
        });
    }
    constraints
}

fn given_constraints(constraints: &mut Vec<ConstraintSpecifier>, given: &Vec<(isize, Vec<UVec2>)>) {
    for (num, positions) in given {
        constraints.push(ConstraintSpecifier::Property {
            region: RegionSpecifier::Many {
                cells: positions.clone(),
            },
            property: Property::Given(*num),
        });
    }
}

pub fn test_model(path: Option<&PathBuf>) -> SudokuSpecifier {
    let mut constraints = sudoku_constraints();
    given_constraints(
        &mut constraints,
        &vec![
            (1, vec![UVec2::new(6, 1), UVec2::new(5, 7)]),
            (
                2,
                vec![
                    UVec2::new(4, 1),
                    UVec2::new(7, 7),
                    UVec2::new(2, 5),
                    UVec2::new(5, 8),
                ],
            ),
            (3, vec![UVec2::new(3, 4)]),
            (
                4,
                vec![UVec2::new(2, 2), UVec2::new(0, 8), UVec2::new(6, 3)],
            ),
            (5, vec![UVec2::new(3, 0), UVec2::new(5, 4)]),
            (
                6,
                vec![UVec2::new(5, 2), UVec2::new(1, 7), UVec2::new(3, 6)],
            ),
            (7, vec![UVec2::new(5, 3), UVec2::new(8, 8)]),
            (
                8,
                vec![UVec2::new(3, 5), UVec2::new(2, 6), UVec2::new(4, 7)],
            ),
            (
                9,
                vec![UVec2::new(1, 0), UVec2::new(6, 6), UVec2::new(4, 4)],
            ),
        ],
    );
    constraints.push(ConstraintSpecifier::Thermometer {
        line: LineSpecifier {
            points: vec![UVec2::new(0, 0), UVec2::new(0, 5)],
        },
    });
    constraints.push(ConstraintSpecifier::Thermometer {
        line: LineSpecifier {
            points: vec![UVec2::new(8, 0), UVec2::new(5, 0)],
        },
    });
    constraints.push(ConstraintSpecifier::Thermometer {
        line: LineSpecifier {
            points: vec![UVec2::new(4, 3), UVec2::new(2, 3)],
        },
    });
    constraints.push(ConstraintSpecifier::Thermometer {
        line: LineSpecifier {
            points: vec![UVec2::new(6, 5), UVec2::new(4, 5)],
        },
    });
    constraints.push(ConstraintSpecifier::Thermometer {
        line: LineSpecifier {
            points: vec![UVec2::new(8, 6), UVec2::new(8, 2)],
        },
    });
    constraints.push(ConstraintSpecifier::Thermometer {
        line: LineSpecifier {
            points: vec![UVec2::new(1, 8), UVec2::new(3, 8)],
        },
    });
    let specifier = SudokuSpecifier {
        size: UVec2::new(9, 9),
        numbers: vec![IVec2::new(1, 9)],
        constraints,
    };
    if let Some(path) = path {
        specifier.to_file(path, true).unwrap();
    }
    specifier
}

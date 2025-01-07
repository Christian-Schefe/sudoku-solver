use crate::model::constraint::{ConstraintSpecifier, Property};
use crate::model::region::{LineSpecifier, RegionSpecifier};
use crate::model::SudokuSpecifier;
use glam::IVec2;
use std::path::PathBuf;

fn sudoku_constraints() -> Vec<ConstraintSpecifier> {
    let mut constraints = Vec::new();
    for i in 0..9 {
        constraints.push(ConstraintSpecifier::Unique {
            region: RegionSpecifier::Line {
                points: vec![IVec2::new(0, i), IVec2::new(8, i)],
            },
        });
        constraints.push(ConstraintSpecifier::Unique {
            region: RegionSpecifier::Line {
                points: vec![IVec2::new(i, 0), IVec2::new(i, 8)],
            },
        });
        let box_start = IVec2::new(i % 3 * 3, i / 3 * 3);
        constraints.push(ConstraintSpecifier::Unique {
            region: RegionSpecifier::Box {
                start: box_start.clone(),
                end: box_start + IVec2::new(2, 2),
            },
        });
    }
    constraints
}

fn given_constraints(constraints: &mut Vec<ConstraintSpecifier>, given: &Vec<(i32, Vec<IVec2>)>) {
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
            (1, vec![IVec2::new(6, 1), IVec2::new(5, 7)]),
            (
                2,
                vec![
                    IVec2::new(4, 1),
                    IVec2::new(7, 7),
                    IVec2::new(2, 5),
                    IVec2::new(5, 8),
                ],
            ),
            (3, vec![IVec2::new(3, 4)]),
            (
                4,
                vec![IVec2::new(2, 2), IVec2::new(0, 8), IVec2::new(6, 3)],
            ),
            (5, vec![IVec2::new(3, 0), IVec2::new(5, 4)]),
            (
                6,
                vec![IVec2::new(5, 2), IVec2::new(1, 7), IVec2::new(3, 6)],
            ),
            (7, vec![IVec2::new(5, 3), IVec2::new(8, 8)]),
            (
                8,
                vec![IVec2::new(3, 5), IVec2::new(2, 6), IVec2::new(4, 7)],
            ),
            (
                9,
                vec![IVec2::new(1, 0), IVec2::new(6, 6), IVec2::new(4, 4)],
            ),
        ],
    );
    constraints.push(ConstraintSpecifier::Thermometer {
        line: LineSpecifier {
            points: vec![IVec2::new(0, 0), IVec2::new(0, 5)],
        },
    });
    constraints.push(ConstraintSpecifier::Thermometer {
        line: LineSpecifier {
            points: vec![IVec2::new(8, 0), IVec2::new(5, 0)],
        },
    });
    constraints.push(ConstraintSpecifier::Thermometer {
        line: LineSpecifier {
            points: vec![IVec2::new(4, 3), IVec2::new(2, 3)],
        },
    });
    constraints.push(ConstraintSpecifier::Thermometer {
        line: LineSpecifier {
            points: vec![IVec2::new(6, 5), IVec2::new(4, 5)],
        },
    });
    constraints.push(ConstraintSpecifier::Thermometer {
        line: LineSpecifier {
            points: vec![IVec2::new(8, 6), IVec2::new(8, 2)],
        },
    });
    constraints.push(ConstraintSpecifier::Thermometer {
        line: LineSpecifier {
            points: vec![IVec2::new(1, 8), IVec2::new(3, 8)],
        },
    });
    let specifier = SudokuSpecifier {
        size: IVec2::new(9, 9),
        numbers: vec![IVec2::new(1, 9)],
        constraints,
    };
    if let Some(path) = path {
        specifier.to_file(path, true).unwrap();
    }
    specifier
}

pub fn killer_test_model(path: Option<&PathBuf>) -> SudokuSpecifier {
    let mut constraints = sudoku_constraints();
    given_constraints(
        &mut constraints,
        &vec![
            (2, vec![IVec2::new(4, 6)]),
            (3, vec![IVec2::new(0, 4)]),
            (4, vec![IVec2::new(3, 0)]),
            (
                5,
                vec![IVec2::new(6, 2), IVec2::new(8, 5), IVec2::new(7, 7)],
            ),
            (6, vec![IVec2::new(5, 1)]),
            (7, vec![IVec2::new(4, 3)]),
            (8, vec![IVec2::new(1, 8)]),
            (9, vec![IVec2::new(1, 0)]),
        ],
    );
    constraints.push(ConstraintSpecifier::Killer {
        region: RegionSpecifier::Box {
            start: IVec2::new(0, 0),
            end: IVec2::new(0, 1),
        },
        sum: 7,
    });
    constraints.push(ConstraintSpecifier::Killer {
        region: RegionSpecifier::Many {
            cells: vec![IVec2::new(1, 0), IVec2::new(1, 1), IVec2::new(2, 0)],
        },
        sum: 16,
    });
    constraints.push(ConstraintSpecifier::Killer {
        region: RegionSpecifier::ManyBox {
            boxes: vec![
                (IVec2::new(3, 0), IVec2::new(4, 1)),
                (IVec2::new(2, 1), IVec2::new(2, 1)),
            ],
        },
        sum: 27,
    });
    constraints.push(ConstraintSpecifier::Killer {
        region: RegionSpecifier::Box {
            start: IVec2::new(5, 0),
            end: IVec2::new(5, 2),
        },
        sum: 9,
    });
    constraints.push(ConstraintSpecifier::Killer {
        region: RegionSpecifier::Box {
            start: IVec2::new(6, 0),
            end: IVec2::new(8, 0),
        },
        sum: 18,
    });
    constraints.push(ConstraintSpecifier::Killer {
        region: RegionSpecifier::Box {
            start: IVec2::new(6, 1),
            end: IVec2::new(7, 1),
        },
        sum: 6,
    });
    constraints.push(ConstraintSpecifier::Killer {
        region: RegionSpecifier::Box {
            start: IVec2::new(8, 1),
            end: IVec2::new(8, 2),
        },
        sum: 10,
    });
    constraints.push(ConstraintSpecifier::Killer {
        region: RegionSpecifier::Many {
            cells: vec![IVec2::new(0, 2)],
        },
        sum: 4,
    });
    constraints.push(ConstraintSpecifier::Killer {
        region: RegionSpecifier::Box {
            start: IVec2::new(1, 2),
            end: IVec2::new(2, 2),
        },
        sum: 10,
    });
    constraints.push(ConstraintSpecifier::Killer {
        region: RegionSpecifier::Box {
            start: IVec2::new(3, 2),
            end: IVec2::new(4, 2),
        },
        sum: 17,
    });
    constraints.push(ConstraintSpecifier::Killer {
        region: RegionSpecifier::Many {
            cells: vec![IVec2::new(6, 2), IVec2::new(6, 3), IVec2::new(7, 2)],
        },
        sum: 19,
    });
    constraints.push(ConstraintSpecifier::Killer {
        region: RegionSpecifier::Box {
            start: IVec2::new(0, 3),
            end: IVec2::new(1, 3),
        },
        sum: 10,
    });
    constraints.push(ConstraintSpecifier::Killer {
        region: RegionSpecifier::Box {
            start: IVec2::new(2, 3),
            end: IVec2::new(3, 3),
        },
        sum: 10,
    });
    constraints.push(ConstraintSpecifier::Killer {
        region: RegionSpecifier::ManyBox {
            boxes: vec![
                (IVec2::new(4, 3), IVec2::new(5, 3)),
                (IVec2::new(5, 4), IVec2::new(5, 5)),
            ],
        },
        sum: 29,
    });
    constraints.push(ConstraintSpecifier::Killer {
        region: RegionSpecifier::Box {
            start: IVec2::new(7, 3),
            end: IVec2::new(8, 3),
        },
        sum: 5,
    });
    constraints.push(ConstraintSpecifier::Killer {
        region: RegionSpecifier::ManyBox {
            boxes: vec![
                (IVec2::new(0, 4), IVec2::new(0, 5)),
                (IVec2::new(1, 4), IVec2::new(1, 6)),
            ],
        },
        sum: 25,
    });
    constraints.push(ConstraintSpecifier::Killer {
        region: RegionSpecifier::Box {
            start: IVec2::new(2, 4),
            end: IVec2::new(2, 5),
        },
        sum: 6,
    });
    constraints.push(ConstraintSpecifier::Killer {
        region: RegionSpecifier::Box {
            start: IVec2::new(3, 4),
            end: IVec2::new(4, 4),
        },
        sum: 6,
    });
    constraints.push(ConstraintSpecifier::Killer {
        region: RegionSpecifier::Many {
            cells: vec![IVec2::new(6, 4), IVec2::new(7, 4), IVec2::new(7, 5)],
        },
        sum: 14,
    });
    constraints.push(ConstraintSpecifier::Killer {
        region: RegionSpecifier::Many {
            cells: vec![IVec2::new(8, 4)],
        },
        sum: 6,
    });
    constraints.push(ConstraintSpecifier::Killer {
        region: RegionSpecifier::Box {
            start: IVec2::new(3, 5),
            end: IVec2::new(4, 5),
        },
        sum: 9,
    });
    constraints.push(ConstraintSpecifier::Killer {
        region: RegionSpecifier::Box {
            start: IVec2::new(6, 5),
            end: IVec2::new(6, 6),
        },
        sum: 16,
    });
    constraints.push(ConstraintSpecifier::Killer {
        region: RegionSpecifier::Box {
            start: IVec2::new(8, 5),
            end: IVec2::new(8, 6),
        },
        sum: 9,
    });
    constraints.push(ConstraintSpecifier::Killer {
        region: RegionSpecifier::Many {
            cells: vec![IVec2::new(0, 6)],
        },
        sum: 1,
    });
    constraints.push(ConstraintSpecifier::Killer {
        region: RegionSpecifier::ManyBox {
            boxes: vec![
                (IVec2::new(2, 6), IVec2::new(2, 6)),
                (IVec2::new(1, 7), IVec2::new(3, 7)),
            ],
        },
        sum: 21,
    });
    constraints.push(ConstraintSpecifier::Killer {
        region: RegionSpecifier::Many {
            cells: vec![IVec2::new(3, 6), IVec2::new(4, 6), IVec2::new(4, 7)],
        },
        sum: 9,
    });
    constraints.push(ConstraintSpecifier::Killer {
        region: RegionSpecifier::Many {
            cells: vec![IVec2::new(5, 6)],
        },
        sum: 7,
    });
    constraints.push(ConstraintSpecifier::Killer {
        region: RegionSpecifier::Many {
            cells: vec![IVec2::new(7, 6), IVec2::new(7, 7), IVec2::new(8, 7)],
        },
        sum: 20,
    });
    constraints.push(ConstraintSpecifier::Killer {
        region: RegionSpecifier::Many {
            cells: vec![IVec2::new(0, 7)],
        },
        sum: 9,
    });
    constraints.push(ConstraintSpecifier::Killer {
        region: RegionSpecifier::ManyBox {
            boxes: vec![
                (IVec2::new(5, 7), IVec2::new(5, 8)),
                (IVec2::new(3, 8), IVec2::new(4, 8)),
            ],
        },
        sum: 21,
    });
    constraints.push(ConstraintSpecifier::Killer {
        region: RegionSpecifier::Box {
            start: IVec2::new(6, 7),
            end: IVec2::new(6, 8),
        },
        sum: 8,
    });
    constraints.push(ConstraintSpecifier::Killer {
        region: RegionSpecifier::Many {
            cells: vec![IVec2::new(0, 8)],
        },
        sum: 7,
    });
    constraints.push(ConstraintSpecifier::Killer {
        region: RegionSpecifier::Box {
            start: IVec2::new(1, 8),
            end: IVec2::new(2, 8),
        },
        sum: 10,
    });
    constraints.push(ConstraintSpecifier::Killer {
        region: RegionSpecifier::Box {
            start: IVec2::new(7, 8),
            end: IVec2::new(8, 8),
        },
        sum: 4,
    });
    let specifier = SudokuSpecifier {
        size: IVec2::new(9, 9),
        numbers: vec![IVec2::new(1, 9)],
        constraints,
    };
    if let Some(path) = path {
        specifier.to_file(path, true).unwrap();
    }
    specifier
}

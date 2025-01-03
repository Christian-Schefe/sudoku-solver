use crate::model::constraint::ConstraintSpecifier;
use crate::model::region::RegionSpecifier;
use crate::model::SudokuSpecifier;
use crate::vec2::{IVec2, UVec2};
use clap::Parser;
use std::path::PathBuf;

mod model;
mod vec2;

#[derive(Debug, Parser)]
struct Args {
    path: PathBuf,
}

pub type Try<T> = Result<T, anyhow::Error>;

fn main() {
    let args = Args::parse();
    let specifier = SudokuSpecifier::from_file(&args.path).unwrap();
    specifier.to_file(&args.path, true).unwrap();
    let model = specifier.build_model();
    println!("{:?}", model);

    let vec = IVec2::new(1, 2);
    let v2 = vec * 2;
    println!("{}", v2);
}

fn test_model() -> SudokuSpecifier {
    SudokuSpecifier {
        size: UVec2::new(9, 9),
        numbers: vec![IVec2::new(1, 2), IVec2::new(3, 4)],
        constraints: vec![ConstraintSpecifier::Unique(RegionSpecifier::Box {
            start: UVec2::new(0, 0),
            end: UVec2::new(3, 3),
        })],
    }
}

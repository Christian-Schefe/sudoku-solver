use crate::model::SudokuModel;
use clap::Parser;
use std::path::PathBuf;

mod example;
mod model;
mod solver;
mod vec2;

#[derive(Debug, Parser)]
struct Args {
    path: PathBuf,
}

pub type Try<T> = Result<T, anyhow::Error>;

fn main() {
    let args = Args::parse();
    let model = example::killer_test_model(Some(&args.path)).build_model();
    SudokuModel::from_file(&args.path).unwrap();
    solver::solve(model);
}

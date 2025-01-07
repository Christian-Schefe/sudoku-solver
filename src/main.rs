use std::path::PathBuf;

use clap::Parser;
use sudoku_solver::{example, model::SudokuModel, solver};

#[derive(Debug, Parser)]
struct Args {
    path: PathBuf,
}

fn main() {
    let args = Args::parse();
    let model = example::killer_test_model(Some(&args.path)).build_model();
    SudokuModel::from_file(&args.path).unwrap();
    solver::solve(model);
}

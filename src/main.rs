use crate::model::SudokuModel;
use crate::vec2::Vec2;
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
    let model = SudokuModel::from_file(&args.path).unwrap();
    model.to_file(&args.path, true).unwrap();
    println!("{:?}", model);

    let vec = Vec2::new(1, 2);
    let v2 = vec * 2;
    println!("{}", v2);
}

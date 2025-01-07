pub mod example;
pub mod model;
pub mod solver;

pub type Try<T> = Result<T, anyhow::Error>;

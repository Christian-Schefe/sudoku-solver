use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::ops::{Add, Div, Mul, Sub};

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
#[serde(from = "(T, T)", into = "(T, T)")]
pub struct Point<T: Clone> {
    x: T,
    y: T,
}

impl<T: Add<T, Output = T> + Clone> Add<Point<T>> for Point<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T: Sub<T, Output = T> + Clone> Sub<Point<T>> for Point<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<T: Mul<T, Output = T> + Copy> Mul<T> for Point<T> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl<T: Div<T, Output = T> + Copy> Div<T> for Point<T> {
    type Output = Self;

    fn div(self, rhs: T) -> Self {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl<T: Clone> From<(T, T)> for Point<T> {
    fn from((x, y): (T, T)) -> Self {
        Self { x, y }
    }
}

impl<T: Clone> Into<(T, T)> for Point<T> {
    fn into(self) -> (T, T) {
        (self.x, self.y)
    }
}

impl<T: Display + Clone> Display for Point<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl<T: Clone> Point<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl Point<isize> {
    pub fn up() -> Self {
        Self { x: 0, y: -1 }
    }
    pub fn down() -> Self {
        Self { x: 0, y: 1 }
    }
    pub fn left() -> Self {
        Self { x: -1, y: 0 }
    }
    pub fn right() -> Self {
        Self { x: 1, y: 0 }
    }
}

impl Point<usize> {
    pub fn from_index(index: usize, width: usize) -> Self {
        Self {
            x: index % width,
            y: index / width,
        }
    }
    pub fn to_index(self, width: usize) -> usize {
        self.y * width + self.x
    }
}

pub type IVec2 = Point<isize>;
pub type UVec2 = Point<usize>;

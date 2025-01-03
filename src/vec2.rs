use crate::Try;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::ops::{Add, Div, Mul, Sub};

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
#[serde(from = "(T, T)", into = "(T, T)")]
pub struct Point<T: Clone> {
    pub x: T,
    pub y: T,
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
    pub fn convert<K: Clone + From<T>>(self) -> Point<K> {
        Point {
            x: K::from(self.x),
            y: K::from(self.y),
        }
    }
    pub fn convert_from<K: Clone + Into<T>>(other: Point<K>) -> Self {
        Point {
            x: other.x.into(),
            y: other.y.into(),
        }
    }
    pub fn try_convert<K: Clone + TryFrom<T>>(self) -> Result<Point<K>, K::Error> {
        Ok(Point {
            x: K::try_from(self.x)?,
            y: K::try_from(self.y)?,
        })
    }

    pub fn try_convert_from<K: Clone + TryInto<T>>(other: Point<K>) -> Result<Self, K::Error> {
        Ok(Point {
            x: other.x.try_into()?,
            y: other.y.try_into()?,
        })
    }
}

impl<T: num::Integer + Display + Clone + num::ToPrimitive> Point<T> {
    pub fn zero() -> Self {
        Self {
            x: T::zero(),
            y: T::zero(),
        }
    }
    pub fn one() -> Self {
        Self {
            x: T::one(),
            y: T::one(),
        }
    }
    pub fn loop_box<'a>(
        start: &Self,
        end: &Self,
        include_end: bool,
    ) -> impl Iterator<Item = Self> + 'a
    where
        T: 'a,
    {
        let s = start.clone();
        let e = if include_end {
            end.clone() + Point::one()
        } else {
            end.clone()
        };

        num::range(s.y.clone(), e.y.clone()).flat_map(move |y| {
            num::range(s.x.clone(), e.x.clone()).map(move |x| Self { x, y: y.clone() })
        })
    }
    pub fn loop_line<'a>(
        start: &'a Self,
        end: &'a Self,
        include_end: bool,
    ) -> Try<impl Iterator<Item = Self> + 'a> {
        let x_order = end.x.cmp(&start.x);
        let y_order = end.y.cmp(&start.y);
        fn try_diagonal<T: num::Integer + Display + Clone>(
            sx: &T,
            sy: &T,
            ex: &T,
            ey: &T,
        ) -> Result<T, anyhow::Error> {
            let dx = ex.clone() - sx.clone();
            let dy = ey.clone() - sy.clone();
            if dx == dy {
                Ok(dx)
            } else {
                Err(anyhow::anyhow!(
                    "Start and end of line are not aligned: dx is {}, but dy is {}",
                    dx,
                    dy
                ))
            }
        }
        let mut steps = match (x_order, y_order) {
            (Ordering::Equal, Ordering::Equal) => Err(anyhow::anyhow!(
                "Start and end of line are the same point: {}",
                start
            )),
            (Ordering::Greater, Ordering::Equal) => Ok(end.x.clone() - start.x.clone()),
            (Ordering::Less, Ordering::Equal) => Ok(start.x.clone() - end.x.clone()),
            (Ordering::Equal, Ordering::Greater) => Ok(end.y.clone() - start.y.clone()),
            (Ordering::Equal, Ordering::Less) => Ok(start.y.clone() - end.y.clone()),
            (Ordering::Greater, Ordering::Greater) => {
                try_diagonal(&start.x, &start.y, &end.x, &end.y)
            }
            (Ordering::Greater, Ordering::Less) => try_diagonal(&start.x, &end.y, &end.x, &start.y),
            (Ordering::Less, Ordering::Greater) => try_diagonal(&end.x, &start.y, &start.x, &end.y),
            (Ordering::Less, Ordering::Less) => try_diagonal(&end.x, &end.y, &start.x, &start.y),
        }?;
        if include_end {
            steps = steps + T::one();
        }
        let iter = num::range(T::zero(), steps).map(move |i| {
            let x = match x_order {
                Ordering::Greater => start.x.clone() + i.clone(),
                Ordering::Less => start.x.clone() - i.clone(),
                Ordering::Equal => start.x.clone(),
            };
            let y = match y_order {
                Ordering::Greater => start.y.clone() + i,
                Ordering::Less => start.y.clone() - i,
                Ordering::Equal => start.y.clone(),
            };
            Self { x, y }
        });
        Ok(iter)
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

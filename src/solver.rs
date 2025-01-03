use crate::model::constraint::{Constraint, Property, Relationship};
use crate::model::SudokuModel;
use crate::vec2::UVec2;
use std::collections::{HashMap, HashSet};

pub struct SolverState {
    pub grid: Vec<Vec<Cell>>,
}

impl SolverState {
    pub fn print_grid(&self) {
        for row in &self.grid {
            for cell in row {
                if let Some(value) = cell.value {
                    print!("{} ", value);
                } else {
                    print!(". ");
                }
            }
            println!();
        }
        for row in &self.grid {
            for cell in row {
                if cell.value.is_some() {
                    continue;
                }
                println!("{}: {:?}", cell.pos, cell.candidates);
            }
        }
    }
}

pub struct Cell {
    pub pos: UVec2,
    pub value: Option<isize>,
    pub candidates: Vec<isize>,
}

impl Cell {
    fn limit<F>(&mut self, filter: F) -> Option<bool>
    where
        F: Fn(&isize) -> bool,
    {
        let old_len = self.candidates.len();
        self.candidates.retain(&filter);
        if self.candidates.len() == 1 {
            self.value = Some(self.candidates[0]);
            self.candidates.clear();
        } else if self.candidates.is_empty() {
            return None;
        }
        Some(self.candidates.len() != old_len)
    }
}

fn empty_grid(size: &UVec2, candidates: &Vec<isize>) -> Vec<Vec<Cell>> {
    (0..size.y)
        .map(|y| {
            (0..size.x)
                .map(|x| Cell {
                    pos: UVec2::new(x, y),
                    value: None,
                    candidates: candidates.clone(),
                })
                .collect()
        })
        .collect()
}

pub fn solve(model: SudokuModel) {
    let grid = empty_grid(&model.size, &model.numbers);
    let mut state = SolverState { grid };

    let mut changed = true;
    while changed {
        changed = false;
        for constraint in &model.constraints {
            if let Some(c) = limit_state(&model, &mut state, constraint) {
                changed |= c;
            } else {
                println!("Invalid constraint");
                return;
            }
        }
    }
    state.print_grid();
}

fn limit_state(
    model: &SudokuModel,
    state: &mut SolverState,
    constraint: &Constraint,
) -> Option<bool> {
    let mut changed = false;
    match constraint {
        Constraint::Unique(region) => {
            let placed = region
                .cells
                .iter()
                .filter_map(|cell| state.grid[cell.y][cell.x].value)
                .collect::<HashSet<isize>>();
            for pos in &region.cells {
                let cell = &mut state.grid[pos.y][pos.x];
                if cell.value.is_some() {
                    continue;
                }
                changed |= cell.limit(|c| !placed.contains(c))?;
            }
            let possibly_contained = region
                .cells
                .iter()
                .flat_map(|cell| state.grid[cell.y][cell.x].candidates.clone())
                .collect::<HashSet<isize>>();
            let free_spots = region
                .cells
                .iter()
                .filter(|cell| state.grid[cell.y][cell.x].value.is_none())
                .collect::<Vec<_>>();
            if possibly_contained.len() == free_spots.len() {
                for pos in &free_spots {
                    let cell = &mut state.grid[pos.y][pos.x];
                    changed |= cell.limit(|c| possibly_contained.contains(c))?;
                }
                let mut possible_spots = HashMap::new();
                for pos in free_spots {
                    for candidate in &state.grid[pos.y][pos.x].candidates {
                        possible_spots
                            .entry(*candidate)
                            .or_insert_with(Vec::new)
                            .push(pos);
                    }
                }
                for (num, spots) in &possible_spots {
                    if spots.len() == 1 {
                        let pos = spots[0];
                        let cell = &mut state.grid[pos.y][pos.x];
                        changed |= cell.limit(|c| *c == *num)?;
                    }
                }
            }
        }
        Constraint::Thermometer(line) => {
            let len = line.cells.len();
            if len > model.numbers.len() {
                return None;
            }
            let mut offset = 0;
            let min_indices: Vec<usize> = (0..len)
                .map(|i| {
                    let cell = &state.grid[line.cells[i].y][line.cells[i].x];
                    if let Some(value) = cell.value {
                        let value_index = model.number_indices[&value];
                        if value_index < i + offset {
                            return None;
                        }
                        offset = value_index - i;
                        Some(value_index)
                    } else {
                        let value_index = i + offset;
                        if value_index >= model.numbers.len() {
                            return None;
                        }
                        Some(value_index)
                    }
                })
                .collect::<Option<_>>()?;

            for (i, pos) in line.cells.iter().enumerate() {
                let cell = &mut state.grid[pos.y][pos.x];
                if cell.value.is_some() {
                    continue;
                }
                changed |= cell.limit(|c| min_indices[i] <= model.number_indices[c])?;
            }
        }
        Constraint::Property { region, property } => {
            for pos in &region.cells {
                let cell = &mut state.grid[pos.y][pos.x];
                if cell.value.is_some() {
                    continue;
                }
                changed |= cell.limit(|c| match property {
                    Property::Even => c % 2 == 0,
                    Property::Odd => c % 2 != 0,
                    Property::Given(value) => *c == *value,
                })?;
            }
        }
        Constraint::Relationship {
            first,
            second,
            relationship,
        } => {
            let first_value = state.grid[first.y][first.x].value;
            let second_value = state.grid[second.y][second.x].value;
            if first_value.is_none() && second_value.is_none() {
                return None;
            }
            if let (Some(first_value), Some(second_value)) = (first_value, second_value) {
                match relationship {
                    Relationship::Less => {
                        if first_value >= second_value {
                            return None;
                        }
                    }
                    Relationship::Greater => {
                        if first_value <= second_value {
                            return None;
                        }
                    }
                    Relationship::Equal => {
                        if first_value != second_value {
                            return None;
                        }
                    }
                    Relationship::NotEqual => {
                        if first_value == second_value {
                            return None;
                        }
                    }
                    Relationship::Consecutive => {
                        let max = std::cmp::max(first_value, second_value);
                        let min = std::cmp::min(first_value, second_value);
                        if max - min != 1 {
                            return None;
                        }
                    }
                    Relationship::Double => {
                        let max = std::cmp::max(first_value, second_value);
                        let min = std::cmp::min(first_value, second_value);
                        if max != min * 2 {
                            return None;
                        }
                    }
                }
            } else {
                let (first_is_present, present, not_present) = if first_value.is_some() {
                    (true, first, second)
                } else {
                    (false, second, first)
                };
                let value = state.grid[present.y][present.x].value.unwrap();
                let cell = &mut state.grid[not_present.y][not_present.x];
                changed |= cell.limit(|c| match relationship {
                    Relationship::Less => {
                        if first_is_present {
                            *c < value
                        } else {
                            *c > value
                        }
                    }
                    Relationship::Greater => {
                        if first_is_present {
                            *c > value
                        } else {
                            *c < value
                        }
                    }
                    Relationship::Equal => *c == value,
                    Relationship::NotEqual => *c != value,
                    Relationship::Consecutive => *c == value + 1 || *c == value - 1,
                    Relationship::Double => *c == value * 2 || value == *c * 2,
                })?;
            }
        }
        _ => {}
    }
    Some(changed)
}

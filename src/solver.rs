use crate::model::constraint::{Constraint, Property, Relationship};
use crate::model::region::{Line, Region};
use crate::model::SudokuModel;
use glam::IVec2;
use std::collections::{HashMap, HashSet};

struct Precomputed {
    highest_sums: Vec<i32>,
    lowest_sums: Vec<i32>,
}

impl Precomputed {
    fn new(model: &SudokuModel) -> Self {
        let number_count = model.numbers.len();
        let mut highest_sums = vec![0; number_count + 1];
        let mut lowest_sums = vec![0; number_count + 1];
        for i in 0..number_count {
            highest_sums[i + 1] = model.numbers[number_count - i - 1] + highest_sums[i];
            lowest_sums[i + 1] = model.numbers[i] + lowest_sums[i];
        }
        Self {
            highest_sums,
            lowest_sums,
        }
    }
}

#[derive(Clone)]
pub struct SolverState<'a> {
    pub grid: Vec<Vec<Cell>>,
    precomputed: &'a Precomputed,
}

impl<'a> SolverState<'a> {
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
    fn is_solved(&self) -> bool {
        self.grid.iter().flatten().all(|cell| cell.value.is_some())
    }
    fn get(&self, pos: &IVec2) -> &Cell {
        &self.grid[pos.y as usize][pos.x as usize]
    }
    fn get_mut(&mut self, pos: &IVec2) -> &mut Cell {
        &mut self.grid[pos.y as usize][pos.x as usize]
    }
}

#[derive(Clone)]
pub struct Cell {
    pub pos: IVec2,
    pub value: Option<i32>,
    pub candidates: Vec<i32>,
}

impl Cell {
    fn limit<F>(&mut self, filter: F) -> Option<bool>
    where
        F: Fn(&i32) -> bool,
    {
        let old_len = self.candidates.len();
        self.candidates.retain(&filter);
        if self.candidates.len() == 1 {
            self.set_value(self.candidates[0]);
        } else if self.candidates.is_empty() {
            return None;
        }
        Some(self.candidates.len() != old_len)
    }
    fn set_value(&mut self, value: i32) {
        self.value = Some(value);
        self.candidates.clear();
    }
}

fn empty_grid(size: &IVec2, candidates: &Vec<i32>) -> Vec<Vec<Cell>> {
    (0..size.y)
        .map(|y| {
            (0..size.x)
                .map(|x| Cell {
                    pos: IVec2::new(x, y),
                    value: None,
                    candidates: candidates.clone(),
                })
                .collect()
        })
        .collect()
}

pub fn solve(model: SudokuModel) {
    let grid = empty_grid(&model.size, &model.numbers);
    let precomputed = Precomputed::new(&model);
    let mut state = SolverState {
        grid,
        precomputed: &precomputed,
    };

    let res = bifurcate(&model, &mut state);
    if res.is_none() {
        println!("No solution found");
    }
    state.print_grid();
}

fn bifurcate(model: &SudokuModel, state: &mut SolverState) -> Option<()> {
    try_limit(model, state)?;
    if state.is_solved() {
        return Some(());
    }
    let lowest = state
        .grid
        .iter()
        .flatten()
        .filter(|cell| cell.value.is_none())
        .min_by_key(|cell| cell.candidates.len())
        .unwrap();
    let pos = &lowest.pos;
    let candidates = lowest.candidates.clone();
    for candidate in candidates {
        let mut new_state = state.clone();
        new_state.get_mut(pos).set_value(candidate);
        if bifurcate(model, &mut new_state).is_some() {
            *state = new_state;
            return Some(());
        }
    }
    None
}

fn try_limit(model: &SudokuModel, state: &mut SolverState) -> Option<()> {
    let mut changed = true;
    while changed {
        changed = false;
        for constraint in &model.constraints {
            changed |= limit_state(&model, state, constraint)?;
        }
    }
    Some(())
}

fn limit_state(
    model: &SudokuModel,
    state: &mut SolverState,
    constraint: &Constraint,
) -> Option<bool> {
    let mut changed = false;
    match constraint {
        Constraint::Unique(region) => {
            limit_unique_clue(region, state, &mut changed)?;
        }
        Constraint::Thermometer(line) => {
            limit_thermometer_clue(line, model, state, &mut changed)?;
        }
        Constraint::Property { region, property } => {
            limit_property_clue(region, state, property, &mut changed)?;
        }
        Constraint::Relationship {
            first,
            second,
            relationship,
        } => {
            limit_relationship_clue(first, second, relationship, state, &mut changed)?;
        }
        Constraint::Killer { region, sum } => {
            limit_killer_clue(region, sum, state, &mut changed)?;
        }
        _ => {}
    }
    Some(changed)
}

fn limit_killer_clue(
    region: &Region,
    sum: &i32,
    state: &mut SolverState,
    changed: &mut bool,
) -> Option<()> {
    let mut sum_so_far = 0;
    let mut unknown_cells = Vec::new();
    for pos in &region.cells {
        let cell = state.get(pos);
        if let Some(value) = cell.value {
            sum_so_far += value;
        } else {
            unknown_cells.push(pos);
        }
    }
    if sum_so_far > *sum {
        return None;
    }
    if sum_so_far == *sum {
        return if !unknown_cells.is_empty() {
            None
        } else {
            Some(())
        };
    }
    let lowest_sum = state.precomputed.lowest_sums[unknown_cells.len()];
    let highest_sum = state.precomputed.highest_sums[unknown_cells.len()];
    if sum_so_far + highest_sum < *sum {
        return None;
    }
    if sum_so_far + lowest_sum > *sum {
        return None;
    }
    if unknown_cells.len() == 1 {
        let pos = unknown_cells[0];
        let cell = state.get_mut(pos);
        *changed |= cell.limit(|c| sum_so_far + c == *sum)?;
    }
    Some(())
}

fn limit_relationship_clue(
    first: &IVec2,
    second: &IVec2,
    relationship: &Relationship,
    state: &mut SolverState,
    changed: &mut bool,
) -> Option<()> {
    let first_value = state.get(first).value;
    let second_value = state.get(second).value;
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
        let value = state.get(present).value.unwrap();
        let cell = state.get_mut(not_present);
        *changed |= cell.limit(|c| match relationship {
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
    Some(())
}

fn limit_property_clue(
    region: &Region,
    state: &mut SolverState,
    property: &Property,
    changed: &mut bool,
) -> Option<()> {
    for pos in &region.cells {
        let cell = state.get_mut(pos);
        if let Some(value) = cell.value {
            match property {
                Property::Even => {
                    if value % 2 != 0 {
                        return None;
                    }
                }
                Property::Odd => {
                    if value % 2 == 0 {
                        return None;
                    }
                }
                Property::Given(given) => {
                    if *given != value {
                        return None;
                    }
                }
            }
        } else {
            *changed |= cell.limit(|c| match property {
                Property::Even => c % 2 == 0,
                Property::Odd => c % 2 != 0,
                Property::Given(value) => *c == *value,
            })?;
        }
    }
    Some(())
}

fn limit_thermometer_clue(
    line: &Line,
    model: &SudokuModel,
    state: &mut SolverState,
    changed: &mut bool,
) -> Option<()> {
    let len = line.cells.len();
    if len > model.numbers.len() {
        return None;
    }
    let mut offset = 0;
    let min_indices: Vec<usize> = (0..len)
        .map(|i| {
            let cell = state.get(&line.cells[i]);
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
        let cell = state.get_mut(pos);
        if cell.value.is_some() {
            continue;
        }
        *changed |= cell.limit(|c| min_indices[i] <= model.number_indices[c])?;
    }
    Some(())
}

fn limit_unique_clue(region: &Region, state: &mut SolverState, changed: &mut bool) -> Option<()> {
    let mut placed = HashSet::new();
    for pos in &region.cells {
        let cell = state.get_mut(pos);
        if let Some(value) = cell.value {
            if !placed.insert(value) {
                return None;
            }
        }
    }
    for pos in &region.cells {
        let cell = state.get_mut(pos);
        if cell.value.is_some() {
            continue;
        }
        *changed |= cell.limit(|c| !placed.contains(c))?;
    }
    find_obvious_pairs(region, state, changed)?;
    find_hidden_pairs(region, state, changed)?;
    Some(())
}

fn find_obvious_pairs(region: &Region, state: &mut SolverState, changed: &mut bool) -> Option<()> {
    let mut pairs = HashMap::new();
    for pos in &region.cells {
        let cell = state.get(pos);
        if cell.value.is_some() {
            continue;
        }
        let key = cell
            .candidates
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(",");
        let entry = pairs
            .entry(key)
            .or_insert_with(|| (HashSet::new(), cell.candidates.clone()));
        entry.0.insert(pos);
    }
    for (_, (positions, elements)) in &pairs {
        if elements.len() == positions.len() {
            for pos in &region.cells {
                if positions.contains(&pos) {
                    continue;
                }
                let cell = state.get_mut(pos);
                if cell.value.is_some() {
                    continue;
                }
                *changed |= cell.limit(|c| !elements.contains(c))?;
            }
        }
    }
    Some(())
}

fn find_hidden_pairs(region: &Region, state: &mut SolverState, changed: &mut bool) -> Option<()> {
    let possible_numbers = region
        .cells
        .iter()
        .flat_map(|cell| state.get(cell).candidates.clone())
        .collect::<HashSet<i32>>();
    let free_spots = region
        .cells
        .iter()
        .filter(|cell| state.get(cell).value.is_none())
        .collect::<Vec<_>>();
    if possible_numbers.len() < free_spots.len() {
        return None;
    }
    if possible_numbers.len() == free_spots.len() {
        for pos in &free_spots {
            let cell = state.get_mut(pos);
            *changed |= cell.limit(|c| possible_numbers.contains(c))?;
        }
        let mut possible_spots = HashMap::new();
        for pos in &free_spots {
            let cell = state.get(pos);
            for candidate in &cell.candidates {
                possible_spots
                    .entry(*candidate)
                    .or_insert_with(Vec::new)
                    .push(pos);
            }
        }
        let mut possible_spots_inverse = HashMap::new();
        for (num, spots) in &possible_spots {
            let key = spots
                .iter()
                .map(|pos| pos.to_string())
                .collect::<Vec<String>>()
                .join(",");
            possible_spots_inverse
                .entry(key)
                .or_insert_with(|| (Vec::new(), spots.clone()))
                .0
                .push(*num);
        }
        for (_, (numbers, spots)) in &possible_spots_inverse {
            if numbers.len() == spots.len() {
                for pos in spots {
                    let cell = state.get_mut(pos);
                    *changed |= cell.limit(|c| numbers.contains(c))?;
                }
            }
        }
    }
    Some(())
}

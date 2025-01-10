use std::collections::HashSet;

use bevy::{color::palettes::css::LIGHT_SKY_BLUE, prelude::*};
use bevy_prototype_lyon::prelude::*;

use crate::{
    region::{get_edges_polygon, get_line_polygon, get_region_polygon},
    stroke, Direction, MouseWorldPos, UnorderedPair,
};

use super::{
    constraint::{CellEdges, CellLine, CellRegion},
    Grid,
};

#[derive(Event)]
pub struct SelectionChangedEvent;

#[derive(Event)]
pub struct ChangeSelectionTypeEvent(pub SelectionType);

#[derive(Component)]
pub struct Selector {
    line_width: f32,
}

#[derive(Component, Debug, Clone, PartialEq)]
pub enum SelectionType {
    Region,
    Line,
    Edges,
}

pub struct SelectionPlugin<T: IntoSystemSet<M>, M> {
    pub grid_setup: T,
    _marker: std::marker::PhantomData<M>,
}

impl<T: IntoSystemSet<M>, M> SelectionPlugin<T, M> {
    pub fn new(grid_setup: T) -> Self {
        Self {
            grid_setup,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: IntoSystemSet<M> + Send + Sync + 'static + Clone, M: Send + Sync + 'static> Plugin
    for SelectionPlugin<T, M>
{
    fn build(&self, app: &mut App) {
        app.add_event::<SelectionChangedEvent>()
            .add_event::<ChangeSelectionTypeEvent>()
            .add_systems(Startup, setup_selector.after(self.grid_setup.clone()))
            .add_systems(
                Update,
                (
                    select_handler,
                    handle_selection_changed_event,
                    handle_change_selection_type_event,
                ),
            );
    }
}

pub fn setup_selector(mut commands: Commands, grid_entity: Query<Entity, With<Grid>>) {
    let grid = grid_entity.single();
    let selector = Selector { line_width: 0.15 };
    let selector_entity = commands
        .spawn((
            ShapeBundle {
                path: PathBuilder::new().build(),
                transform: Transform::from_translation(Vec3::ZERO),
                ..Default::default()
            },
            stroke(Color::Srgba(LIGHT_SKY_BLUE), selector.line_width, true),
            selector,
            SelectionType::Region,
            CellRegion {
                cells: HashSet::new(),
            },
            CellLine {
                cells: Vec::new(),
                cell_set: HashSet::new(),
            },
            CellEdges {
                edges: HashSet::new(),
            },
        ))
        .id();
    commands.entity(grid).add_child(selector_entity);
}

pub fn select_handler(
    mouse_world: Res<MouseWorldPos>,
    mut q_selection: Query<(
        &Selector,
        &SelectionType,
        &mut CellRegion,
        &mut CellLine,
        &mut CellEdges,
    )>,
    q_grid: Query<(&Grid, &GlobalTransform)>,
    mut ev_selection_changed: EventWriter<SelectionChangedEvent>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    keybord_button_input: Res<ButtonInput<KeyCode>>,
) {
    let (_, selection_type, mut region_selection, mut line_selection, mut edge_selection) =
        q_selection.single_mut();
    let mut changed = false;
    if mouse_button_input.just_pressed(MouseButton::Left) {
        if !keybord_button_input.pressed(KeyCode::ShiftLeft) {
            match selection_type {
                SelectionType::Region => {
                    changed |= region_selection.cells.len() > 0;
                    region_selection.cells.clear();
                }
                SelectionType::Line => {
                    changed |= line_selection.cells.len() > 0;
                    line_selection.cells.clear();
                    line_selection.cell_set.clear();
                }
                SelectionType::Edges => {
                    changed |= edge_selection.edges.len() > 0;
                    edge_selection.edges.clear();
                }
            }
        }
    }

    'handler: {
        if mouse_button_input.pressed(MouseButton::Left) {
            let (grid, grid_transform) = q_grid.single();
            let mouse_grid_space = grid_transform
                .affine()
                .inverse()
                .transform_point(mouse_world.0.extend(0.))
                .truncate();
            if *selection_type != SelectionType::Edges {
                let cell_pos = mouse_grid_space.round().as_ivec2();
                if (cell_pos.as_vec2() - mouse_grid_space).length() > 0.6 {
                    break 'handler;
                }
                if !(cell_pos.x >= 0
                    && cell_pos.x < grid.size.x
                    && cell_pos.y >= 0
                    && cell_pos.y < grid.size.y)
                {
                    break 'handler;
                }

                match selection_type {
                    SelectionType::Region => {
                        changed |= region_selection.cells.insert(cell_pos);
                    }
                    SelectionType::Line => {
                        if line_selection.cells.len() == 0 {
                            line_selection.cells.push(cell_pos);
                            line_selection.cell_set.insert(cell_pos);
                            changed = true;
                            break 'handler;
                        }
                        let prev = *line_selection.cells.last().unwrap();
                        let Some(points) = subdivide_line_step(prev, cell_pos) else {
                            break 'handler;
                        };

                        if points.iter().all(|p| !line_selection.cell_set.contains(p)) {
                            line_selection.cells.extend(&points);
                            line_selection.cell_set.extend(&points);
                            changed = true;
                        } else if points.len() == 1 && line_selection.cells.len() > 1 {
                            let point = points[0];
                            let pre_prev = line_selection.cells[line_selection.cells.len() - 2];
                            if point == pre_prev {
                                line_selection.cells.pop();
                                line_selection.cell_set.remove(&prev);
                                changed = true;
                            }
                        }
                    }
                    SelectionType::Edges => unreachable!(),
                }
            } else {
                let corner_pos = mouse_grid_space.ceil().as_ivec2();
                let dir = mouse_grid_space - corner_pos.as_vec2() + Vec2::splat(0.5);
                let cardinal_dir = Direction::closest_from_vec2(dir);
                let (left, right) = match cardinal_dir {
                    Direction::Up => (corner_pos - IVec2::new(1, 0), corner_pos),
                    Direction::Right => (corner_pos - IVec2::new(0, 1), corner_pos),
                    Direction::Down => {
                        (corner_pos - IVec2::new(1, 1), corner_pos - IVec2::new(0, 1))
                    }
                    Direction::Left => {
                        (corner_pos - IVec2::new(1, 1), corner_pos - IVec2::new(1, 0))
                    }
                };
                if ((left.as_vec2() + right.as_vec2()) / 2. - mouse_grid_space).length() > 0.3 {
                    break 'handler;
                }
                if (left.x >= 0 && left.x < grid.size.x && left.y >= 0 && left.y < grid.size.y)
                    || (right.x >= 0
                        && right.x < grid.size.x
                        && right.y >= 0
                        && right.y < grid.size.y)
                {
                    let pair = UnorderedPair::new(left, right);
                    changed |= edge_selection.edges.insert(pair);
                }
            }
        }
    }

    if changed {
        ev_selection_changed.send(SelectionChangedEvent);
    }
}

pub fn handle_selection_changed_event(
    mut selection_changed_event: EventReader<SelectionChangedEvent>,
    mut q_selector: Query<(
        &Selector,
        &mut Path,
        &SelectionType,
        &CellRegion,
        &CellLine,
        &CellEdges,
    )>,
) {
    let (
        selector,
        mut selector_path,
        selection_type,
        region_selection,
        line_selection,
        edge_selection,
    ) = q_selector.single_mut();
    let all_events = selection_changed_event.read().collect::<Vec<_>>();
    if all_events.len() == 0 {
        return;
    }

    match selection_type {
        SelectionType::Region => {
            let path = get_region_polygon(&region_selection.cells, selector.line_width / 2.);
            selector_path.0 = path.0;
        }
        SelectionType::Line => {
            let path = get_line_polygon(&line_selection.cells);
            selector_path.0 = path.0;
        }
        SelectionType::Edges => {
            let path = get_edges_polygon(&edge_selection.edges);
            selector_path.0 = path.0;
        }
    }
}

pub fn handle_change_selection_type_event(
    mut ev_change_selection_type: EventReader<ChangeSelectionTypeEvent>,
    mut q_selector: Query<(
        &Selector,
        &mut SelectionType,
        &mut CellRegion,
        &mut CellLine,
        &mut CellEdges,
    )>,
    mut ev_selection_changed: EventWriter<SelectionChangedEvent>,
) {
    let (_, mut selection_type, mut cell_region, mut cell_line, mut cell_edges) =
        q_selector.single_mut();

    for event in ev_change_selection_type.read() {
        *selection_type = event.0.clone();
        cell_region.cells.clear();
        cell_line.cells.clear();
        cell_line.cell_set.clear();
        cell_edges.edges.clear();
        ev_selection_changed.send(SelectionChangedEvent);
    }
}

fn subdivide_line_step(from_excluded: IVec2, to_included: IVec2) -> Option<Vec<IVec2>> {
    let diff = to_included - from_excluded;
    let abs_diff = diff.abs();
    if abs_diff.x != 0 && abs_diff.y != 0 && abs_diff.x != abs_diff.y {
        return None;
    }
    if abs_diff.x == 0 && abs_diff.y == 0 {
        return None;
    }
    let steps = abs_diff.x.max(abs_diff.y);
    let step = diff.signum();

    let mut result = Vec::new();
    for i in 1..=steps {
        result.push(from_excluded + step * i);
    }
    Some(result)
}

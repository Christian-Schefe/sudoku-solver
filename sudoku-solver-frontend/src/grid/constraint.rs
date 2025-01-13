use std::collections::HashSet;

use bevy::{prelude::*, utils::HashMap};
use bevy_prototype_lyon::prelude::*;
use sudoku_solver::model::constraint::Relationship;

use crate::{
    make_fill,
    region::{get_line_polygon, get_region_polygon},
    make_stroke, UnorderedPair,
};

use super::{
    selection::{SelectionType, Selector},
    Fonts, Grid,
};

#[derive(Event)]
pub enum SpawnConstraintEvent {
    KillerCage(i32),
    Thermometer,
    Unique,
    Relationship(Relationship),
}

#[derive(Component)]
pub struct ConstraintKillerCage(i32);

#[derive(Component)]
pub struct ConstraintThermometer;

#[derive(Component)]
pub struct ConstraintUnique;

#[derive(Component)]
pub struct ConstraintRelationship(Relationship);

#[derive(Component)]
pub struct CellRegion {
    pub cells: HashSet<IVec2>,
}

#[derive(Component)]
pub struct CellLine {
    pub cells: Vec<IVec2>,
    pub cell_set: HashSet<IVec2>,
}

#[derive(Component)]
pub struct CellEdges {
    pub edges: HashSet<UnorderedPair>,
}

#[derive(Component)]
pub struct Constraints {
    pub killer_cages: HashMap<IVec2, Entity>,
    pub thermometers: HashMap<IVec2, Entity>,
    pub uniques: HashMap<IVec2, Entity>,
    pub relationships: HashMap<UnorderedPair, Entity>,
}

pub fn constraints_plugin(app: &mut App) {
    app.add_event::<SpawnConstraintEvent>()
        .add_systems(Startup, setup_constraints)
        .add_systems(
            Update,
            (
                handle_spawn_killer_cage,
                handle_spawn_thermometer,
                handle_spawn_unique,
                handle_spawn_relationships,
            ),
        );
}

fn setup_constraints(mut commands: Commands) {
    commands.spawn((Constraints {
        killer_cages: HashMap::default(),
        thermometers: HashMap::default(),
        uniques: HashMap::default(),
        relationships: HashMap::default(),
    },));
}

fn handle_spawn_killer_cage(
    mut commands: Commands,
    mut constraints: Query<&mut Constraints>,
    fonts: Res<Fonts>,
    q_grid: Query<(Entity, &Grid)>,
    q_selection: Query<(&Selector, &SelectionType, &CellRegion)>,
    mut ev_spawn_constraint: EventReader<SpawnConstraintEvent>,
) {
    let Ok((_, SelectionType::Region, selection)) = q_selection.get_single() else {
        ev_spawn_constraint.read().for_each(drop);
        return;
    };
    let mut constraints = constraints.single_mut();
    for event in ev_spawn_constraint.read() {
        let SpawnConstraintEvent::KillerCage(sum) = event else {
            continue;
        };
        if !is_cardinally_connected(&selection.cells) {
            continue;
        }
        if selection
            .cells
            .iter()
            .any(|cell| constraints.killer_cages.contains_key(cell))
        {
            continue;
        }
        let path = get_region_polygon(&selection.cells, 0.1);

        let cage = commands
            .spawn((
                ShapeBundle {
                    path,
                    transform: Transform::from_translation(Vec3::ZERO.with_z(0.1)),
                    ..Default::default()
                },
                make_stroke(Color::BLACK, 0.02, false),
            ))
            .id();

        let mut sorted_cells = selection.cells.iter().cloned().collect::<Vec<_>>();
        sorted_cells.sort_by(killer_cage_ordering);
        let text_pos = sorted_cells.first().unwrap();
        let vec = Vec3::new(text_pos.x as f32, text_pos.y as f32, 0.) + Vec3::new(-0.25, 0.25, 0.1);

        let cage_text = commands
            .spawn((
                Text2d(sum.to_string()),
                Transform::from_translation(vec).with_scale(Vec3::splat(0.0033)),
                TextFont {
                    font: fonts.fira_mono.clone(),
                    font_size: 60.0,
                    ..Default::default()
                },
                TextColor(Color::BLACK),
                ConstraintKillerCage(*sum),
                CellRegion {
                    cells: selection.cells.clone(),
                },
            ))
            .id();

        let (grid_entity, _) = q_grid.single();
        commands.entity(grid_entity).add_child(cage);
        commands.entity(cage).add_child(cage_text);

        for cell in &selection.cells {
            constraints.killer_cages.insert(*cell, cage);
        }
    }
}

fn killer_cage_ordering(a: &IVec2, b: &IVec2) -> std::cmp::Ordering {
    (a.x - a.y).cmp(&(b.x - b.y)).then(b.y.cmp(&a.y))
}

fn is_cardinally_connected(set: &HashSet<IVec2>) -> bool {
    let mut visited = HashSet::new();
    let mut stack = vec![set.iter().next().unwrap().clone()];
    while let Some(current) = stack.pop() {
        visited.insert(current);
        for dir in [
            IVec2::new(0, 1),
            IVec2::new(1, 0),
            IVec2::new(0, -1),
            IVec2::new(-1, 0),
        ] {
            let next = current + dir;
            if set.contains(&next) && !visited.contains(&next) {
                stack.push(next);
            }
        }
    }
    visited.len() == set.len()
}

fn handle_spawn_thermometer(
    mut commands: Commands,
    mut constraints: Query<&mut Constraints>,
    q_grid: Query<(Entity, &Grid)>,
    q_selection: Query<(&Selector, &SelectionType, &CellLine)>,
    mut ev_spawn_constraint: EventReader<SpawnConstraintEvent>,
) {
    let Ok((_, SelectionType::Line, selection)) = q_selection.get_single() else {
        ev_spawn_constraint.read().for_each(drop);
        return;
    };
    let mut constraints = constraints.single_mut();
    for event in ev_spawn_constraint.read() {
        let SpawnConstraintEvent::Thermometer = event else {
            continue;
        };
        if selection
            .cells
            .iter()
            .any(|cell| constraints.thermometers.contains_key(cell))
        {
            continue;
        }
        let path = get_line_polygon(&selection.cells);
        let circle = GeometryBuilder::build_as(&shapes::Circle {
            center: selection.cells.first().unwrap().as_vec2(),
            radius: 0.25,
        });
        let color = Color::srgb(0.8, 0.8, 0.8);

        let thermometer = commands
            .spawn((
                ShapeBundle {
                    path,
                    transform: Transform::from_translation(Vec3::ZERO.with_z(0.05)),
                    ..Default::default()
                },
                make_stroke(color, 0.125, true),
                ConstraintThermometer,
            ))
            .id();

        let circle = commands
            .spawn((
                ShapeBundle {
                    path: circle,
                    transform: Transform::from_translation(Vec3::ZERO.with_z(0.05)),
                    ..Default::default()
                },
                make_fill(color),
            ))
            .id();

        let (grid_entity, _) = q_grid.single();
        commands.entity(grid_entity).add_child(thermometer);
        commands.entity(thermometer).add_child(circle);

        for cell in &selection.cells {
            constraints.thermometers.insert(*cell, thermometer);
        }
    }
}

fn handle_spawn_unique(
    mut commands: Commands,
    mut constraints: Query<&mut Constraints>,
    q_grid: Query<(Entity, &Grid)>,
    q_selection: Query<(&Selector, &SelectionType, &CellRegion)>,
    mut ev_spawn_constraint: EventReader<SpawnConstraintEvent>,
) {
    let Ok((_, SelectionType::Region, selection)) = q_selection.get_single() else {
        ev_spawn_constraint.read().for_each(drop);
        return;
    };
    let mut constraints = constraints.single_mut();
    for event in ev_spawn_constraint.read() {
        let SpawnConstraintEvent::Unique = event else {
            continue;
        };
        if !is_cardinally_connected(&selection.cells) {
            continue;
        }
        if selection
            .cells
            .iter()
            .any(|cell| constraints.uniques.contains_key(cell))
        {
            continue;
        }
        let path = get_region_polygon(&selection.cells, 0.);

        let unique_region = commands
            .spawn((
                ShapeBundle {
                    path,
                    transform: Transform::from_translation(Vec3::ZERO.with_z(0.04)),
                    ..Default::default()
                },
                make_stroke(Color::BLACK, 0.04, false),
                ConstraintUnique,
            ))
            .id();

        let (grid_entity, _) = q_grid.single();
        commands.entity(grid_entity).add_child(unique_region);

        for cell in &selection.cells {
            constraints.uniques.insert(*cell, unique_region);
        }
    }
}

fn handle_spawn_relationships(
    mut commands: Commands,
    mut constraints: Query<&mut Constraints>,
    q_grid: Query<(Entity, &Grid)>,
    q_selection: Query<(&Selector, &SelectionType, &CellEdges)>,
    mut ev_spawn_constraint: EventReader<SpawnConstraintEvent>,
) {
    let Ok((_, SelectionType::Edges, selection)) = q_selection.get_single() else {
        ev_spawn_constraint.read().for_each(drop);
        return;
    };
    let mut constraints = constraints.single_mut();
    for event in ev_spawn_constraint.read() {
        let SpawnConstraintEvent::Relationship(relationship) = event else {
            continue;
        };
        for edge in &selection.edges {
            if constraints.relationships.contains_key(edge) {
                continue;
            }
            let center_pos = (edge.0.as_vec2() + edge.1.as_vec2()) / 2.;
            let relationship_entity = match relationship {
                Relationship::Consecutive => commands
                    .spawn((
                        ShapeBundle {
                            path: GeometryBuilder::build_as(&shapes::Circle {
                                center: center_pos,
                                radius: 0.15,
                            }),
                            transform: Transform::from_translation(Vec3::ZERO.with_z(0.15)),
                            ..Default::default()
                        },
                        make_stroke(Color::BLACK, 0.02, false),
                        make_fill(Color::WHITE),
                        ConstraintRelationship(relationship.clone()),
                    ))
                    .id(),
                Relationship::Double => commands
                    .spawn((
                        ShapeBundle {
                            path: GeometryBuilder::build_as(&shapes::Circle {
                                center: center_pos,
                                radius: 0.15,
                            }),
                            transform: Transform::from_translation(Vec3::ZERO.with_z(0.15)),
                            ..Default::default()
                        },
                        make_fill(Color::BLACK),
                        ConstraintRelationship(relationship.clone()),
                    ))
                    .id(),
                _ => {
                    continue;
                }
            };

            let (grid_entity, _) = q_grid.single();
            commands.entity(grid_entity).add_child(relationship_entity);

            constraints.relationships.insert(*edge, relationship_entity);
        }
    }
}

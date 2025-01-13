use bevy::{
    input::{
        keyboard::{Key, KeyboardInput},
        ButtonState,
    },
    prelude::*,
};
use bevy_prototype_lyon::prelude::*;
use constraint::{CellRegion, SpawnConstraintEvent};
use selection::{ChangeSelectionTypeEvent, SelectionType, Selector};
use sudoku_solver::model::constraint::Relationship;

use crate::make_stroke;

mod constraint;
mod selection;
mod menu;

#[derive(Component)]
struct Grid {
    cells: Vec<Vec<Entity>>,
    size: IVec2,
}

#[derive(Component)]
struct Cell;

#[derive(Resource)]
struct Fonts {
    fira_mono: Handle<Font>,
}

pub fn grid_plugin(app: &mut App) {
    app.add_plugins((
        selection::SelectionPlugin::new(setup_grid),
        constraint::constraints_plugin,
        menu::menu_plugin,
    ))
    .add_systems(PreStartup, setup_fonts)
    .add_systems(Startup, setup_grid)
    .add_systems(Update, (handle_type_number, handle_keyboard_input_debug));
}

fn setup_fonts(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("FiraMono-Medium.ttf");
    commands.insert_resource(Fonts {
        fira_mono: font.clone(),
    });
}

pub fn setup_grid(mut commands: Commands, fonts: Res<Fonts>) {
    let mut cells = Vec::new();
    let size = IVec2::new(9, 9);
    let center = Vec3::new(size.x as f32 / 2., size.y as f32 / 2., 0.) - Vec3::ONE * 0.5;

    for y in 0..size.y {
        let mut row_cells = Vec::new();
        for x in 0..size.x {
            let vec = Vec3::new(x as f32, y as f32, 1.);
            let cell = commands
                .spawn((
                    Cell,
                    Text2d("".to_string()),
                    TextFont {
                        font: fonts.fira_mono.clone(),
                        font_size: 60.0,
                        ..Default::default()
                    },
                    TextColor(Color::BLACK),
                    Transform::from_translation(vec).with_scale(Vec3::splat(0.01)),
                ))
                .id();
            row_cells.push(cell);
        }
        cells.push(row_cells);
    }
    let mut grid = commands.spawn((
        Grid {
            cells: cells.clone(),
            size,
        },
        Transform::from_translation(-center),
        Visibility::Inherited,
    ));
    let grid_entity = grid.id();
    for row in cells {
        grid.add_children(&row);
    }

    setup_grid_lines(&mut commands, grid_entity, size);
}

fn setup_grid_lines(commands: &mut Commands, grid_entity: Entity, size: IVec2) {
    for row in 0..=size.y {
        setup_line(commands, grid_entity, size, row, true);
    }
    for col in 0..=size.x {
        setup_line(commands, grid_entity, size, col, false);
    }
}

fn setup_line(commands: &mut Commands, grid: Entity, size: IVec2, index: i32, is_row: bool) {
    let mut path_builder = PathBuilder::new();
    path_builder.move_to(Vec2::new(0., 0.));
    path_builder.line_to(if is_row {
        Vec2::new(size.x as f32, 0.)
    } else {
        Vec2::new(0., size.y as f32)
    });
    let path = path_builder.build();
    let pos = if is_row {
        Vec3::new(-0.5, index as f32 - 0.5, -1.)
    } else {
        Vec3::new(index as f32 - 0.5, -0.5, -1.)
    };
    let line = commands
        .spawn((
            ShapeBundle {
                path,
                transform: Transform::from_translation(pos),
                ..Default::default()
            },
            make_stroke(Color::BLACK, 0.02, true),
        ))
        .id();
    commands.entity(grid).add_child(line);
}

fn handle_type_number(
    q_selection: Query<(&Selector, &CellRegion)>,
    q_grid: Query<&Grid>,
    mut q_cells: Query<(&Cell, &mut Text2d)>,
    mut keybord_button_input: EventReader<KeyboardInput>,
) {
    let grid = q_grid.single();
    for event in keybord_button_input.read() {
        if event.state == ButtonState::Released {
            continue;
        }
        let Ok((_, selection)) = q_selection.get_single() else {
            return;
        };
        let typed = match &event.logical_key {
            Key::Character(input) => {
                if input.chars().any(|c| c.is_control()) {
                    continue;
                }
                Some(input.to_string())
            }
            Key::Backspace => None,
            _ => continue,
        };
        for cell_pos in &selection.cells {
            let cell_entity = grid.cells[cell_pos.y as usize][cell_pos.x as usize];
            let (_, mut cell_text) = q_cells.get_mut(cell_entity).unwrap();
            if let Some(typed) = &typed {
                if typed.len() == 1 && typed.chars().all(|c| c.is_ascii_digit()) {
                    cell_text.0 = typed.clone();
                }
            } else {
                cell_text.0 = "".to_string();
            }
        }
    }
}

fn handle_keyboard_input_debug(
    keybord_button_input: Res<ButtonInput<KeyCode>>,
    mut ev_spawn_constraint: EventWriter<SpawnConstraintEvent>,
    mut ev_change_selection_type: EventWriter<ChangeSelectionTypeEvent>,
    q_selection_type: Query<&SelectionType>,
) {
    if keybord_button_input.just_pressed(KeyCode::Numpad1) {
        ev_spawn_constraint.send(SpawnConstraintEvent::KillerCage(20));
    }
    if keybord_button_input.just_pressed(KeyCode::Numpad2) {
        ev_spawn_constraint.send(SpawnConstraintEvent::Thermometer);
    }
    if keybord_button_input.just_pressed(KeyCode::Numpad3) {
        ev_spawn_constraint.send(SpawnConstraintEvent::Unique);
    }
    if keybord_button_input.just_pressed(KeyCode::Numpad4) {
        ev_spawn_constraint.send(SpawnConstraintEvent::Relationship(
            Relationship::Double,
        ));
    }
    if keybord_button_input.just_pressed(KeyCode::KeyL) {
        ev_change_selection_type.send(ChangeSelectionTypeEvent(SelectionType::Line));
    }
    if keybord_button_input.just_pressed(KeyCode::KeyR) {
        ev_change_selection_type.send(ChangeSelectionTypeEvent(SelectionType::Region));
    }
    if keybord_button_input.just_pressed(KeyCode::KeyE) {
        ev_change_selection_type.send(ChangeSelectionTypeEvent(SelectionType::Edges));
    }
    if keybord_button_input.just_pressed(KeyCode::Space) {
        let selection_type = q_selection_type.single();
        ev_change_selection_type.send(ChangeSelectionTypeEvent(match selection_type {
            SelectionType::Region => SelectionType::Line,
            SelectionType::Line => SelectionType::Edges,
            SelectionType::Edges => SelectionType::Region,
        }));
    }
}

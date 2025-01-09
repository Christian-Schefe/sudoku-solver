use std::collections::HashSet;

use bevy::{
    color::palettes::css::LIGHT_SKY_BLUE,
    input::{
        keyboard::{Key, KeyboardInput},
        ButtonState,
    },
    prelude::*,
};
use bevy_prototype_lyon::prelude::*;
use sudoku_solver::model::region::Region;

use crate::{region::get_region_polygon, MouseWorldPos};

#[derive(Component)]
struct Grid {
    cells: Vec<Vec<Entity>>,
    model: sudoku_solver::model::SudokuModel,
}

#[derive(Resource)]
struct Selection {
    cells: HashSet<IVec2>,
}

#[derive(Event)]
struct SelectionChangedEvent;

#[derive(Component)]
struct Cell;

#[derive(Component)]
struct Selector {
    line_width: f32,
}

#[derive(Event)]
enum SpawnConstraintEvent {
    KillerCage(i32),
}

#[derive(Resource)]
struct Fonts {
    fira_mono: Handle<Font>,
}

pub fn grid_plugin(app: &mut App) {
    app.insert_resource(Selection {
        cells: HashSet::new(),
    })
    .add_event::<SelectionChangedEvent>()
    .add_event::<SpawnConstraintEvent>()
    .add_systems(Startup, setup_grid)
    .add_systems(
        Update,
        (
            select_handler,
            handle_selection_changed_event,
            handle_type_number,
            handle_spawn_killer_cage,
            handle_keyboard_input_debug,
        ),
    );
}

fn setup_grid(mut commands: Commands, asset_server: Res<AssetServer>) {
    let model = sudoku_solver::example::killer_test_model(None).build_model();
    let mut cells = Vec::new();
    let size = model.size;
    let center = Vec3::new(size.x as f32 / 2., size.y as f32 / 2., 0.) - Vec3::ONE * 0.5;

    let font = asset_server.load("FiraMono-Medium.ttf");
    commands.insert_resource(Fonts {
        fira_mono: font.clone(),
    });

    for y in 0..size.y {
        let mut row_cells = Vec::new();
        for x in 0..size.x {
            let vec = Vec3::new(x as f32, y as f32, 1.);
            let cell = commands
                .spawn((
                    Cell,
                    Text2d("".to_string()),
                    TextFont {
                        font: font.clone(),
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
            model,
        },
        Transform::from_translation(-center),
        Visibility::Inherited,
    ));
    let grid_entity = grid.id();
    for row in cells {
        grid.add_children(&row);
    }

    let selector = Selector { line_width: 0.15 };
    let selector_entity = commands
        .spawn((
            ShapeBundle {
                path: PathBuilder::new().build(),
                transform: Transform::from_translation(Vec3::ZERO),
                ..Default::default()
            },
            Stroke {
                color: Color::Srgba(LIGHT_SKY_BLUE),
                options: StrokeOptions::default().with_line_width(selector.line_width),
            },
            Fill::color(Color::NONE),
            selector,
        ))
        .id();
    commands.entity(grid_entity).add_child(selector_entity);
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
            Stroke::new(Color::BLACK, 0.02),
            Fill::color(Color::NONE),
        ))
        .id();
    commands.entity(grid).add_child(line);
}

fn select_handler(
    mouse_world: Res<MouseWorldPos>,
    mut selection: ResMut<Selection>,
    q_grid: Query<(&Grid, &GlobalTransform)>,
    mut ev_selection_changed: EventWriter<SelectionChangedEvent>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    keybord_button_input: Res<ButtonInput<KeyCode>>,
) {
    let mut changed = false;
    if mouse_button_input.just_pressed(MouseButton::Left) {
        if !keybord_button_input.pressed(KeyCode::ShiftLeft) {
            selection.cells.clear();
            changed = true;
        }
    }

    if mouse_button_input.pressed(MouseButton::Left) {
        let (grid, grid_transform) = q_grid.single();
        let mouse_grid_space = grid_transform
            .affine()
            .inverse()
            .transform_point(mouse_world.0.extend(0.));
        let cell_pos = mouse_grid_space.truncate().round().as_ivec2();
        if cell_pos.x >= 0
            && cell_pos.x < grid.model.size.x
            && cell_pos.y >= 0
            && cell_pos.y < grid.model.size.y
        {
            add_to_selection(cell_pos, &mut selection, &mut changed);
        }
    }

    if changed {
        ev_selection_changed.send(SelectionChangedEvent);
    }
}

fn add_to_selection(pos: IVec2, selection: &mut Selection, ev_selection_changed: &mut bool) {
    if selection.cells.insert(pos) {
        *ev_selection_changed = true;
    }
}

fn handle_selection_changed_event(
    selection: Res<Selection>,
    mut selection_changed_event: EventReader<SelectionChangedEvent>,
    mut q_selector: Query<(&Selector, &mut Path)>,
) {
    for _ in selection_changed_event.read() {
        let (selector, mut selector_path) = q_selector.single_mut();
        let path = get_region_polygon(
            &Region {
                cells: selection.cells.iter().cloned().collect(),
            },
            selector.line_width / 2.,
        );
        selector_path.0 = path.0;
    }
}

fn handle_type_number(
    selection: Res<Selection>,
    q_grid: Query<&Grid>,
    mut q_cells: Query<(&Cell, &mut Text2d)>,
    mut keybord_button_input: EventReader<KeyboardInput>,
) {
    let grid = q_grid.single();
    for event in keybord_button_input.read() {
        if event.state == ButtonState::Released {
            continue;
        }
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
                if typed.len() == 1 && typed.chars().all(|c| c.is_ascii_alphanumeric()) {
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
) {
    if keybord_button_input.just_pressed(KeyCode::Digit1) {
        ev_spawn_constraint.send(SpawnConstraintEvent::KillerCage(20));
    }
}

fn handle_spawn_killer_cage(
    mut commands: Commands,
    fonts: Res<Fonts>,
    q_grid: Query<(Entity, &Grid)>,
    selection: ResMut<Selection>,
    mut ev_spawn_constraint: EventReader<SpawnConstraintEvent>,
) {
    for event in ev_spawn_constraint.read() {
        let sum = match event {
            SpawnConstraintEvent::KillerCage(sum) => sum,
        };
        let region = Region {
            cells: selection.cells.iter().cloned().collect(),
        };
        let path = get_region_polygon(&region, 0.1);

        let cage = commands
            .spawn((
                ShapeBundle {
                    path,
                    transform: Transform::from_translation(Vec3::ZERO.with_z(0.1)),
                    ..Default::default()
                },
                Stroke::new(Color::BLACK, 0.02),
                Fill::color(Color::NONE),
            ))
            .id();

        let mut sorted_cells = region.cells.into_iter().collect::<Vec<_>>();
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
            ))
            .id();

        let (grid_entity, _) = q_grid.single();
        commands.entity(grid_entity).add_child(cage);
        commands.entity(cage).add_child(cage_text);
    }
}

fn killer_cage_ordering(a: &IVec2, b: &IVec2) -> std::cmp::Ordering {
    b.y.cmp(&a.y).then(a.x.cmp(&b.x))
}

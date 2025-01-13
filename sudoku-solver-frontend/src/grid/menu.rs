use crate::grid::selection::{ChangeSelectionTypeEvent, SelectionType};
use crate::grid::{setup_grid, Grid};
use crate::{make_fill, Container, MouseWorldPos};
use bevy::color::palettes::css::GRAY;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

#[derive(Component)]
#[require(ButtonState, ButtonAction)]
pub struct Button {
    pub bounds: Rect,
}

#[derive(Component, PartialEq, Debug)]
pub enum ButtonState {
    Normal,
    Hover,
    Pressed,
}

#[derive(Component, PartialEq, Debug)]
pub enum ButtonAction {
    Idle,
    Down,
    Up,
}

impl Default for ButtonState {
    fn default() -> Self {
        ButtonState::Normal
    }
}

impl Default for ButtonAction {
    fn default() -> Self {
        ButtonAction::Idle
    }
}

pub fn menu_plugin(app: &mut App) {
    app.add_systems(Startup, setup_buttons.after(setup_grid))
        .add_systems(
            Update,
            (
                handle_button_state,
                handle_button_state_change,
                handle_button_action_change,
            ),
        );
}

pub fn setup_buttons(mut commands: Commands, q_grid: Query<(Entity, &Grid)>) {
    let grid = q_grid.single().0;
    create_button(
        Vec2::new(-2.0, 8.0),
        Vec2::new(0.8, 0.8),
        &mut commands,
        grid.clone(),
        Some(SelectionType::Region),
    );
    create_button(
        Vec2::new(-2.0, 7.0),
        Vec2::new(0.8, 0.8),
        &mut commands,
        grid.clone(),
        Some(SelectionType::Line),
    );
    create_button(
        Vec2::new(-2.0, 6.0),
        Vec2::new(0.8, 0.8),
        &mut commands,
        grid.clone(),
        Some(SelectionType::Edges),
    );
}

fn create_button<T: 'static + Send + Sync>(
    pos: Vec2,
    size: Vec2,
    commands: &mut Commands,
    grid: Entity,
    data: Option<T>,
) {
    let shape = shapes::Rectangle {
        extents: size,
        origin: RectangleOrigin::Center,
        radii: None,
    };
    let path = GeometryBuilder::build_as(&shape);
    let mut button = commands.spawn((
        Button {
            bounds: Rect::from_corners(-size / 2.0, size / 2.0),
        },
        ShapeBundle {
            path,
            transform: Transform::from_translation(Vec3::new(pos.x, pos.y, 1.0)),
            ..Default::default()
        },
        make_fill(Color::Srgba(GRAY)),
    ));
    if let Some(data) = data {
        button.insert(Container(data));
    }
    let button = button.id();
    commands.entity(grid).add_child(button);
}

fn handle_button_state_change(
    mut q_button: Query<(&Button, &ButtonState, &mut Fill), Changed<ButtonState>>,
) {
    for (_, state, mut fill) in q_button.iter_mut() {
        let color = match state {
            ButtonState::Normal => Color::Srgba(Srgba::rgb(0.8, 0.8, 0.8)),
            ButtonState::Hover => Color::Srgba(Srgba::rgb(0.75, 0.75, 0.75)),
            ButtonState::Pressed => Color::Srgba(Srgba::rgb(0.7, 0.7, 0.7)),
        };
        fill.color = color;
    }
}

fn handle_button_action_change(
    mut q_button: Query<(&Button, &Container<SelectionType>, &ButtonAction), Changed<ButtonAction>>,
    mut ev_set_selection_type: EventWriter<ChangeSelectionTypeEvent>,
) {
    for (_, container, state) in q_button.iter() {
        if *state == ButtonAction::Up {
            ev_set_selection_type.send(ChangeSelectionTypeEvent(container.0.clone()));
        }
    }
}

fn handle_button_state(
    mouse_world: Res<MouseWorldPos>,
    mut q_button: Query<(&Button, &Transform, &mut ButtonState, &mut ButtonAction)>,
    q_grid: Query<(&Grid, &GlobalTransform)>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
) {
    let (_, grid_transform) = q_grid.single();
    let mouse_grid_space = grid_transform
        .affine()
        .inverse()
        .transform_point(mouse_world.0.extend(0.))
        .truncate();

    for ((button, button_transform, mut button_state, mut button_action)) in q_button.iter_mut() {
        let button_pos = button_transform.translation.truncate();
        let rect = Rect::from_corners(
            button_pos + button.bounds.min,
            button_pos + button.bounds.max,
        );
        let mut new_action = ButtonAction::Idle;

        let new_state = if rect.contains(mouse_grid_space) {
            if mouse_button_input.just_pressed(MouseButton::Left) {
                new_action = ButtonAction::Down;
                ButtonState::Pressed
            } else {
                if *button_state == ButtonState::Pressed {
                    if mouse_button_input.just_released(MouseButton::Left) {
                        new_action = ButtonAction::Up;
                    }
                    if mouse_button_input.pressed(MouseButton::Left) {
                        ButtonState::Pressed
                    } else {
                        ButtonState::Hover
                    }
                } else {
                    ButtonState::Hover
                }
            }
        } else {
            if *button_state == ButtonState::Pressed
                && mouse_button_input.pressed(MouseButton::Left)
            {
                ButtonState::Pressed
            } else {
                ButtonState::Normal
            }
        };

        if *button_state != new_state {
            *button_state = new_state;
        }

        if *button_action != new_action {
            *button_action = new_action;
        }
    }
}

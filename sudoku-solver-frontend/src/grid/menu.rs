use crate::grid::selection::{ChangeSelectionTypeEvent, SelectionType};
use crate::grid::{setup_grid, Grid};
use crate::{make_fill, make_stroke, Container, MouseWorldPos};
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

#[derive(Component)]
#[require(ButtonState, ButtonAction)]
pub struct Button {
    pub bounds: Rect,
}

#[derive(Component, PartialEq, Debug)]
pub enum ButtonState {
    Disabled,
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

#[derive(Component)]
pub struct RadioGroup {
    pub selected: Option<Entity>,
}

#[derive(Component)]
#[require(RadioButtonState)]
pub struct RadioButton {
    pub group: Entity,
}

#[derive(Component)]
pub enum RadioButtonState {
    Unselected,
    Selected,
}

impl Default for RadioButtonState {
    fn default() -> Self {
        RadioButtonState::Unselected
    }
}

pub fn menu_plugin(app: &mut App) {
    app.add_systems(Startup, setup_buttons.after(setup_grid))
        .add_systems(
            Update,
            (
                handle_button_state,
                handle_button_state_change.after(handle_button_state),
                handle_button_action_change.after(handle_button_state),
                handle_radio_groups,
                handle_button_radio_change.after(handle_radio_groups),
            ),
        );
}

pub fn setup_buttons(mut commands: Commands, q_grid: Query<(Entity, &Grid)>) {
    let grid = q_grid.single().0;
    let radio_group = commands.spawn(RadioGroup { selected: None }).id();
    create_button(
        Vec2::new(-2.0, 8.0),
        Vec2::new(0.8, 0.8),
        &mut commands,
        grid.clone(),
        Some((
            Container(SelectionType::Region),
            RadioButton { group: radio_group },
        )),
    );
    create_button(
        Vec2::new(-2.0, 7.0),
        Vec2::new(0.8, 0.8),
        &mut commands,
        grid.clone(),
        Some((
            Container(SelectionType::Line),
            RadioButton { group: radio_group },
        )),
    );
    create_button(
        Vec2::new(-2.0, 6.0),
        Vec2::new(0.8, 0.8),
        &mut commands,
        grid.clone(),
        Some((
            Container(SelectionType::Edges),
            RadioButton { group: radio_group },
        )),
    );
}

fn create_button(
    pos: Vec2,
    size: Vec2,
    commands: &mut Commands,
    grid: Entity,
    bundle: Option<impl Bundle>,
) -> Entity {
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
        make_fill(Color::WHITE),
        make_stroke(Color::BLACK, 0.02, false),
    ));
    if let Some(bundle) = bundle {
        button.insert(bundle);
    }
    let button = button.id();
    commands.entity(grid).add_child(button.clone());
    button
}

fn handle_button_state_change(
    mut q_button: Query<(&Button, &ButtonState, &mut Fill), Changed<ButtonState>>,
) {
    for (_, state, mut fill) in q_button.iter_mut() {
        let color = match state {
            ButtonState::Disabled => Color::Srgba(Srgba::rgb(0.7, 0.7, 0.7)),
            ButtonState::Normal => Color::Srgba(Srgba::rgb(1.0, 1.0, 1.0)),
            ButtonState::Hover => Color::Srgba(Srgba::rgb(0.95, 0.95, 0.95)),
            ButtonState::Pressed => Color::Srgba(Srgba::rgb(0.9, 0.9, 0.9)),
        };
        fill.color = color;
    }
}

fn handle_button_radio_change(
    mut q_button: Query<(&Button, &RadioButtonState, &mut Stroke), Changed<RadioButtonState>>,
) {
    for (_, state, mut stroke) in q_button.iter_mut() {
        let line_width = match state {
            RadioButtonState::Selected => 0.04,
            RadioButtonState::Unselected => 0.02,
        };
        stroke.options.line_width = line_width;
    }
}

fn handle_button_action_change(
    q_button: Query<(&Button, &Container<SelectionType>, &ButtonAction), Changed<ButtonAction>>,
    mut ev_set_selection_type: EventWriter<ChangeSelectionTypeEvent>,
) {
    for (_, container, state) in q_button.iter() {
        if *state == ButtonAction::Up {
            ev_set_selection_type.send(ChangeSelectionTypeEvent(container.0.clone()));
        }
    }
}

fn handle_radio_groups(
    q_button: Query<(Entity, &RadioButton, &ButtonAction), Changed<ButtonAction>>,
    mut q_radio_state: Query<&mut RadioButtonState>,
    mut q_radio_groups: Query<&mut RadioGroup>,
) {
    for (button_entity, button, state) in q_button.iter() {
        if *state == ButtonAction::Up {
            let mut group = q_radio_groups
                .get_mut(button.group)
                .expect("RadioGroup not found");
            if let Some(cur_selected) = group.selected {
                if cur_selected == button_entity {
                    continue;
                }
                let mut cur_radio_state = q_radio_state
                    .get_mut(cur_selected)
                    .expect("RadioButtonState not found");
                *cur_radio_state = RadioButtonState::Unselected;
            }
            group.selected = Some(button_entity);
            let mut radio_state = q_radio_state
                .get_mut(button_entity)
                .expect("RadioButtonState not found");
            *radio_state = RadioButtonState::Selected;
        }
    }
}

fn handle_button_state(
    mouse_world: Res<MouseWorldPos>,
    mut q_button: Query<(
        &Button,
        &Transform,
        &InheritedVisibility,
        &mut ButtonState,
        &mut ButtonAction,
    )>,
    q_grid: Query<(&Grid, &GlobalTransform)>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
) {
    let (_, grid_transform) = q_grid.single();
    let mouse_grid_space = grid_transform
        .affine()
        .inverse()
        .transform_point(mouse_world.0.extend(0.))
        .truncate();

    for (button, button_transform, button_visibility, mut button_state, mut button_action) in
        q_button.iter_mut()
    {
        if *button_state == ButtonState::Disabled {
            continue;
        }
        if !button_visibility.get() {
            continue;
        }
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

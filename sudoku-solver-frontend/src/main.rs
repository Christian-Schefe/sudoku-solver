use bevy::{prelude::*, window::PrimaryWindow};
use bevy_prototype_lyon::plugin::ShapePlugin;

mod grid;
mod region;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, ShapePlugin, grid::grid_plugin))
        .insert_resource(ClearColor(Color::srgb(0.3, 0.3, 0.4)))
        .insert_resource(MouseWorldPos(Vec2::ZERO))
        .add_systems(Startup, setup_main)
        .add_systems(Update, mouse_world_pos)
        .run();
}

#[derive(Component)]
pub struct MainCamera;

#[derive(Resource)]
pub struct MouseWorldPos(pub Vec2);

fn setup_main(mut commands: Commands) {
    let mut proj = OrthographicProjection::default_2d();
    proj.scaling_mode = bevy::render::camera::ScalingMode::AutoMin {
        min_width: 10.0,
        min_height: 10.0,
    };

    commands.spawn((MainCamera, Camera2d, Msaa::Sample4, proj));
}

fn mouse_world_pos(
    mut coords: ResMut<MouseWorldPos>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    let window = match q_window.get_single() {
        Ok(window) => window,
        Err(_) => return,
    };
    let (camera, camera_transform) = q_camera.single();

    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
        .map(|ray| ray.origin.truncate())
    {
        coords.0 = world_position;
    }
}

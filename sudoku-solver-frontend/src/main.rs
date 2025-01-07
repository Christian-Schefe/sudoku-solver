use bevy::prelude::*;
use bevy_prototype_lyon::plugin::ShapePlugin;

mod grid;
mod region;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, ShapePlugin))
        .insert_resource(ClearColor(Color::WHITE))
        .add_systems(Startup, (setup_main, grid::setup_grid))
        .run();
}

fn setup_main(mut commands: Commands) {
    let mut proj = OrthographicProjection::default_2d();
    proj.scaling_mode = bevy::render::camera::ScalingMode::AutoMin { min_width: 10.0, min_height: 10.0 };

    commands.spawn((Camera2d, Msaa::Sample4, proj));
}

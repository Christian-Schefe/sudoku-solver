use bevy::{prelude::*, window::PrimaryWindow};
use bevy_prototype_lyon::plugin::ShapePlugin;

mod grid;
mod region;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, ShapePlugin, grid::grid_plugin))
        .insert_resource(ClearColor(Color::srgb(1.0, 1.0, 1.0)))
        .insert_resource(MouseWorldPos(Vec2::ZERO))
        .add_systems(Startup, setup_main)
        .add_systems(PreUpdate, mouse_world_pos)
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

    commands.spawn((MainCamera, Camera2d, Msaa::Sample4, UiAntiAlias::On, proj));
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

fn stroke(color: Color, width: f32, round: bool) -> bevy_prototype_lyon::prelude::Stroke {
    let mut options = bevy_prototype_lyon::prelude::StrokeOptions::DEFAULT
        .with_line_width(width)
        .with_tolerance(0.001);
    if round {
        options = options
            .with_line_cap(bevy_prototype_lyon::prelude::LineCap::Round)
            .with_line_join(bevy_prototype_lyon::prelude::LineJoin::Round);
    }
    bevy_prototype_lyon::prelude::Stroke { color, options }
}

fn fill(color: Color) -> bevy_prototype_lyon::prelude::Fill {
    let options = bevy_prototype_lyon::prelude::FillOptions::DEFAULT.with_tolerance(0.001);
    bevy_prototype_lyon::prelude::Fill { color, options }
}

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn as_ivec2(&self) -> IVec2 {
        match self {
            Direction::Up => IVec2::new(0, 1),
            Direction::Down => IVec2::new(0, -1),
            Direction::Left => IVec2::new(-1, 0),
            Direction::Right => IVec2::new(1, 0),
        }
    }
    pub fn closest_from_vec2(vec: Vec2) -> Direction {
        if vec.x.abs() > vec.y.abs() {
            if vec.x >= 0.0 {
                Direction::Right
            } else {
                Direction::Left
            }
        } else {
            if vec.y >= 0.0 {
                Direction::Up
            } else {
                Direction::Down
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UnorderedPair(IVec2, IVec2);

impl UnorderedPair {
    pub fn new(a: IVec2, b: IVec2) -> Self {
        match UnorderedPair::cmp_vec(&a, &b) {
            std::cmp::Ordering::Greater => UnorderedPair(b, a),
            _ => UnorderedPair(a, b),
        }
    }
    fn cmp_vec(a: &IVec2, b: &IVec2) -> std::cmp::Ordering {
        a.y.cmp(&b.y).then(a.x.cmp(&b.x))
    }
}

impl PartialOrd for UnorderedPair {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for UnorderedPair {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match UnorderedPair::cmp_vec(&self.0, &other.0) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        UnorderedPair::cmp_vec(&self.1, &other.1)
    }
}

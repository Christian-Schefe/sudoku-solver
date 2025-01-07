use bevy::{
    color::palettes::css::{BLACK, LIGHT_SKY_BLUE},
    prelude::*,
};
use bevy_prototype_lyon::prelude::*;
use sudoku_solver::model::region::Region;
use tess::geom::euclid::Translation2D;

use crate::region::get_region_polygon;

#[derive(Component)]
struct Grid {
    cells: Vec<Vec<Entity>>,
    model: sudoku_solver::model::SudokuModel,
}

#[derive(Component)]
struct CellPos {
    pos: IVec2,
}

pub fn setup_grid(mut commands: Commands, asset_server: Res<AssetServer>) {
    let model = sudoku_solver::example::killer_test_model(None).build_model();
    let mut cells = Vec::new();
    let size = model.size;
    let center = Vec3::new(size.x as f32 / 2., size.y as f32 / 2., 0.) - Vec3::ONE * 0.5;

    let font = asset_server.load("FiraMono-Medium.ttf");

    for row in 0..size.y {
        let mut row_cells = Vec::new();
        for col in 0..size.x {
            let pos = IVec2::new(col, row);
            let vec = Vec3::new(row as f32, col as f32, 0f32) - center;
            let cell = commands
                .spawn((
                    CellPos { pos },
                    Text2d("1".to_string()),
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
    commands.spawn(Grid { cells, model });
    setup_grid_lines(&mut commands, size);

    commands.spawn((
        ShapeBundle {
            path: get_region_polygon(
                &Region {
                    cells: [
                        IVec2::new(1, 1),
                        IVec2::new(1, 2),
                        IVec2::new(2, 2),
                    ]
                    .into_iter()
                    .collect(),
                },
                0.1,
            ),
            transform: Transform::from_translation(Vec3::new(0., 0., -1.)),
            ..Default::default()
        },
        Stroke::new(BLACK, 0.02),
        Fill::color(Color::NONE),
    ));
}

fn setup_grid_lines(commands: &mut Commands, size: IVec2) {
    for row in 0..=size.y {
        setup_line(commands, size, row, true);
    }
    for col in 0..=size.x {
        setup_line(commands, size, col, false);
    }
}

fn setup_line(commands: &mut Commands, size: IVec2, index: i32, is_row: bool) {
    let mut path_builder = PathBuilder::new();
    path_builder.move_to(Vec2::new(0., 0.));
    path_builder.line_to(if is_row {
        Vec2::new(size.x as f32, 0.)
    } else {
        Vec2::new(0., size.y as f32)
    });
    let path = path_builder.build();
    let center = Vec3::new(size.x as f32 / 2., size.y as f32 / 2., 0.) - Vec3::ONE * 0.5;
    let pos = if is_row {
        Vec3::new(-0.5, index as f32 - 0.5, -1.)
    } else {
        Vec3::new(index as f32 - 0.5, -0.5, -1.)
    } - center;
    commands.spawn((
        ShapeBundle {
            path,
            transform: Transform::from_translation(pos),
            ..Default::default()
        },
        Stroke::new(BLACK, 0.02),
        Fill::color(Color::NONE),
    ));
}

fn get_selector_shape(width: f32, neighbours: &[bool; 8]) -> Path {
    fn get_corner_shape(width: f32) -> Path {
        let shape = shapes::RegularPolygon {
            sides: 4,
            feature: shapes::RegularPolygonFeature::SideLength(width),
            ..shapes::RegularPolygon::default()
        };
        GeometryBuilder::build_as(&shape)
    }
    fn get_edge_shape(width: f32, is_horizontal: bool) -> Path {
        let half = width / 2.;
        let opposite = (1.0 - width * 2.0) / 2.;
        let shape = shapes::Polygon {
            points: if is_horizontal {
                vec![
                    Vec2::new(-opposite, -half),
                    Vec2::new(opposite, -half),
                    Vec2::new(opposite, half),
                    Vec2::new(-opposite, half),
                ]
            } else {
                vec![
                    Vec2::new(-half, -opposite),
                    Vec2::new(half, -opposite),
                    Vec2::new(half, opposite),
                    Vec2::new(-half, opposite),
                ]
            },
            closed: true,
        };
        GeometryBuilder::build_as(&shape)
    }
    let corner = get_corner_shape(width);
    let hori_edge = get_edge_shape(width, true);
    let vert_edge = get_edge_shape(width, false);

    let maybe_corner = |hori: bool, vert: bool, diagonal: bool, point: Vec2| {
        if vert && hori && diagonal {
            return None;
        }
        let translation = Translation2D::new(point.x, point.y);
        Some(Path(corner.clone().0.transformed(&translation)))
    };
    let maybe_hori_edge = |top: bool, point: Vec2| {
        if top {
            return None;
        }
        let translation = Translation2D::new(point.x, point.y);
        Some(Path(hori_edge.clone().0.transformed(&translation)))
    };
    let maybe_vert_edge = |left: bool, point: Vec2| {
        if left {
            return None;
        }
        let translation = Translation2D::new(point.x, point.y);
        Some(Path(vert_edge.clone().0.transformed(&translation)))
    };

    let mut geometry_builder = GeometryBuilder::new();
    let corner_indices = [0, 2, 4, 6].map(|i| [i, (i + 1) % 8, (i + 2) % 8]);
    let index_offsets = [
        Vec2::new(-1., 1.) + Vec2::new(width, -width),
        Vec2::new(0., 1.) + Vec2::new(0., -width),
        Vec2::new(1., 1.) + Vec2::new(-width, -width),
        Vec2::new(1., 0.) + Vec2::new(-width, 0.),
        Vec2::new(1., -1.) + Vec2::new(-width, width),
        Vec2::new(0., -1.) + Vec2::new(0., width),
        Vec2::new(-1., -1.) + Vec2::new(width, width),
        Vec2::new(-1., 0.) + Vec2::new(width, 0.),
    ]
    .map(|x| x / 2.);

    let hori_edge_indices = [1, 5];
    let vert_edge_indices = [3, 7];
    for [i, j, k] in corner_indices {
        let corner = maybe_corner(
            neighbours[i],
            neighbours[j],
            neighbours[k],
            index_offsets[i],
        );
        if let Some(corner) = corner {
            geometry_builder = geometry_builder.add(&corner);
        }
    }
    for i in hori_edge_indices {
        let edge = maybe_hori_edge(neighbours[i], index_offsets[i]);
        if let Some(edge) = edge {
            geometry_builder = geometry_builder.add(&edge);
        }
    }
    for i in vert_edge_indices {
        let edge = maybe_vert_edge(neighbours[i], index_offsets[i]);
        if let Some(edge) = edge {
            geometry_builder = geometry_builder.add(&edge);
        }
    }
    geometry_builder.build()
}

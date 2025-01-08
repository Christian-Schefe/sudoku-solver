use bevy::{
    color::palettes::css::{BLACK, LIGHT_SKY_BLUE},
    prelude::*,
};
use bevy_prototype_lyon::prelude::*;
use sudoku_solver::model::region::Region;

use crate::{region::get_region_polygon, MouseWorldPos};

#[derive(Component)]
pub struct Grid {
    cells: Vec<Vec<Entity>>,
    model: sudoku_solver::model::SudokuModel,
    offset: Vec2,
}

#[derive(Component)]
pub struct CellPos {
    pos: IVec2,
}

pub fn setup_grid(mut commands: Commands, asset_server: Res<AssetServer>) {
    let model = sudoku_solver::example::killer_test_model(None).build_model();
    let mut cells = Vec::new();
    let size = model.size;
    let center = Vec3::new(size.x as f32 / 2., size.y as f32 / 2., 0.) - Vec3::ONE * 0.5;

    let font = asset_server.load("FiraMono-Medium.ttf");

    for y in 0..size.y {
        let mut row_cells = Vec::new();
        for x in 0..size.x {
            let pos = IVec2::new(x, y);
            let vec = Vec3::new(x as f32, y as f32, 0f32) - center;
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
    commands.spawn(Grid {
        cells,
        model,
        offset: center.truncate(),
    });
    setup_grid_lines(&mut commands, size);

    commands.spawn((
        ShapeBundle {
            path: get_region_polygon(
                &Region {
                    cells: [
                        IVec2::new(0, 0),
                        IVec2::new(0, 1),
                        IVec2::new(1, 0),
                        IVec2::new(2, 0),
                        IVec2::new(2, 1),
                        IVec2::new(2, 2),
                        IVec2::new(1, 2),
                        IVec2::new(0, 2),
                        IVec2::new(4, 4),
                        IVec2::new(5, 5),
                        IVec2::new(6, 6),
                    ]
                    .into_iter()
                    .collect(),
                },
                0.1,
            ),
            transform: Transform::from_translation(Vec3::new(0., 0., -2.) - center),
            ..Default::default()
        },
        Stroke {
            color: Color::Srgba(LIGHT_SKY_BLUE),
            options: StrokeOptions::default().with_line_width(0.2),
        },
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

pub fn select_handler(
    mouse_world: Res<MouseWorldPos>,
    q_grid: Query<&Grid>,
    mut q_cells: Query<(&CellPos, &mut Text2d)>,
) {
    for grid in q_grid.iter() {
        let cell_pos = (mouse_world.0 + grid.offset).round().as_ivec2();
        println!("Cell pos: {:?}", cell_pos);
        if cell_pos.x >= 0
            && cell_pos.x < grid.model.size.x
            && cell_pos.y >= 0
            && cell_pos.y < grid.model.size.y
        {
            let cell = &grid.cells[cell_pos.y as usize][cell_pos.x as usize];
            let mut text = q_cells.get_mut(*cell).unwrap().1;
            text.0 = "2".to_string();
        }
    }
}

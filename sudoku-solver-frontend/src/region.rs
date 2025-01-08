use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use sudoku_solver::model::region::Region;

pub fn get_region_polygon(region: &Region, inset: f32) -> Path {
    let mut shape = GeometryBuilder::new();
    for line_segments in get_region_line_segments(region, inset) {
        let mut path = PathBuilder::new();
        let mut prev = None;
        line_segments.iter().for_each(|(start, end)| {
            if let Some(prev_end) = prev {
                if prev_end != *start {
                    path.line_to(*start);
                }
            } else {
                path.move_to(*start);
            }
            path.line_to(*end);
            prev = Some(*end);
        });
        path.close();
        shape = shape.add(&path.build());
    }
    shape.build()
}

fn get_region_line_segments(region: &Region, inset: f32) -> Vec<Vec<(Vec2, Vec2)>> {
    let (lines, edges) = get_region_boundaries(region);
    lines
        .into_iter()
        .map(|boundary| {
            let line_segments = boundary.iter().map(|(start, end)| {
                let normal = edges[&(*start, *end)];
                let dir = *end - *start;
                let start = start.as_vec2() + dir.as_vec2() * inset - normal.as_vec2() * inset;
                let end = end.as_vec2() - dir.as_vec2() * inset - normal.as_vec2() * inset;
                let offset = Vec2::new(-0.5, -0.5);
                (start + offset, end + offset)
            });
            line_segments.collect()
        })
        .collect()
}

fn get_region_boundaries(
    region: &Region,
) -> (Vec<Vec<(IVec2, IVec2)>>, HashMap<(IVec2, IVec2), IVec2>) {
    let edges = get_region_edges(region);
    let mut all_paths = Vec::new();
    let mut visited = HashSet::new();

    for (edge, _) in &edges {
        if visited.contains(edge) {
            continue;
        }
        let mut path = vec![];
        let mut current = edge.clone();
        while !visited.contains(&current) {
            println!("({}, {})", current.0, current.1);
            visited.insert(current.clone());
            visited.insert((current.1, current.0));
            let cur_dir = current.1 - current.0;
            for dir in [clockwise(cur_dir), cur_dir, anticlockwise(cur_dir)] {
                let next = (current.1, current.1 + dir);
                if edges.contains_key(&next) {
                    path.push(next.clone());
                    current = next.clone();
                    break;
                }
            }
        }
        println!("{:?}", path);
        all_paths.push(path);
    }

    (all_paths, edges)
}

fn clockwise(vec: IVec2) -> IVec2 {
    IVec2::new(vec.y, -vec.x)
}

fn anticlockwise(vec: IVec2) -> IVec2 {
    IVec2::new(-vec.y, vec.x)
}

fn get_region_edges(region: &Region) -> HashMap<(IVec2, IVec2), IVec2> {
    let mut boundary = HashMap::new();

    for cell in &region.cells {
        let cell = *cell;
        let neighbors = [
            (cell + IVec2::new(0, 1), IVec2::new(0, 1), IVec2::new(1, 0)),
            (cell + IVec2::new(1, 0), IVec2::new(1, 0), IVec2::new(0, 1)),
            (cell + IVec2::new(0, -1), IVec2::new(0, 0), IVec2::new(1, 0)),
            (cell + IVec2::new(-1, 0), IVec2::new(0, 0), IVec2::new(0, 1)),
        ];
        for (neighbor, start_offset, dir) in neighbors.iter() {
            if !region.cells.contains(neighbor) {
                let normal = *neighbor - cell;
                let start = cell + start_offset;
                let end = start + dir;
                boundary.insert((start, end), normal);
                boundary.insert((end, start), normal);
            }
        }
    }
    println!("{:?}", boundary);
    boundary
}

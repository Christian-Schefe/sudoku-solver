use std::collections::HashSet;

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use sudoku_solver::model::region::Region;

pub fn get_region_polygon(region: &Region, inset: f32) -> Path {
    let mut shape = PathBuilder::new();
    for line_segments in get_region_line_segments(region, inset) {
        let mut prev = None;
        line_segments.iter().for_each(|(start, end)| {
            if let Some(prev_end) = prev {
                if prev_end != *start {
                    shape.line_to(*start);
                }
            } else {
                shape.move_to(*start);
            }
            shape.line_to(*end);
            prev = Some(*end);
        });
    }
    shape.build()
}

fn get_region_line_segments(region: &Region, inset: f32) -> Vec<Vec<(Vec2, Vec2)>> {
    get_region_boundaries(region)
        .into_iter()
        .map(|boundary| {
            let outset = 0.5 - inset;
            let line_segments = boundary.iter().map(|(pos, normal)| {
                let dir = IVec2::new(-normal.y, normal.x);
                let start = pos.as_vec2() + dir.as_vec2() * outset + normal.as_vec2() * outset;
                let end = pos.as_vec2() - dir.as_vec2() * outset + normal.as_vec2() * outset;
                (start, end)
            });
            line_segments.collect()
        })
        .collect()
}

fn get_region_boundaries(region: &Region) -> Vec<Vec<(IVec2, IVec2)>> {
    let edges = get_region_edges(region);
    let mut edge_set = edges.iter().cloned().collect::<HashSet<_>>();
    let mut all_paths = Vec::new();

    fn do_edges_connect(pos1: IVec2, normal1: IVec2, pos2: IVec2, normal2: IVec2) -> bool {
        let lrot = IVec2::new(-normal1.y, normal1.x);
        let rrot = IVec2::new(normal1.y, -normal1.x);
        (pos1 == pos2 && normal2 == rrot)
                || (pos1 + normal1 == pos2 && normal2 == rrot)
                || (pos1 == pos2 + rrot && normal2 == normal1)
    }

    while !edge_set.is_empty() {
        let (start, start_dir) = edge_set.iter().next().copied().unwrap();
        edge_set.remove(&(start, start_dir));
        let mut path = vec![(start, start_dir)];
        let mut cur = start;
        let mut cur_normal = start_dir;
        while let Some((next, next_normal)) = edge_set
            .iter()
            .find(|(pos, normal)| do_edges_connect(cur, cur_normal, *pos, *normal))
            .copied()
        {
            edge_set.remove(&(next, next_normal));
            path.push((next, next_normal));
            cur = next;
            cur_normal = next_normal;
        }
        path.push((start, start_dir));
        println!("{:?}", path);
        all_paths.push(path);
    }
    println!("{:?}", all_paths);
    all_paths
}

fn get_region_edges(region: &Region) -> Vec<(IVec2, IVec2)> {
    let mut boundary = Vec::new();

    for cell in &region.cells {
        let cell = *cell;
        let neighbors = [
            cell + IVec2::new(1, 0),
            cell + IVec2::new(0, 1),
            cell + IVec2::new(-1, 0),
            cell + IVec2::new(0, -1),
        ];
        for neighbor in neighbors.iter() {
            if !region.cells.contains(neighbor) {
                boundary.push((cell, *neighbor - cell));
            }
        }
    }
    println!("{:?}", boundary);
    boundary
}

fn get_region_boundary_for_start(region: &Region, start_point: IVec2) -> Vec<(IVec2, IVec2)> {
    let mut dir = IVec2::new(-1, 0);
    let mut cur = start_point.clone();

    let mut boundary = Vec::new();

    loop {
        let mut next = cur + dir;
        while !region.cells.contains(&next) {
            boundary.push((cur, dir));
            dir = IVec2::new(dir.y, -dir.x);
            next = cur + dir;
        }
        cur = next;
        dir = IVec2::new(-dir.y, dir.x);
        if cur == start_point {
            break;
        }
    }

    boundary
}

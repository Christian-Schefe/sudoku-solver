use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

use crate::UnorderedPair;

pub fn get_line_polygon(line: &Vec<IVec2>) -> Path {
    let mut path = PathBuilder::new();
    if line.is_empty() {
        return path.build();
    }
    path.move_to(line[0].as_vec2());
    for cell in line.iter() {
        path.line_to(cell.as_vec2());
    }
    path.build()
}

pub fn get_edges_polygon(edges: &HashSet<UnorderedPair>) -> Path {
    let mut shape = GeometryBuilder::new();
    for UnorderedPair(start, end) in edges.iter() {
        let dir = end - start;
        let normal = IVec2::new(dir.y, -dir.x);
        let start = start.as_vec2() + dir.as_vec2() * 0.5 + normal.as_vec2() * 0.5;
        let end = end.as_vec2() - dir.as_vec2() * 0.5 - normal.as_vec2() * 0.5;
        let mut path = PathBuilder::new();
        path.move_to(start);
        path.line_to(end);
        shape = shape.add(&path.build());
    }
    shape.build()
}

pub fn get_region_polygon(region: &HashSet<IVec2>, inset: f32) -> Path {
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

fn get_region_line_segments(region: &HashSet<IVec2>, inset: f32) -> Vec<Vec<(Vec2, Vec2)>> {
    let (lines, edges) = get_region_boundaries(region);
    lines
        .into_iter()
        .map(|boundary| {
            let len = boundary.len();
            let mut line_segments = Vec::new();
            for i in 0..len {
                let cur = &boundary[i];
                let normal = edges[cur];
                let dir = cur.1 - cur.0;
                let offset = Vec2::new(-0.5, -0.5);
                let start =
                    cur.0.as_vec2() + dir.as_vec2() * inset - normal.as_vec2() * inset + offset;
                let end =
                    cur.1.as_vec2() - dir.as_vec2() * inset - normal.as_vec2() * inset + offset;
                line_segments.push((start, end));
            }

            for i in 0..len {
                let prev = &boundary[(i + len - 1) % len];
                let next = &boundary[(i + 1) % len];
                let cur = &boundary[i];
                let new_start = if let Some(prev_is_hori) = can_grid_intersect(prev, cur) {
                    grid_intersection(
                        &line_segments[(i + len - 1) % len],
                        &line_segments[i],
                        prev_is_hori,
                    )
                } else {
                    line_segments[i].0
                };
                let new_end = if let Some(next_is_hori) = can_grid_intersect(cur, next) {
                    grid_intersection(
                        &line_segments[i],
                        &line_segments[(i + 1) % len],
                        next_is_hori,
                    )
                } else {
                    line_segments[i].1
                };
                line_segments[i] = (new_start, new_end);
            }

            line_segments
        })
        .collect()
}

fn get_region_boundaries(
    region: &HashSet<IVec2>,
) -> (Vec<Vec<(IVec2, IVec2)>>, HashMap<(IVec2, IVec2), IVec2>) {
    let edges = get_region_edges(region);
    let all_edges = edges
        .iter()
        .flat_map(|((start, end), normal)| [((*start, *end), *normal), ((*end, *start), *normal)])
        .collect::<HashMap<_, _>>();
    let mut all_paths = Vec::new();
    let mut visited = HashSet::new();

    for (edge, _) in &edges {
        if visited.contains(edge) {
            continue;
        }
        let mut path = vec![];
        let mut current = edge.clone();
        while !visited.contains(&current) {
            visited.insert(current.clone());
            visited.insert((current.1, current.0));
            let cur_dir = current.1 - current.0;
            for dir in [clockwise(cur_dir), cur_dir, anticlockwise(cur_dir)] {
                let next = (current.1, current.1 + dir);
                if all_edges.contains_key(&next) {
                    path.push(next.clone());
                    current = next.clone();
                    break;
                }
            }
        }
        all_paths.push(path);
    }

    (all_paths, all_edges)
}

fn clockwise(vec: IVec2) -> IVec2 {
    IVec2::new(vec.y, -vec.x)
}

fn anticlockwise(vec: IVec2) -> IVec2 {
    IVec2::new(-vec.y, vec.x)
}

fn can_grid_intersect(line1: &(IVec2, IVec2), line2: &(IVec2, IVec2)) -> Option<bool> {
    let d1 = line1.1 - line1.0;
    let d2 = line2.1 - line2.0;
    let is_line1_hori = d1.x != 0;
    let is_line2_hori = d2.x != 0;
    if is_line1_hori == is_line2_hori {
        None
    } else {
        Some(is_line1_hori)
    }
}

fn grid_intersection(line1: &(Vec2, Vec2), line2: &(Vec2, Vec2), first_is_hori: bool) -> Vec2 {
    if first_is_hori {
        Vec2::new(line2.0.x, line1.0.y)
    } else {
        Vec2::new(line1.0.x, line2.0.y)
    }
}

fn get_region_edges(region: &HashSet<IVec2>) -> HashMap<(IVec2, IVec2), IVec2> {
    let mut boundary = HashMap::new();

    for cell in region.iter() {
        let cell = *cell;
        let neighbors = [
            (cell + IVec2::new(0, 1), IVec2::new(0, 1), IVec2::new(1, 0)),
            (cell + IVec2::new(1, 0), IVec2::new(1, 1), IVec2::new(0, -1)),
            (
                cell + IVec2::new(0, -1),
                IVec2::new(1, 0),
                IVec2::new(-1, 0),
            ),
            (cell + IVec2::new(-1, 0), IVec2::new(0, 0), IVec2::new(0, 1)),
        ];
        for (neighbor, start_offset, dir) in neighbors.iter() {
            if !region.contains(neighbor) {
                let normal = *neighbor - cell;
                let start = cell + start_offset;
                let end = start + dir;
                boundary.insert((start, end), normal);
            }
        }
    }
    boundary
}

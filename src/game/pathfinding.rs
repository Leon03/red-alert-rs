use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

/// A* pathfinding on a tile grid.
/// Coordinates are `(x, y)` where both are `u32`.
/// Movement is 4-directional (no diagonals).
//
// Priority-queue node. We use a max-heap (Rust's BinaryHeap default),
// so we invert `Ord` to get a min-heap on `f_cost`.
#[derive(Clone, Copy, Debug)]
struct Node {
    f_cost: f32,
    x: u32,
    y: u32,
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.f_cost == other.f_cost
    }
}

impl Eq for Node {}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Node {
    // Reversed so that BinaryHeap (max-heap) behaves as a min-heap on f_cost.
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .f_cost
            .partial_cmp(&self.f_cost)
            .unwrap_or(Ordering::Equal)
    }
}

const DIRECTIONS: [(i32, i32); 4] = [(0, -1), (0, 1), (-1, 0), (1, 0)];

/// Compute an A* path from `start` to `end` on a grid of size `(width, height)`,
/// avoiding the tiles in `blocked`.
///
/// Returns a vector of tile coordinates from `start` (inclusive) to `end`
/// (inclusive). If no path exists, returns an empty vector.
/// If `start == end`, returns a single-element path.
pub fn find_path(
    start: (u32, u32),
    end: (u32, u32),
    blocked: &HashSet<(u32, u32)>,
    width: u32,
    height: u32,
) -> Vec<(u32, u32)> {
    // Edge case: start == end
    if start == end {
        return vec![start];
    }

    // If start or end is out of bounds or blocked, no path
    if !in_bounds(start, width, height) || !in_bounds(end, width, height) {
        return Vec::new();
    }
    if blocked.contains(&start) || blocked.contains(&end) {
        return Vec::new();
    }

    let mut open: BinaryHeap<Node> = BinaryHeap::new();
    let mut g_score: HashMap<(u32, u32), f32> = HashMap::new();
    let mut came_from: HashMap<(u32, u32), (u32, u32)> = HashMap::new();
    let mut closed: HashSet<(u32, u32)> = HashSet::new();

    g_score.insert(start, 0.0_f32);
    open.push(Node {
        f_cost: heuristic(start, end),
        x: start.0,
        y: start.1,
    });

    while let Some(current) = open.pop() {
        let cur_pos = (current.x, current.y);

        // If we've already processed this node with a better score, skip
        if closed.contains(&cur_pos) {
            continue;
        }

        if cur_pos == end {
            // Reconstruct path
            return reconstruct(came_from, start, end);
        }

        closed.insert(cur_pos);

        for (dx, dy) in DIRECTIONS.iter() {
            let nx = cur_pos.0 as i32 + dx;
            let ny = cur_pos.1 as i32 + dy;

            if nx < 0 || ny < 0 {
                continue;
            }
            let next = (nx as u32, ny as u32);
            if !in_bounds(next, width, height) {
                continue;
            }
            if blocked.contains(&next) {
                continue;
            }
            if closed.contains(&next) {
                continue;
            }

            let tentative_g = g_score[&cur_pos] + 1.0_f32;

            if tentative_g < *g_score.get(&next).unwrap_or(&f32::MAX) {
                came_from.insert(next, cur_pos);
                g_score.insert(next, tentative_g);
                let f = tentative_g + heuristic(next, end);
                open.push(Node {
                    f_cost: f,
                    x: next.0,
                    y: next.1,
                });
            }
        }
    }

    // No path found
    Vec::new()
}

fn in_bounds(pos: (u32, u32), width: u32, height: u32) -> bool {
    pos.0 < width && pos.1 < height
}

fn heuristic(a: (u32, u32), b: (u32, u32)) -> f32 {
    // Manhattan distance (admissible for 4-directional movement)
    (a.0 as i32 - b.0 as i32).unsigned_abs() as f32
        + (a.1 as i32 - b.1 as i32).unsigned_abs() as f32
}

fn reconstruct(
    came_from: HashMap<(u32, u32), (u32, u32)>,
    start: (u32, u32),
    end: (u32, u32),
) -> Vec<(u32, u32)> {
    let mut path = Vec::new();
    let mut current = end;
    path.push(current);
    while current != start {
        current = came_from[&current];
        path.push(current);
    }
    path.reverse();
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn straight_line_no_obstacles() {
        let blocked = HashSet::new();
        let path = find_path((0, 0), (5, 0), &blocked, 50, 50);
        assert!(!path.is_empty());
        assert_eq!(path[0], (0, 0));
        assert_eq!(*path.last().unwrap(), (5, 0));
        // Each step must be adjacent (Manhattan distance 1)
        for window in path.windows(2) {
            let d = (window[0].0 as i32 - window[1].0 as i32).unsigned_abs()
                + (window[0].1 as i32 - window[1].1 as i32).unsigned_abs();
            assert_eq!(d, 1);
        }
    }

    #[test]
    fn path_around_obstacle() {
        let mut blocked = HashSet::new();
        // Block the entire row y=2 from x=0..=5, except we go around
        for x in 0..=6_u32 {
            blocked.insert((x, 2));
        }
        let path = find_path((0, 0), (0, 4), &blocked, 50, 50);
        assert!(!path.is_empty(), "Path should exist around the obstacle");
        assert_eq!(path[0], (0, 0));
        assert_eq!(*path.last().unwrap(), (0, 4));
        // No tile in the path should be blocked
        for tile in &path {
            assert!(
                !blocked.contains(tile),
                "Path goes through blocked tile {tile:?}"
            );
        }
    }

    #[test]
    fn no_path_when_fully_blocked() {
        let mut blocked = HashSet::new();
        // Block all tiles around start (0,0) — all 4 neighbors
        blocked.insert((1, 0));
        blocked.insert((0, 1));
        // (−1,0) and (0,−1) are out of bounds already
        let path = find_path((0, 0), (5, 5), &blocked, 50, 50);
        assert!(path.is_empty(), "Should have no path when surrounded");
    }

    #[test]
    fn start_equals_end() {
        let blocked = HashSet::new();
        let path = find_path((3, 3), (3, 3), &blocked, 50, 50);
        assert_eq!(path.len(), 1);
        assert_eq!(path[0], (3, 3));
    }
}

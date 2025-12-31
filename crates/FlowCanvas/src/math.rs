use glam::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Rect {
    pub min: Vec2,
    pub max: Vec2,
}

impl Rect {
    pub fn new(pos: Vec2, size: Vec2) -> Self {
        Self {
            min: pos,
            max: pos + size,
        }
    }

    pub fn contains(&self, p: Vec2) -> bool {
        p.x >= self.min.x && p.x <= self.max.x && p.y >= self.min.y && p.y <= self.max.y
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
    }

    pub fn expand(&self, amount: f32) -> Self {
        Self {
            min: self.min - Vec2::splat(amount),
            max: self.max + Vec2::splat(amount),
        }
    }
}

/// Calculates the two control points for a cubic Bezier curve connecting `start` to `end`.
///
/// This assumes a horizontal flow (left-to-right).
pub fn calculate_bezier_points(start: Vec2, end: Vec2) -> (Vec2, Vec2) {
    let dist = start.distance(end);
    let control_dist = (dist * 0.5).min(150.0);
    let cp1 = start + Vec2::new(control_dist, 0.0);
    let cp2 = end - Vec2::new(control_dist, 0.0);
    (cp1, cp2)
}

/// Calculates a smart orthogonal path avoiding obstacles.
pub fn calculate_smart_orthogonal(
    start: Vec2,
    end: Vec2,
    obstacles: &[Rect],
    buffer: f32,
) -> Vec<Vec2> {
    let outset = 20.0;

    // Start/End points for the pathfinder (after outsets)
    let p_start_real = start;
    let p_start = start + Vec2::new(outset, 0.0);
    let p_end_real = end;
    let p_end = end - Vec2::new(outset, 0.0);

    // If start and end are very close or overlap in a weird way, return simple path
    if p_start.distance(p_end) < 1.0 {
        return vec![p_start_real, p_end_real];
    }

    // 1. Generate interesting coordinates (Sparse Grid)
    let mut xs = vec![p_start.x, p_end.x];
    let mut ys = vec![p_start.y, p_end.y];

    // Add "global bypass" lanes around the entire graph
    let mut min_pt = Vec2::new(f32::INFINITY, f32::INFINITY);
    let mut max_pt = Vec2::new(f32::NEG_INFINITY, f32::NEG_INFINITY);
    min_pt = min_pt.min(p_start).min(p_end);
    max_pt = max_pt.max(p_start).max(p_end);

    for obs in obstacles {
        min_pt = min_pt.min(obs.min);
        max_pt = max_pt.max(obs.max);
    }

    xs.push(min_pt.x - 100.0);
    xs.push(max_pt.x + 100.0);
    ys.push(min_pt.y - 100.0);
    ys.push(max_pt.y + 100.0);

    for obs in obstacles {
        let b = obs.expand(buffer);
        // Main buffer lines
        xs.push(b.min.x);
        xs.push(b.max.x);
        ys.push(b.min.y);
        ys.push(b.max.y);

        // Intermediate navigation lanes
        xs.push(b.min.x - 20.0);
        xs.push(b.max.x + 20.0);
        ys.push(b.min.y - 20.0);
        ys.push(b.max.y + 20.0);

        // Help with narrow gaps
        xs.push((obs.min.x + obs.max.x) * 0.5);
        ys.push((obs.min.y + obs.max.y) * 0.5);
    }

    // Sort and remove duplicates
    use std::cmp::Ordering;
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    xs.dedup_by(|a, b| (*a - *b).abs() < 1.0);
    ys.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    ys.dedup_by(|a, b| (*a - *b).abs() < 1.0);

    // 2. A* Search
    use std::collections::{BinaryHeap, HashMap};

    #[derive(Copy, Clone, PartialEq)]
    struct Node {
        x_idx: usize,
        y_idx: usize,
        dir: i32, // 0: None, 1: H, 2: V
        g_score: f32,
        f_score: f32,
    }

    impl Eq for Node {}
    impl Ord for Node {
        fn cmp(&self, other: &Self) -> Ordering {
            other
                .f_score
                .partial_cmp(&self.f_score)
                .unwrap_or(Ordering::Equal)
        }
    }
    impl PartialOrd for Node {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    let start_x_idx = xs
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            (**a - p_start.x)
                .abs()
                .partial_cmp(&(**b - p_start.x).abs())
                .unwrap()
        })
        .map(|(i, _)| i)
        .unwrap();
    let start_y_idx = ys
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            (**a - p_start.y)
                .abs()
                .partial_cmp(&(**b - p_start.y).abs())
                .unwrap()
        })
        .map(|(i, _)| i)
        .unwrap();
    let end_x_idx = xs
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            (**a - p_end.x)
                .abs()
                .partial_cmp(&(**b - p_end.x).abs())
                .unwrap()
        })
        .map(|(i, _)| i)
        .unwrap();
    let end_y_idx = ys
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            (**a - p_end.y)
                .abs()
                .partial_cmp(&(**b - p_end.y).abs())
                .unwrap()
        })
        .map(|(i, _)| i)
        .unwrap();

    let mut open_set = BinaryHeap::new();
    let mut g_score = HashMap::new();
    let mut came_from = HashMap::new();

    open_set.push(Node {
        x_idx: start_x_idx,
        y_idx: start_y_idx,
        dir: 0,
        g_score: 0.0,
        f_score: Vec2::new(xs[start_x_idx], ys[start_y_idx])
            .distance(Vec2::new(xs[end_x_idx], ys[end_y_idx])),
    });
    g_score.insert((start_x_idx, start_y_idx), 0.0);

    let mut found = false;

    while let Some(current) = open_set.pop() {
        if current.x_idx == end_x_idx && current.y_idx == end_y_idx {
            found = true;
            break;
        }

        let neighbors = [
            (current.x_idx as i32 - 1, current.y_idx as i32, 1), // H
            (current.x_idx as i32 + 1, current.y_idx as i32, 1), // H
            (current.x_idx as i32, current.y_idx as i32 - 1, 2), // V
            (current.x_idx as i32, current.y_idx as i32 + 1, 2), // V
        ];

        for (nx, ny, n_dir) in neighbors {
            if nx < 0 || nx >= xs.len() as i32 || ny < 0 || ny >= ys.len() as i32 {
                continue;
            }
            let nx = nx as usize;
            let ny = ny as usize;
            let neighbor_pos = Vec2::new(xs[nx], ys[ny]);
            let current_pos = Vec2::new(xs[current.x_idx], ys[current.y_idx]);
            let mid_pos = (current_pos + neighbor_pos) * 0.5;

            // Robust collision check: check neighbor AND midpoint
            let mut blocked = false;
            for obs in obstacles {
                if obs.expand(-2.0).contains(neighbor_pos) || obs.expand(-2.0).contains(mid_pos) {
                    blocked = true;
                    break;
                }
            }
            if blocked {
                continue;
            }

            let dist = current_pos.distance(neighbor_pos);
            let mut step_cost = dist;

            // Proximity penalty: discourage being too close to nodes
            for obs in obstacles {
                if obs.expand(buffer).contains(neighbor_pos) {
                    step_cost += dist * 3.0; // Higher cost near nodes
                }
            }

            // Turn penalty (high cost to discourage unnecessary bends)
            if current.dir != 0 && current.dir != n_dir {
                step_cost += 150.0;
            }

            let tentative_g = g_score[&(current.x_idx, current.y_idx)] + step_cost;

            if tentative_g < *g_score.get(&(nx, ny)).unwrap_or(&f32::INFINITY) {
                came_from.insert((nx, ny), (current.x_idx, current.y_idx));
                g_score.insert((nx, ny), tentative_g);
                open_set.push(Node {
                    x_idx: nx,
                    y_idx: ny,
                    dir: n_dir,
                    g_score: tentative_g,
                    f_score: tentative_g
                        + neighbor_pos.distance(Vec2::new(xs[end_x_idx], ys[end_y_idx])),
                });
            }
        }
    }

    if found {
        let mut path = vec![p_end_real, p_end];
        let mut curr = (end_x_idx, end_y_idx);
        while let Some(&prev) = came_from.get(&curr) {
            path.push(Vec2::new(xs[curr.0], ys[curr.1]));
            curr = prev;
        }
        path.push(p_start);
        path.push(p_start_real);
        path.reverse();

        // Simplify path
        if path.len() > 2 {
            let mut simplified = vec![path[0]];
            for i in 1..path.len() - 1 {
                let p_prev = simplified.last().unwrap();
                let p_curr = path[i];
                let p_next = path[i + 1];
                let d1 = (p_curr - *p_prev).normalize_or_zero();
                let d2 = (p_next - p_curr).normalize_or_zero();
                if d1.dot(d2) < 0.999 {
                    simplified.push(p_curr);
                }
            }
            simplified.push(*path.last().unwrap());
            path = simplified;
        }
        path
    } else {
        let mid_x = (p_start.x + p_end.x) * 0.5;
        vec![
            p_start_real,
            p_start,
            Vec2::new(mid_x, p_start.y),
            Vec2::new(mid_x, p_end.y),
            p_end,
            p_end_real,
        ]
    }
}

/// Legacy orthogonal calculation (simple Z-shape)
pub fn calculate_orthogonal_points(start: Vec2, end: Vec2) -> Vec<Vec2> {
    let mid_x = (start.x + end.x) * 0.5;
    vec![
        start,
        Vec2::new(mid_x, start.y),
        Vec2::new(mid_x, end.y),
        end,
    ]
}

/// Simple two-point path for a linear connection.
pub fn calculate_linear_points(start: Vec2, end: Vec2) -> Vec<Vec2> {
    vec![start, end]
}

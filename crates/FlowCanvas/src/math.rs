//! # Math Helpers
//!
//! Utility functions for geometry and curves.

use glam::Vec2;

/// Calculates the two control points for a cubic Bezier curve connecting `start` to `end`.
///
/// This assumes a horizontal flow (left-to-right).
///
/// # Returns
/// `(cp1, cp2)`
pub fn calculate_bezier_points(start: Vec2, end: Vec2) -> (Vec2, Vec2) {
    let dist = start.distance(end);
    // Dynamic curvature based on distance, but capped to avoid wild loops for far nodes.
    // 0.5 is a standard smoothness factor.
    let control_dist = (dist * 0.5).min(150.0);

    // Horizontal S-curve: Output goes Right, Input comes from Left.
    let cp1 = start + Vec2::new(control_dist, 0.0);
    let cp2 = end - Vec2::new(control_dist, 0.0);

    (cp1, cp2)
}

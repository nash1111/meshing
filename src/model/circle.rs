use std::fmt;

use crate::model::point_2d::*;

/// A circle defined by a center [`Point2D`] and a radius.
///
/// Used to represent the circumcircle of a triangle during the
/// Bowyer-Watson algorithm.
#[derive(Debug, Clone, Copy)]
pub struct Circle {
    pub center: Point2D,
    pub radius: f64,
}

impl Circle {
    /// Returns `true` if the given point lies inside or on the boundary of this circle.
    ///
    /// Uses squared distance comparison to avoid computing a square root.
    pub fn point_in_circle(&self, point: &Point2D) -> bool {
        self.center.distance_squared(point) <= self.radius * self.radius
    }
}

impl fmt::Display for Circle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Circle(center={}, r={})", self.center, self.radius)
    }
}

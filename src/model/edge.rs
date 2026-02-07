use std::fmt;

use crate::model::point_2d::*;

/// A directed edge connecting two [`Point2D`] vertices.
///
/// Two edges are considered equal if they connect the same pair of points,
/// regardless of direction (i.e., edge equality is undirected).
#[derive(Debug, Clone, Copy)]
pub struct Edge {
    pub start: Point2D,
    pub end: Point2D,
}

impl Edge {
    /// Returns a new edge with the start and end points swapped.
    pub fn reverse(&self) -> Edge {
        Edge {
            start: self.end,
            end: self.start,
        }
    }
}

impl PartialEq for Edge {
    fn eq(&self, other: &Self) -> bool {
        (self.start == other.start && self.end == other.end)
            || (self.start == other.end && self.end == other.start)
    }
}

impl fmt::Display for Edge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Edge({} -> {})", self.start, self.end)
    }
}

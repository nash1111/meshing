use std::fmt;

/// A point in 2D space with an associated index.
///
/// The `index` field identifies the point within a point set, which is useful
/// for tracking which original points form each triangle in the Bowyer-Watson output.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2D {
    pub index: i64,
    pub x: f64,
    pub y: f64,
}

impl Point2D {
    /// Returns the squared Euclidean distance between this point and `other`.
    ///
    /// Avoids computing a square root, which is useful when only relative
    /// distances need to be compared.
    pub fn distance_squared(&self, other: &Point2D) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }

    /// Returns the Euclidean distance between this point and `p`.
    pub fn distance(&self, p: &Point2D) -> f64 {
        self.distance_squared(p).sqrt()
    }
}

impl fmt::Display for Point2D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Point2D(#{}, {}, {})", self.index, self.x, self.y)
    }
}

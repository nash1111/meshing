use crate::{Edge, Point2D, Triangle};

/// Creates a super-triangle that encloses all the given points.
///
/// The super-triangle is computed from the bounding box of the input points,
/// with a margin proportional to the spread of the point set. This ensures
/// that all points lie well within the super-triangle regardless of scale.
pub fn create_super_triangle(points: &[Point2D]) -> Triangle {
    let index = 0;
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;

    for point in points {
        if point.x < min_x {
            min_x = point.x;
        }
        if point.y < min_y {
            min_y = point.y;
        }
        if point.x > max_x {
            max_x = point.x;
        }
        if point.y > max_y {
            max_y = point.y;
        }
    }

    // Compute margin based on the spread of the point set so that the
    // super-triangle scales with the data rather than using a fixed value.
    let dx = max_x - min_x;
    let dy = max_y - min_y;
    let spread = if dx > dy { dx } else { dy };
    let margin = (spread + 1.0) * 10.0;

    let a = Point2D {
        index,
        x: min_x - margin,
        y: min_y - margin,
    };
    let b = Point2D {
        index,
        x: max_x + margin,
        y: min_y - margin,
    };
    let c = Point2D {
        index,
        x: (min_x + max_x) / 2.0,
        y: max_y + margin,
    };

    Triangle { a, b, c }
}

/// Returns `true` if the given edge is shared by any of the provided triangles.
///
/// An edge is considered shared if it matches any edge of any triangle,
/// regardless of direction.
pub fn edge_is_shared_by_triangles(edge: &Edge, triangles: &[Triangle]) -> bool {
    for triangle in triangles {
        let edges_of_triangle = triangle.edges();
        for edge_of_triangle in edges_of_triangle {
            if edge_of_triangle == *edge {
                return true;
            }
            if edge_of_triangle.reverse() == *edge {
                return true;
            }
        }
    }
    false
}

/// Creates a new triangle from an edge and a point.
///
/// The new triangle has vertices at the edge's start, the edge's end, and
/// the given point. Used during the re-triangulation step of Bowyer-Watson.
pub fn retriangulate(edge: &Edge, point: &Point2D) -> Triangle {
    Triangle {
        a: edge.start,
        b: edge.end,
        c: *point,
    }
}

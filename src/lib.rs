//! A Delaunay triangulation library using the Bowyer-Watson algorithm.
//!
//! # Example
//!
//! ```
//! use meshing::{bowyer_watson, Point2D};
//!
//! let points = vec![
//!     Point2D { index: 0, x: 0.0, y: 0.0 },
//!     Point2D { index: 1, x: 1.0, y: 0.0 },
//!     Point2D { index: 2, x: 0.0, y: 1.0 },
//!     Point2D { index: 3, x: 1.0, y: 1.0 },
//! ];
//! let triangles = bowyer_watson(points);
//! assert_eq!(triangles.len(), 2);
//! ```

use geometry::{create_super_triangle, edge_is_shared_by_triangles, retriangulate};
pub use model::{Edge, Point2D, Triangle};
use triangle_utils::remove_triangles_with_vertices_from_super_triangle;

mod geometry;
mod model;
mod triangle_utils;

/// Computes the Delaunay triangulation of a set of 2D points using the
/// Bowyer-Watson incremental insertion algorithm.
///
/// Returns a list of [`Triangle`]s forming the Delaunay triangulation.
///
/// # Panics
///
/// Panics if `points` is empty.
pub fn bowyer_watson(points: Vec<Point2D>) -> Vec<Triangle> {
    let mut triangulation: Vec<Triangle> = Vec::new();

    // Step 1: Create a super-triangle large enough to contain all input points.
    let super_triangle = create_super_triangle(&points);
    triangulation.push(super_triangle);

    // Step 2: Insert each point one at a time.
    for point in points {
        // Step 2a: Find all triangles whose circumcircle contains the new point.
        // These are "bad" triangles that violate the Delaunay condition.
        let mut bad_triangles: Vec<Triangle> = Vec::new();

        for triangle in &triangulation {
            let circumcircle = triangle.generate_circumcircle();
            if circumcircle.point_in_circle(&point) {
                bad_triangles.push(*triangle);
            }
        }

        // Step 2b: Determine the boundary polygon of the "bad" region.
        // An edge is on the boundary if it is not shared by any other bad triangle.
        let mut polygon: Vec<Edge> = Vec::new();

        for triangle in &bad_triangles {
            let edges = triangle.edges();
            let bad_triangles_without_triangle: Vec<Triangle> = bad_triangles
                .iter()
                .filter(|t| t != &triangle)
                .cloned()
                .collect();
            for edge in edges {
                if !edge_is_shared_by_triangles(&edge, &bad_triangles_without_triangle) {
                    polygon.push(edge);
                }
            }
        }

        // Step 2c: Remove all bad triangles from the triangulation.
        for bad_triangle in &bad_triangles {
            triangulation.retain(|triangle| triangle != bad_triangle);
        }

        // Step 2d: Re-triangulate the polygonal hole by connecting each
        // boundary edge to the new point.
        for edge in &polygon {
            let new_tri = retriangulate(edge, &point);
            triangulation.push(new_tri);
        }
    }

    // Step 3: Remove any triangles that share vertices with the super-triangle.
    remove_triangles_with_vertices_from_super_triangle(&triangulation, &super_triangle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bowyer_watson() {
        let square = vec![
            Point2D {
                x: 0.0,
                y: 0.0,
                index: 0,
            },
            Point2D {
                x: 1.0,
                y: 0.0,
                index: 1,
            },
            Point2D {
                x: 0.0,
                y: 1.0,
                index: 2,
            },
            Point2D {
                x: 1.0,
                y: 1.0,
                index: 3,
            },
        ];
        let result = bowyer_watson(square);
        assert_eq!(result.len(), 2);
    }
}

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
//! let triangles = bowyer_watson(points).unwrap();
//! assert_eq!(triangles.len(), 2);
//! ```

use error::MeshingError;
use geometry::{create_super_triangle, edge_is_shared_by_triangles, retriangulate};
use geometry_3d::{create_super_tetrahedron, face_is_shared_by_tetrahedra, retetrahedralize};
pub use model::{Edge, Face, Point2D, Point3D, Sphere, Tetrahedron, Triangle};
use tetrahedron_utils::remove_tetrahedra_with_vertices_from_super_tetrahedron;
use triangle_utils::remove_triangles_with_vertices_from_super_triangle;

pub mod error;
pub mod export;
mod geometry;
mod geometry_3d;
mod model;
mod tetrahedron_utils;
mod triangle_utils;
#[cfg(target_arch = "wasm32")]
pub mod wasm;

/// Computes the Delaunay triangulation of a set of 2D points using the
/// Bowyer-Watson incremental insertion algorithm.
///
/// Returns a list of [`Triangle`]s forming the Delaunay triangulation.
///
/// # Errors
///
/// Returns [`MeshingError::EmptyInput`] if `points` is empty.
/// Returns [`MeshingError::InsufficientPoints`] if fewer than 3 points are given.
pub fn bowyer_watson(points: Vec<Point2D>) -> Result<Vec<Triangle>, MeshingError> {
    if points.is_empty() {
        return Err(MeshingError::EmptyInput);
    }
    if points.len() < 3 {
        return Err(MeshingError::InsufficientPoints(points.len()));
    }

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
    Ok(remove_triangles_with_vertices_from_super_triangle(
        &triangulation,
        &super_triangle,
    ))
}

pub fn bowyer_watson_3d(points: Vec<Point3D>) -> Vec<Tetrahedron> {
    let mut tetrahedralization: Vec<Tetrahedron> = Vec::new();
    let super_tetrahedron = create_super_tetrahedron(&points);
    tetrahedralization.push(super_tetrahedron);

    for point in points {
        let mut bad_tetrahedra: Vec<Tetrahedron> = Vec::new();

        for tet in &tetrahedralization {
            let circumsphere = tet.circumsphere();
            if circumsphere.point_in_sphere(&point) {
                bad_tetrahedra.push(*tet);
            }
        }

        let mut boundary_faces: Vec<Face> = Vec::new();

        for tet in &bad_tetrahedra {
            let faces = tet.faces();
            let bad_tetrahedra_without_tet: Vec<Tetrahedron> = bad_tetrahedra
                .iter()
                .filter(|t| t != &tet)
                .cloned()
                .collect();
            for face in faces {
                if !face_is_shared_by_tetrahedra(&face, &bad_tetrahedra_without_tet) {
                    boundary_faces.push(face);
                }
            }
        }

        for bad_tet in &bad_tetrahedra {
            tetrahedralization.retain(|tet| tet != bad_tet);
        }

        for face in &boundary_faces {
            let new_tet = retetrahedralize(face, &point);
            tetrahedralization.push(new_tet);
        }
    }

    remove_tetrahedra_with_vertices_from_super_tetrahedron(
        &tetrahedralization,
        &super_tetrahedron,
    )
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
        let result = bowyer_watson(square).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_bowyer_watson_empty_input() {
        let result = bowyer_watson(vec![]);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "input points vector is empty"
        );
    }

    #[test]
    fn test_bowyer_watson_insufficient_points() {
        let points = vec![
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
        ];
        let result = bowyer_watson(points);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "insufficient points for triangulation: need at least 3, got 2"
        );
    }

    #[test]
    fn test_bowyer_watson_3d_single_tetrahedron() {
        let points = vec![
            Point3D {
                index: 0,
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            Point3D {
                index: 1,
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            Point3D {
                index: 2,
                x: 0.5,
                y: 1.0,
                z: 0.0,
            },
            Point3D {
                index: 3,
                x: 0.5,
                y: 0.5,
                z: 1.0,
            },
        ];
        let result = bowyer_watson_3d(points);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_bowyer_watson_3d_cube() {
        let points = vec![
            Point3D {
                index: 0,
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            Point3D {
                index: 1,
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            Point3D {
                index: 2,
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            Point3D {
                index: 3,
                x: 1.0,
                y: 1.0,
                z: 0.0,
            },
            Point3D {
                index: 4,
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            Point3D {
                index: 5,
                x: 1.0,
                y: 0.0,
                z: 1.0,
            },
            Point3D {
                index: 6,
                x: 0.0,
                y: 1.0,
                z: 1.0,
            },
            Point3D {
                index: 7,
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
        ];
        let result = bowyer_watson_3d(points);
        assert!(result.len() >= 5);
        for tet in &result {
            for v in tet.vertices() {
                assert!(v.index >= 0 && v.index <= 7);
            }
        }
    }

    #[test]
    fn test_circumsphere() {
        let tet = Tetrahedron {
            a: Point3D {
                index: 0,
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            b: Point3D {
                index: 1,
                x: -1.0,
                y: 0.0,
                z: 0.0,
            },
            c: Point3D {
                index: 2,
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            d: Point3D {
                index: 3,
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
        };
        let sphere = tet.circumsphere();
        let eps = 1e-10;
        let d_a = sphere.center.distance(&tet.a);
        let d_b = sphere.center.distance(&tet.b);
        let d_c = sphere.center.distance(&tet.c);
        let d_d = sphere.center.distance(&tet.d);
        assert!((d_a - sphere.radius).abs() < eps);
        assert!((d_b - sphere.radius).abs() < eps);
        assert!((d_c - sphere.radius).abs() < eps);
        assert!((d_d - sphere.radius).abs() < eps);
    }
}

use geometry::{create_super_triangle, edge_is_shared_by_triangles, retriangulate};
pub use model::{Edge, Point2D, Triangle};
use triangle_utils::remove_triangles_with_vertices_from_super_triangle;

pub mod export;
mod geometry;
mod model;
mod triangle_utils;
#[cfg(target_arch = "wasm32")]
pub mod wasm;

pub fn bowyer_watson(points: Vec<Point2D>) -> Vec<Triangle> {
    let mut triangulation: Vec<Triangle> = Vec::new();
    let super_triangle = create_super_triangle(&points);
    triangulation.push(super_triangle);

    for point in points {
        let mut bad_triangles: Vec<Triangle> = Vec::new();

        for triangle in &triangulation {
            let circumcircle = triangle.generate_circumcircle();
            if circumcircle.point_in_circle(&point) {
                bad_triangles.push(*triangle);
            }
        }

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

        for bad_triangle in &bad_triangles {
            triangulation.retain(|triangle| triangle != bad_triangle);
        }

        for edge in &polygon {
            let new_tri = retriangulate(edge, &point);
            triangulation.push(new_tri);
        }
    }

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

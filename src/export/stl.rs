use crate::Triangle;

/// Exports a slice of triangles to ASCII STL format.
/// Since the triangles are 2D, z coordinates are set to 0
/// and face normals point in the +z direction (0, 0, 1).
pub fn triangles_to_stl(triangles: &[Triangle], name: &str) -> String {
    let mut result = format!("solid {}\n", name);

    for triangle in triangles {
        result.push_str("  facet normal 0 0 1\n");
        result.push_str("    outer loop\n");
        for vertex in &triangle.vertices() {
            result.push_str(&format!("      vertex {} {} 0\n", vertex.x, vertex.y));
        }
        result.push_str("    endloop\n");
        result.push_str("  endfacet\n");
    }

    result.push_str(&format!("endsolid {}\n", name));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Point2D;

    #[test]
    fn test_triangles_to_stl_empty() {
        let result = triangles_to_stl(&[], "test");
        assert_eq!(result, "solid test\nendsolid test\n");
    }

    #[test]
    fn test_triangles_to_stl_single_triangle() {
        let triangles = vec![Triangle {
            a: Point2D { index: 0, x: 0.0, y: 0.0 },
            b: Point2D { index: 1, x: 1.0, y: 0.0 },
            c: Point2D { index: 2, x: 0.0, y: 1.0 },
        }];

        let result = triangles_to_stl(&triangles, "mesh");
        assert!(result.starts_with("solid mesh\n"));
        assert!(result.ends_with("endsolid mesh\n"));
        assert!(result.contains("facet normal 0 0 1"));
        assert!(result.contains("vertex 0 0 0"));
        assert!(result.contains("vertex 1 0 0"));
        assert!(result.contains("vertex 0 1 0"));
    }

    #[test]
    fn test_triangles_to_stl_multiple_triangles() {
        let triangles = vec![
            Triangle {
                a: Point2D { index: 0, x: 0.0, y: 0.0 },
                b: Point2D { index: 1, x: 1.0, y: 0.0 },
                c: Point2D { index: 2, x: 0.0, y: 1.0 },
            },
            Triangle {
                a: Point2D { index: 1, x: 1.0, y: 0.0 },
                b: Point2D { index: 3, x: 1.0, y: 1.0 },
                c: Point2D { index: 2, x: 0.0, y: 1.0 },
            },
        ];

        let result = triangles_to_stl(&triangles, "quad");
        let facet_count = result.matches("facet normal").count();
        assert_eq!(facet_count, 2);
    }
}

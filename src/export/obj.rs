use crate::export::stl::extract_surface_faces;
use crate::{Face, Point3D, Tetrahedron, Triangle};

/// Exports a slice of triangles to Wavefront OBJ format.
/// Since the triangles are 2D, z coordinates are set to 0.
/// Vertices are deduplicated by index and faces reference vertex positions.
pub fn triangles_to_obj(triangles: &[Triangle]) -> String {
    let mut vertices: Vec<(i64, f64, f64)> = Vec::new();

    for triangle in triangles {
        for vertex in &triangle.vertices() {
            if !vertices.iter().any(|(idx, _, _)| *idx == vertex.index) {
                vertices.push((vertex.index, vertex.x, vertex.y));
            }
        }
    }

    vertices.sort_by_key(|(idx, _, _)| *idx);

    let mut result = String::new();

    for (_, x, y) in &vertices {
        result.push_str(&format!("v {} {} 0\n", x, y));
    }

    for triangle in triangles {
        let a_pos = vertices.iter().position(|(idx, _, _)| *idx == triangle.a.index).unwrap() + 1;
        let b_pos = vertices.iter().position(|(idx, _, _)| *idx == triangle.b.index).unwrap() + 1;
        let c_pos = vertices.iter().position(|(idx, _, _)| *idx == triangle.c.index).unwrap() + 1;
        result.push_str(&format!("f {} {} {}\n", a_pos, b_pos, c_pos));
    }

    result
}

/// Exports a tetrahedral mesh to Wavefront OBJ format by extracting surface faces.
pub fn tetrahedra_to_obj(tetrahedra: &[Tetrahedron]) -> String {
    let surface = extract_surface_faces(tetrahedra);
    faces_to_obj(&surface)
}

/// Exports 3D faces to Wavefront OBJ format.
/// Vertices are deduplicated by index.
pub fn faces_to_obj(faces: &[Face]) -> String {
    let mut vertices: Vec<(i64, Point3D)> = Vec::new();

    for face in faces {
        for vertex in &face.vertices() {
            if !vertices.iter().any(|(idx, _)| *idx == vertex.index) {
                vertices.push((vertex.index, *vertex));
            }
        }
    }

    vertices.sort_by_key(|(idx, _)| *idx);

    let mut result = String::new();

    for (_, v) in &vertices {
        result.push_str(&format!("v {} {} {}\n", v.x, v.y, v.z));
    }

    for face in faces {
        let a_pos = vertices.iter().position(|(idx, _)| *idx == face.a.index).unwrap() + 1;
        let b_pos = vertices.iter().position(|(idx, _)| *idx == face.b.index).unwrap() + 1;
        let c_pos = vertices.iter().position(|(idx, _)| *idx == face.c.index).unwrap() + 1;
        result.push_str(&format!("f {} {} {}\n", a_pos, b_pos, c_pos));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Point2D;

    #[test]
    fn test_triangles_to_obj_empty() {
        let result = triangles_to_obj(&[]);
        assert_eq!(result, "");
    }

    #[test]
    fn test_triangles_to_obj_single_triangle() {
        let triangles = vec![Triangle {
            a: Point2D { index: 0, x: 0.0, y: 0.0 },
            b: Point2D { index: 1, x: 1.0, y: 0.0 },
            c: Point2D { index: 2, x: 0.0, y: 1.0 },
        }];

        let result = triangles_to_obj(&triangles);
        assert!(result.contains("v 0 0 0"));
        assert!(result.contains("v 1 0 0"));
        assert!(result.contains("v 0 1 0"));
        assert!(result.contains("f 1 2 3"));
    }

    #[test]
    fn test_triangles_to_obj_shared_vertices() {
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

        let result = triangles_to_obj(&triangles);
        // Should have 4 unique vertices, not 6
        let vertex_count = result.matches("\nv ").count() + if result.starts_with("v ") { 1 } else { 0 };
        assert_eq!(vertex_count, 4);
        // Two face lines
        let face_count = result.matches("f ").count();
        assert_eq!(face_count, 2);
    }

    #[test]
    fn test_tetrahedra_to_obj_single() {
        let tet = Tetrahedron {
            a: Point3D { index: 0, x: 0.0, y: 0.0, z: 0.0 },
            b: Point3D { index: 1, x: 1.0, y: 0.0, z: 0.0 },
            c: Point3D { index: 2, x: 0.0, y: 1.0, z: 0.0 },
            d: Point3D { index: 3, x: 0.0, y: 0.0, z: 1.0 },
        };
        let result = tetrahedra_to_obj(&[tet]);
        let vertex_count = result.matches("\nv ").count()
            + if result.starts_with("v ") { 1 } else { 0 };
        assert_eq!(vertex_count, 4);
        let face_count = result.matches("f ").count();
        assert_eq!(face_count, 4);
    }

    #[test]
    fn test_tetrahedra_to_obj_empty() {
        let result = tetrahedra_to_obj(&[]);
        assert_eq!(result, "");
    }

    #[test]
    fn test_faces_to_obj_3d_vertices() {
        let face = Face {
            a: Point3D { index: 0, x: 1.0, y: 2.0, z: 3.0 },
            b: Point3D { index: 1, x: 4.0, y: 5.0, z: 6.0 },
            c: Point3D { index: 2, x: 7.0, y: 8.0, z: 9.0 },
        };
        let result = faces_to_obj(&[face]);
        assert!(result.contains("v 1 2 3"));
        assert!(result.contains("v 4 5 6"));
        assert!(result.contains("v 7 8 9"));
        assert!(result.contains("f 1 2 3"));
    }
}

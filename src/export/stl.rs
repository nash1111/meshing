use crate::{Face, Tetrahedron, Triangle};

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

/// Extracts the boundary surface faces from a tetrahedral mesh.
/// A face is on the boundary if it appears in exactly one tetrahedron.
pub fn extract_surface_faces(tetrahedra: &[Tetrahedron]) -> Vec<Face> {
    let all_faces: Vec<Face> = tetrahedra.iter().flat_map(|t| t.faces()).collect();
    let mut surface = Vec::new();
    for face in &all_faces {
        let count = all_faces.iter().filter(|f| *f == face).count();
        if count == 1 {
            // Avoid duplicates in the output (each unique boundary face appears once)
            if !surface.iter().any(|f: &Face| f == face) {
                surface.push(*face);
            }
        }
    }
    surface
}

fn face_normal(face: &Face) -> (f64, f64, f64) {
    let ux = face.b.x - face.a.x;
    let uy = face.b.y - face.a.y;
    let uz = face.b.z - face.a.z;
    let vx = face.c.x - face.a.x;
    let vy = face.c.y - face.a.y;
    let vz = face.c.z - face.a.z;
    let nx = uy * vz - uz * vy;
    let ny = uz * vx - ux * vz;
    let nz = ux * vy - uy * vx;
    let len = (nx * nx + ny * ny + nz * nz).sqrt();
    if len < 1e-15 {
        return (0.0, 0.0, 0.0);
    }
    (nx / len, ny / len, nz / len)
}

/// Exports a tetrahedral mesh to ASCII STL format by extracting surface faces.
pub fn tetrahedra_to_stl(tetrahedra: &[Tetrahedron], name: &str) -> String {
    let surface = extract_surface_faces(tetrahedra);
    faces_to_stl(&surface, name)
}

/// Exports 3D faces to ASCII STL format with computed normals.
pub fn faces_to_stl(faces: &[Face], name: &str) -> String {
    let mut result = format!("solid {}\n", name);

    for face in faces {
        let (nx, ny, nz) = face_normal(face);
        result.push_str(&format!("  facet normal {} {} {}\n", nx, ny, nz));
        result.push_str("    outer loop\n");
        for vertex in &face.vertices() {
            result.push_str(&format!(
                "      vertex {} {} {}\n",
                vertex.x, vertex.y, vertex.z
            ));
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
    use crate::{Point2D, Point3D};

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

    fn single_tet() -> Tetrahedron {
        Tetrahedron {
            a: Point3D { index: 0, x: 0.0, y: 0.0, z: 0.0 },
            b: Point3D { index: 1, x: 1.0, y: 0.0, z: 0.0 },
            c: Point3D { index: 2, x: 0.0, y: 1.0, z: 0.0 },
            d: Point3D { index: 3, x: 0.0, y: 0.0, z: 1.0 },
        }
    }

    #[test]
    fn test_tetrahedra_to_stl_single() {
        let result = tetrahedra_to_stl(&[single_tet()], "tet");
        assert!(result.starts_with("solid tet\n"));
        assert!(result.ends_with("endsolid tet\n"));
        // Single tet has 4 boundary faces
        let facet_count = result.matches("facet normal").count();
        assert_eq!(facet_count, 4);
    }

    #[test]
    fn test_tetrahedra_to_stl_empty() {
        let result = tetrahedra_to_stl(&[], "empty");
        assert_eq!(result, "solid empty\nendsolid empty\n");
    }

    #[test]
    fn test_faces_to_stl_computes_normal() {
        let face = Face {
            a: Point3D { index: 0, x: 0.0, y: 0.0, z: 0.0 },
            b: Point3D { index: 1, x: 1.0, y: 0.0, z: 0.0 },
            c: Point3D { index: 2, x: 0.0, y: 1.0, z: 0.0 },
        };
        let result = faces_to_stl(&[face], "test");
        // Normal should be (0, 0, 1) for a face in the XY plane
        assert!(result.contains("facet normal 0 0 1"));
    }

    #[test]
    fn test_extract_surface_faces_single_tet() {
        let surface = extract_surface_faces(&[single_tet()]);
        assert_eq!(surface.len(), 4);
    }

    #[test]
    fn test_extract_surface_shared_face_excluded() {
        // Two tetrahedra sharing a face — the shared face should be excluded
        let p0 = Point3D { index: 0, x: 0.0, y: 0.0, z: 0.0 };
        let p1 = Point3D { index: 1, x: 1.0, y: 0.0, z: 0.0 };
        let p2 = Point3D { index: 2, x: 0.0, y: 1.0, z: 0.0 };
        let p3 = Point3D { index: 3, x: 0.0, y: 0.0, z: 1.0 };
        let p4 = Point3D { index: 4, x: 0.0, y: 0.0, z: -1.0 };

        let tet1 = Tetrahedron { a: p0, b: p1, c: p2, d: p3 };
        let tet2 = Tetrahedron { a: p0, b: p1, c: p2, d: p4 };
        let surface = extract_surface_faces(&[tet1, tet2]);
        // 2 tets * 4 faces = 8 total, minus 2 (shared face counted in both) = 6
        assert_eq!(surface.len(), 6);
    }
}

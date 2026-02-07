use crate::{Face, Point3D, Tetrahedron};

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

fn face_centroid(face: &Face) -> (f64, f64, f64) {
    (
        (face.a.x + face.b.x + face.c.x) / 3.0,
        (face.a.y + face.b.y + face.c.y) / 3.0,
        (face.a.z + face.b.z + face.c.z) / 3.0,
    )
}

fn dot(a: (f64, f64, f64), b: (f64, f64, f64)) -> f64 {
    a.0 * b.0 + a.1 * b.1 + a.2 * b.2
}

fn remove_face_from_front(front: &mut Vec<Face>, face: &Face) {
    if let Some(pos) = front.iter().position(|f| f == face) {
        front.swap_remove(pos);
    }
}

/// Generates a tetrahedral mesh by advancing from boundary faces inward.
///
/// Takes a closed surface mesh (as `faces`) and a set of interior/boundary `points`,
/// then grows tetrahedra from the front until the volume is filled.
pub fn advancing_front(faces: Vec<Face>, points: Vec<Point3D>) -> Vec<Tetrahedron> {
    if faces.is_empty() {
        return Vec::new();
    }

    let mut front: Vec<Face> = faces;
    let mut tetrahedra: Vec<Tetrahedron> = Vec::new();
    let mut all_points: Vec<Point3D> = points;

    let max_iterations = front.len() * 100;
    let mut iterations = 0;

    while !front.is_empty() && iterations < max_iterations {
        iterations += 1;
        let face = front[0];
        let normal = face_normal(&face);
        let centroid = face_centroid(&face);

        // Find the best candidate point: closest point on the positive normal side
        let face_verts = face.vertices();
        let mut best_point: Option<Point3D> = None;
        let mut best_dist = f64::MAX;

        for &p in &all_points {
            // Skip points that are vertices of the current face
            if face_verts.contains(&p) {
                continue;
            }
            let dx = p.x - centroid.0;
            let dy = p.y - centroid.1;
            let dz = p.z - centroid.2;
            let side = dot(normal, (dx, dy, dz));

            // Point must be on the positive normal side
            if side > 1e-10 {
                let dist = dx * dx + dy * dy + dz * dz;
                if dist < best_dist {
                    best_dist = dist;
                    best_point = Some(p);
                }
            }
        }

        // If no point found on positive side, try the negative side
        if best_point.is_none() {
            for &p in &all_points {
                if face_verts.contains(&p) {
                    continue;
                }
                let dx = p.x - centroid.0;
                let dy = p.y - centroid.1;
                let dz = p.z - centroid.2;
                let side = dot(normal, (dx, dy, dz));
                if side < -1e-10 {
                    let dist = dx * dx + dy * dy + dz * dz;
                    if dist < best_dist {
                        best_dist = dist;
                        best_point = Some(p);
                    }
                }
            }
        }

        // If still no point found, generate one along the normal at ideal distance
        if best_point.is_none() {
            let edge_len = face.a.distance(&face.b)
                .min(face.b.distance(&face.c))
                .min(face.a.distance(&face.c));
            let ideal_dist = edge_len * 0.8;
            let new_index = all_points.len() as i64;
            let new_point = Point3D {
                index: new_index,
                x: centroid.0 + normal.0 * ideal_dist,
                y: centroid.1 + normal.1 * ideal_dist,
                z: centroid.2 + normal.2 * ideal_dist,
            };
            all_points.push(new_point);
            best_point = Some(new_point);
        }

        let chosen = best_point.unwrap();

        // Create the new tetrahedron
        let tet = Tetrahedron {
            a: face.a,
            b: face.b,
            c: face.c,
            d: chosen,
        };
        tetrahedra.push(tet);

        // Remove the current face from the front
        remove_face_from_front(&mut front, &face);

        // Update the front with the 3 new faces of the tetrahedron (excluding the original face)
        let new_faces = [
            Face {
                a: face.a,
                b: face.b,
                c: chosen,
            },
            Face {
                a: face.b,
                b: face.c,
                c: chosen,
            },
            Face {
                a: face.a,
                b: face.c,
                c: chosen,
            },
        ];

        for new_face in &new_faces {
            // If this face already exists in the front, it's an internal face — remove it
            if let Some(pos) = front.iter().position(|f| f == new_face) {
                front.swap_remove(pos);
            } else {
                front.push(*new_face);
            }
        }
    }

    tetrahedra
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        let result = advancing_front(Vec::new(), Vec::new());
        assert!(result.is_empty());
    }

    #[test]
    fn test_single_tetrahedron() {
        let p0 = Point3D { index: 0, x: 0.0, y: 0.0, z: 0.0 };
        let p1 = Point3D { index: 1, x: 1.0, y: 0.0, z: 0.0 };
        let p2 = Point3D { index: 2, x: 0.5, y: 1.0, z: 0.0 };
        let p3 = Point3D { index: 3, x: 0.5, y: 0.3, z: 1.0 };

        // 4 faces of a tetrahedron (with outward-facing normals)
        let faces = vec![
            Face { a: p0, b: p2, c: p1 }, // bottom
            Face { a: p0, b: p1, c: p3 }, // front
            Face { a: p1, b: p2, c: p3 }, // right
            Face { a: p0, b: p3, c: p2 }, // left
        ];
        let points = vec![p0, p1, p2, p3];

        let result = advancing_front(faces, points);
        assert!(!result.is_empty());
        // Should produce at least 1 tetrahedron
        assert!(result.len() >= 1);
    }

    #[test]
    fn test_all_tetrahedra_have_nonzero_volume() {
        let p0 = Point3D { index: 0, x: 0.0, y: 0.0, z: 0.0 };
        let p1 = Point3D { index: 1, x: 1.0, y: 0.0, z: 0.0 };
        let p2 = Point3D { index: 2, x: 0.5, y: 1.0, z: 0.0 };
        let p3 = Point3D { index: 3, x: 0.5, y: 0.3, z: 1.0 };

        let faces = vec![
            Face { a: p0, b: p2, c: p1 },
            Face { a: p0, b: p1, c: p3 },
            Face { a: p1, b: p2, c: p3 },
            Face { a: p0, b: p3, c: p2 },
        ];
        let points = vec![p0, p1, p2, p3];
        let result = advancing_front(faces, points);
        for tet in &result {
            assert!(tet.signed_volume().abs() > 1e-15, "Degenerate tetrahedron found");
        }
    }

    #[test]
    fn test_faces_no_points() {
        // Faces provided but no points — should still generate via normal projection
        let p0 = Point3D { index: 0, x: 0.0, y: 0.0, z: 0.0 };
        let p1 = Point3D { index: 1, x: 1.0, y: 0.0, z: 0.0 };
        let p2 = Point3D { index: 2, x: 0.5, y: 1.0, z: 0.0 };
        let faces = vec![Face { a: p0, b: p1, c: p2 }];
        let result = advancing_front(faces, Vec::new());
        assert!(!result.is_empty());
    }

    #[test]
    fn test_cube_surface() {
        // 8 vertices of a unit cube
        let p = [
            Point3D { index: 0, x: 0.0, y: 0.0, z: 0.0 },
            Point3D { index: 1, x: 1.0, y: 0.0, z: 0.0 },
            Point3D { index: 2, x: 1.0, y: 1.0, z: 0.0 },
            Point3D { index: 3, x: 0.0, y: 1.0, z: 0.0 },
            Point3D { index: 4, x: 0.0, y: 0.0, z: 1.0 },
            Point3D { index: 5, x: 1.0, y: 0.0, z: 1.0 },
            Point3D { index: 6, x: 1.0, y: 1.0, z: 1.0 },
            Point3D { index: 7, x: 0.0, y: 1.0, z: 1.0 },
        ];

        // 12 triangular faces (2 per cube face, normals pointing outward)
        let faces = vec![
            // Bottom (z=0), normal -z
            Face { a: p[0], b: p[1], c: p[2] },
            Face { a: p[0], b: p[2], c: p[3] },
            // Top (z=1), normal +z
            Face { a: p[4], b: p[6], c: p[5] },
            Face { a: p[4], b: p[7], c: p[6] },
            // Front (y=0), normal -y
            Face { a: p[0], b: p[5], c: p[1] },
            Face { a: p[0], b: p[4], c: p[5] },
            // Back (y=1), normal +y
            Face { a: p[2], b: p[7], c: p[3] },
            Face { a: p[2], b: p[6], c: p[7] },
            // Left (x=0), normal -x
            Face { a: p[0], b: p[3], c: p[7] },
            Face { a: p[0], b: p[7], c: p[4] },
            // Right (x=1), normal +x
            Face { a: p[1], b: p[5], c: p[6] },
            Face { a: p[1], b: p[6], c: p[2] },
        ];
        let points = p.to_vec();

        let result = advancing_front(faces, points);
        assert!(!result.is_empty());
        // A cube needs at least 5 tetrahedra
        assert!(result.len() >= 5);
    }
}

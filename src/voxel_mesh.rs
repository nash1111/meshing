use crate::{Point3D, Tetrahedron};

/// Generates a tetrahedral volume mesh from a uniform voxel grid.
///
/// Divides the bounding box into `nx * ny * nz` cells. For each cell whose
/// center satisfies `is_inside`, the hexahedral cell is decomposed into 5
/// tetrahedra.
///
/// # Arguments
///
/// * `min` - Minimum corner of the bounding box.
/// * `max` - Maximum corner of the bounding box.
/// * `nx`, `ny`, `nz` - Number of cells along each axis.
/// * `is_inside` - Predicate function; returns `true` if a point is inside the domain.
///
/// # Returns
///
/// A vector of [`Tetrahedron`]s filling the region where `is_inside` is true.
///
/// # Examples
///
/// ```
/// use meshing::voxel_mesh::voxel_mesh;
/// use meshing::Point3D;
///
/// let min = Point3D { index: 0, x: 0.0, y: 0.0, z: 0.0 };
/// let max = Point3D { index: 0, x: 1.0, y: 1.0, z: 1.0 };
/// let tets = voxel_mesh(min, max, 2, 2, 2, &|_| true);
/// assert_eq!(tets.len(), 40); // 8 cells × 5 tets
/// ```
pub fn voxel_mesh(
    min: Point3D,
    max: Point3D,
    nx: usize,
    ny: usize,
    nz: usize,
    is_inside: &dyn Fn(&Point3D) -> bool,
) -> Vec<Tetrahedron> {
    let dx = (max.x - min.x) / nx as f64;
    let dy = (max.y - min.y) / ny as f64;
    let dz = (max.z - min.z) / nz as f64;

    let mut tetrahedra = Vec::new();

    for i in 0..nx {
        for j in 0..ny {
            for k in 0..nz {
                let center = Point3D {
                    index: -1,
                    x: min.x + (i as f64 + 0.5) * dx,
                    y: min.y + (j as f64 + 0.5) * dy,
                    z: min.z + (k as f64 + 0.5) * dz,
                };

                if !is_inside(&center) {
                    continue;
                }

                // 8 corner vertices, indices based on grid vertex position
                let vertex_index = |ix: usize, iy: usize, iz: usize| -> i64 {
                    (ix * (ny + 1) * (nz + 1) + iy * (nz + 1) + iz) as i64
                };

                let p0 = Point3D {
                    index: vertex_index(i, j, k),
                    x: min.x + i as f64 * dx,
                    y: min.y + j as f64 * dy,
                    z: min.z + k as f64 * dz,
                };
                let p1 = Point3D {
                    index: vertex_index(i + 1, j, k),
                    x: min.x + (i + 1) as f64 * dx,
                    y: min.y + j as f64 * dy,
                    z: min.z + k as f64 * dz,
                };
                let p2 = Point3D {
                    index: vertex_index(i + 1, j + 1, k),
                    x: min.x + (i + 1) as f64 * dx,
                    y: min.y + (j + 1) as f64 * dy,
                    z: min.z + k as f64 * dz,
                };
                let p3 = Point3D {
                    index: vertex_index(i, j + 1, k),
                    x: min.x + i as f64 * dx,
                    y: min.y + (j + 1) as f64 * dy,
                    z: min.z + k as f64 * dz,
                };
                let p4 = Point3D {
                    index: vertex_index(i, j, k + 1),
                    x: min.x + i as f64 * dx,
                    y: min.y + j as f64 * dy,
                    z: min.z + (k + 1) as f64 * dz,
                };
                let p5 = Point3D {
                    index: vertex_index(i + 1, j, k + 1),
                    x: min.x + (i + 1) as f64 * dx,
                    y: min.y + j as f64 * dy,
                    z: min.z + (k + 1) as f64 * dz,
                };
                let p6 = Point3D {
                    index: vertex_index(i + 1, j + 1, k + 1),
                    x: min.x + (i + 1) as f64 * dx,
                    y: min.y + (j + 1) as f64 * dy,
                    z: min.z + (k + 1) as f64 * dz,
                };
                let p7 = Point3D {
                    index: vertex_index(i, j + 1, k + 1),
                    x: min.x + i as f64 * dx,
                    y: min.y + (j + 1) as f64 * dy,
                    z: min.z + (k + 1) as f64 * dz,
                };

                // Standard 5-tetrahedra decomposition of a hexahedron
                tetrahedra.push(Tetrahedron { a: p0, b: p1, c: p3, d: p4 });
                tetrahedra.push(Tetrahedron { a: p1, b: p2, c: p3, d: p6 });
                tetrahedra.push(Tetrahedron { a: p1, b: p4, c: p5, d: p6 });
                tetrahedra.push(Tetrahedron { a: p3, b: p4, c: p6, d: p7 });
                tetrahedra.push(Tetrahedron { a: p1, b: p3, c: p4, d: p6 });
            }
        }
    }

    tetrahedra
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_cell_always_inside() {
        let min = Point3D { index: 0, x: 0.0, y: 0.0, z: 0.0 };
        let max = Point3D { index: 0, x: 1.0, y: 1.0, z: 1.0 };
        let result = voxel_mesh(min, max, 1, 1, 1, &|_| true);
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_2x2x2_all_inside() {
        let min = Point3D { index: 0, x: 0.0, y: 0.0, z: 0.0 };
        let max = Point3D { index: 0, x: 1.0, y: 1.0, z: 1.0 };
        let result = voxel_mesh(min, max, 2, 2, 2, &|_| true);
        assert_eq!(result.len(), 40);
    }

    #[test]
    fn test_sphere_containment() {
        let min = Point3D { index: 0, x: -1.0, y: -1.0, z: -1.0 };
        let max = Point3D { index: 0, x: 1.0, y: 1.0, z: 1.0 };
        let result = voxel_mesh(min, max, 4, 4, 4, &|p| {
            p.x * p.x + p.y * p.y + p.z * p.z <= 1.0
        });
        assert!(!result.is_empty());
        assert!(result.len() < 64 * 5);
    }

    #[test]
    fn test_empty_domain() {
        let min = Point3D { index: 0, x: 0.0, y: 0.0, z: 0.0 };
        let max = Point3D { index: 0, x: 1.0, y: 1.0, z: 1.0 };
        let result = voxel_mesh(min, max, 4, 4, 4, &|_| false);
        assert!(result.is_empty());
    }

    #[test]
    fn test_all_tetrahedra_have_nonzero_volume() {
        let min = Point3D { index: 0, x: 0.0, y: 0.0, z: 0.0 };
        let max = Point3D { index: 0, x: 1.0, y: 1.0, z: 1.0 };
        let result = voxel_mesh(min, max, 3, 3, 3, &|_| true);
        for tet in &result {
            assert!(tet.signed_volume().abs() > 1e-15, "Degenerate tetrahedron found");
        }
    }

    #[test]
    fn test_asymmetric_resolution() {
        let min = Point3D { index: 0, x: 0.0, y: 0.0, z: 0.0 };
        let max = Point3D { index: 0, x: 1.0, y: 1.0, z: 1.0 };
        let result = voxel_mesh(min, max, 1, 2, 3, &|_| true);
        // 1×2×3 = 6 cells × 5 tets = 30
        assert_eq!(result.len(), 30);
    }

    #[test]
    fn test_shared_vertex_indices() {
        let min = Point3D { index: 0, x: 0.0, y: 0.0, z: 0.0 };
        let max = Point3D { index: 0, x: 1.0, y: 1.0, z: 1.0 };
        let result = voxel_mesh(min, max, 2, 2, 2, &|_| true);
        // Collect all unique vertex indices
        let mut indices: Vec<i64> = Vec::new();
        for tet in &result {
            for v in tet.vertices() {
                if !indices.contains(&v.index) {
                    indices.push(v.index);
                }
            }
        }
        // 2x2x2 grid has 3x3x3 = 27 unique vertices
        assert_eq!(indices.len(), 27);
    }
}

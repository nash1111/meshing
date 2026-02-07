use crate::{Point3D, Tetrahedron};

#[derive(Clone, Copy)]
struct Bounds {
    min_x: f64,
    min_y: f64,
    min_z: f64,
    max_x: f64,
    max_y: f64,
    max_z: f64,
}

fn subdivide(
    b: Bounds,
    depth: usize,
    max_depth: usize,
    is_inside: &dyn Fn(&Point3D) -> bool,
    next_index: &mut i64,
    tetrahedra: &mut Vec<Tetrahedron>,
) {
    if depth >= max_depth {
        let center = Point3D {
            index: -1,
            x: (b.min_x + b.max_x) / 2.0,
            y: (b.min_y + b.max_y) / 2.0,
            z: (b.min_z + b.max_z) / 2.0,
        };
        if !is_inside(&center) {
            return;
        }

        let idx = *next_index;
        *next_index += 8;
        let p = [
            Point3D { index: idx, x: b.min_x, y: b.min_y, z: b.min_z },
            Point3D { index: idx + 1, x: b.max_x, y: b.min_y, z: b.min_z },
            Point3D { index: idx + 2, x: b.max_x, y: b.max_y, z: b.min_z },
            Point3D { index: idx + 3, x: b.min_x, y: b.max_y, z: b.min_z },
            Point3D { index: idx + 4, x: b.min_x, y: b.min_y, z: b.max_z },
            Point3D { index: idx + 5, x: b.max_x, y: b.min_y, z: b.max_z },
            Point3D { index: idx + 6, x: b.max_x, y: b.max_y, z: b.max_z },
            Point3D { index: idx + 7, x: b.min_x, y: b.max_y, z: b.max_z },
        ];

        // Standard 5-tetrahedra decomposition of a hexahedron
        tetrahedra.push(Tetrahedron { a: p[0], b: p[1], c: p[3], d: p[4] });
        tetrahedra.push(Tetrahedron { a: p[1], b: p[2], c: p[3], d: p[6] });
        tetrahedra.push(Tetrahedron { a: p[1], b: p[4], c: p[5], d: p[6] });
        tetrahedra.push(Tetrahedron { a: p[3], b: p[4], c: p[6], d: p[7] });
        tetrahedra.push(Tetrahedron { a: p[1], b: p[3], c: p[4], d: p[6] });
        return;
    }

    let mid_x = (b.min_x + b.max_x) / 2.0;
    let mid_y = (b.min_y + b.max_y) / 2.0;
    let mid_z = (b.min_z + b.max_z) / 2.0;

    let octants = [
        Bounds { min_x: b.min_x, min_y: b.min_y, min_z: b.min_z, max_x: mid_x, max_y: mid_y, max_z: mid_z },
        Bounds { min_x: mid_x, min_y: b.min_y, min_z: b.min_z, max_x: b.max_x, max_y: mid_y, max_z: mid_z },
        Bounds { min_x: b.min_x, min_y: mid_y, min_z: b.min_z, max_x: mid_x, max_y: b.max_y, max_z: mid_z },
        Bounds { min_x: mid_x, min_y: mid_y, min_z: b.min_z, max_x: b.max_x, max_y: b.max_y, max_z: mid_z },
        Bounds { min_x: b.min_x, min_y: b.min_y, min_z: mid_z, max_x: mid_x, max_y: mid_y, max_z: b.max_z },
        Bounds { min_x: mid_x, min_y: b.min_y, min_z: mid_z, max_x: b.max_x, max_y: mid_y, max_z: b.max_z },
        Bounds { min_x: b.min_x, min_y: mid_y, min_z: mid_z, max_x: mid_x, max_y: b.max_y, max_z: b.max_z },
        Bounds { min_x: mid_x, min_y: mid_y, min_z: mid_z, max_x: b.max_x, max_y: b.max_y, max_z: b.max_z },
    ];

    for octant in &octants {
        subdivide(*octant, depth + 1, max_depth, is_inside, next_index, tetrahedra);
    }
}

/// Generates a tetrahedral mesh using octree-based spatial subdivision.
///
/// Recursively subdivides the bounding box into octants up to `max_depth` levels.
/// At each leaf cell whose center satisfies `is_inside`, a hexahedral cell is
/// decomposed into 5 tetrahedra.
///
/// # Arguments
///
/// * `min` - Minimum corner of the bounding box.
/// * `max` - Maximum corner of the bounding box.
/// * `max_depth` - Number of recursive subdivision levels (depth 1 = 8 cells).
/// * `is_inside` - Predicate function; returns `true` if a point is inside the domain.
///
/// # Returns
///
/// A vector of [`Tetrahedron`]s filling the region where `is_inside` is true.
///
/// # Examples
///
/// ```
/// use meshing::octree::octree_mesh;
/// use meshing::Point3D;
///
/// let min = Point3D { index: 0, x: 0.0, y: 0.0, z: 0.0 };
/// let max = Point3D { index: 0, x: 1.0, y: 1.0, z: 1.0 };
/// let tets = octree_mesh(min, max, 1, &|_| true);
/// assert_eq!(tets.len(), 40); // 8 cells × 5 tets
/// ```
pub fn octree_mesh(
    min: Point3D,
    max: Point3D,
    max_depth: usize,
    is_inside: &dyn Fn(&Point3D) -> bool,
) -> Vec<Tetrahedron> {
    let mut tetrahedra = Vec::new();
    let mut next_index: i64 = 0;
    let bounds = Bounds {
        min_x: min.x,
        min_y: min.y,
        min_z: min.z,
        max_x: max.x,
        max_y: max.y,
        max_z: max.z,
    };
    subdivide(bounds, 0, max_depth, is_inside, &mut next_index, &mut tetrahedra);
    tetrahedra
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_cell_always_inside() {
        let min = Point3D { index: 0, x: 0.0, y: 0.0, z: 0.0 };
        let max = Point3D { index: 0, x: 1.0, y: 1.0, z: 1.0 };
        let result = octree_mesh(min, max, 0, &|_| true);
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_depth_1_all_inside() {
        let min = Point3D { index: 0, x: 0.0, y: 0.0, z: 0.0 };
        let max = Point3D { index: 0, x: 1.0, y: 1.0, z: 1.0 };
        let result = octree_mesh(min, max, 1, &|_| true);
        assert_eq!(result.len(), 40);
    }

    #[test]
    fn test_sphere_containment() {
        let min = Point3D { index: 0, x: -1.0, y: -1.0, z: -1.0 };
        let max = Point3D { index: 0, x: 1.0, y: 1.0, z: 1.0 };
        let result = octree_mesh(min, max, 2, &|p| {
            p.x * p.x + p.y * p.y + p.z * p.z <= 1.0
        });
        assert!(!result.is_empty());
        assert!(result.len() < 64 * 5);
    }

    #[test]
    fn test_all_tetrahedra_have_nonzero_volume() {
        let min = Point3D { index: 0, x: 0.0, y: 0.0, z: 0.0 };
        let max = Point3D { index: 0, x: 1.0, y: 1.0, z: 1.0 };
        let result = octree_mesh(min, max, 2, &|_| true);
        for tet in &result {
            assert!(tet.signed_volume().abs() > 1e-15, "Degenerate tetrahedron found");
        }
    }

    #[test]
    fn test_depth_2_cell_count() {
        let min = Point3D { index: 0, x: 0.0, y: 0.0, z: 0.0 };
        let max = Point3D { index: 0, x: 1.0, y: 1.0, z: 1.0 };
        let result = octree_mesh(min, max, 2, &|_| true);
        // depth=2 → 64 leaf cells × 5 tets = 320
        assert_eq!(result.len(), 320);
    }

    #[test]
    fn test_partial_containment() {
        let min = Point3D { index: 0, x: 0.0, y: 0.0, z: 0.0 };
        let max = Point3D { index: 0, x: 2.0, y: 2.0, z: 2.0 };
        // Only accept cells whose center x < 1.0 (half the domain)
        let result = octree_mesh(min, max, 1, &|p| p.x < 1.0);
        // 8 octants at depth 1, 4 have center.x < 1.0
        assert_eq!(result.len(), 4 * 5);
    }

    #[test]
    fn test_empty_domain() {
        let min = Point3D { index: 0, x: 0.0, y: 0.0, z: 0.0 };
        let max = Point3D { index: 0, x: 1.0, y: 1.0, z: 1.0 };
        let result = octree_mesh(min, max, 2, &|_| false);
        assert!(result.is_empty());
    }
}

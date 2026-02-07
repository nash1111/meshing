use crate::advancing_front::advancing_front;
use crate::delaunay_refinement::delaunay_refinement;
use crate::marching_cubes::marching_cubes;
use crate::octree::octree_mesh;
use crate::voxel_mesh::voxel_mesh;
use crate::{Face, Point3D, Tetrahedron};

/// Extracts unique points from a tetrahedral mesh.
fn extract_unique_points(tetrahedra: &[Tetrahedron]) -> Vec<Point3D> {
    let mut points: Vec<Point3D> = Vec::new();
    for tet in tetrahedra {
        for v in tet.vertices() {
            if !points.iter().any(|p| p.index == v.index) {
                points.push(v);
            }
        }
    }
    points.sort_by_key(|p| p.index);
    points
}

/// Runs Marching Cubes to extract a surface, then fills the interior with
/// Advancing Front to produce a tetrahedral volume mesh.
///
/// This is useful for converting an implicit surface (scalar field) directly
/// into a volumetric tetrahedral mesh in one step.
///
/// # Arguments
///
/// * `nx`, `ny`, `nz` - Grid resolution for Marching Cubes.
/// * `min`, `max` - Bounding box corners.
/// * `scalar_field` - Implicit function `f(x,y,z)` defining the surface at `f = iso_value`.
/// * `iso_value` - Isosurface threshold.
pub fn surface_to_volume(
    nx: usize,
    ny: usize,
    nz: usize,
    min: Point3D,
    max: Point3D,
    scalar_field: &dyn Fn(f64, f64, f64) -> f64,
    iso_value: f64,
) -> Vec<Tetrahedron> {
    let faces = marching_cubes(nx, ny, nz, min, max, scalar_field, iso_value);
    if faces.is_empty() {
        return Vec::new();
    }
    let points = collect_face_points(&faces);
    advancing_front(faces, points)
}

/// Generates an octree mesh and then refines it for quality.
///
/// Combines octree spatial subdivision with Delaunay refinement to produce
/// a quality tetrahedral mesh. The octree provides the initial coarse mesh,
/// and refinement improves element shapes.
///
/// # Arguments
///
/// * `min`, `max` - Bounding box corners.
/// * `max_depth` - Octree subdivision depth.
/// * `is_inside` - Domain containment predicate.
/// * `max_radius_edge_ratio` - Quality threshold for refinement (lower = better quality).
pub fn octree_refined(
    min: Point3D,
    max: Point3D,
    max_depth: usize,
    is_inside: &dyn Fn(&Point3D) -> bool,
    max_radius_edge_ratio: f64,
) -> Vec<Tetrahedron> {
    let tets = octree_mesh(min, max, max_depth, is_inside);
    if tets.is_empty() {
        return Vec::new();
    }
    let points = extract_unique_points(&tets);
    delaunay_refinement(points, max_radius_edge_ratio)
}

/// Generates a voxel mesh and then refines it for quality.
///
/// Combines uniform voxel meshing with Delaunay refinement to produce
/// a quality tetrahedral mesh.
///
/// # Arguments
///
/// * `min`, `max` - Bounding box corners.
/// * `nx`, `ny`, `nz` - Voxel grid resolution.
/// * `is_inside` - Domain containment predicate.
/// * `max_radius_edge_ratio` - Quality threshold for refinement.
pub fn voxel_refined(
    min: Point3D,
    max: Point3D,
    nx: usize,
    ny: usize,
    nz: usize,
    is_inside: &dyn Fn(&Point3D) -> bool,
    max_radius_edge_ratio: f64,
) -> Vec<Tetrahedron> {
    let tets = voxel_mesh(min, max, nx, ny, nz, is_inside);
    if tets.is_empty() {
        return Vec::new();
    }
    let points = extract_unique_points(&tets);
    delaunay_refinement(points, max_radius_edge_ratio)
}

/// Applies Delaunay refinement to an existing tetrahedral mesh by extracting
/// its unique vertices and re-meshing with quality constraints.
///
/// # Arguments
///
/// * `tetrahedra` - Input tetrahedral mesh from any source.
/// * `max_radius_edge_ratio` - Quality threshold (lower = better quality, 2.0 is typical).
pub fn refine_tetrahedra(
    tetrahedra: &[Tetrahedron],
    max_radius_edge_ratio: f64,
) -> Vec<Tetrahedron> {
    if tetrahedra.is_empty() {
        return Vec::new();
    }
    let points = extract_unique_points(tetrahedra);
    delaunay_refinement(points, max_radius_edge_ratio)
}

fn collect_face_points(faces: &[Face]) -> Vec<Point3D> {
    let mut points: Vec<Point3D> = Vec::new();
    for face in faces {
        for v in face.vertices() {
            if !points.iter().any(|p| p.index == v.index) {
                points.push(v);
            }
        }
    }
    points.sort_by_key(|p| p.index);
    points
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sphere_field(x: f64, y: f64, z: f64) -> f64 {
        x * x + y * y + z * z - 1.0
    }

    #[test]
    fn test_surface_to_volume_sphere() {
        let min = Point3D { index: 0, x: -2.0, y: -2.0, z: -2.0 };
        let max = Point3D { index: 0, x: 2.0, y: 2.0, z: 2.0 };
        let result = surface_to_volume(8, 8, 8, min, max, &sphere_field, 0.0);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_surface_to_volume_empty_field() {
        let min = Point3D { index: 0, x: -1.0, y: -1.0, z: -1.0 };
        let max = Point3D { index: 0, x: 1.0, y: 1.0, z: 1.0 };
        // Field always positive → no isosurface
        let result = surface_to_volume(4, 4, 4, min, max, &|_, _, _| 10.0, 0.0);
        assert!(result.is_empty());
    }

    #[test]
    fn test_octree_refined() {
        let min = Point3D { index: 0, x: -1.0, y: -1.0, z: -1.0 };
        let max = Point3D { index: 0, x: 1.0, y: 1.0, z: 1.0 };
        // Use depth=1 and loose ratio to keep test fast
        let result = octree_refined(min, max, 1, &|_| true, 2.0);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_voxel_refined() {
        let min = Point3D { index: 0, x: 0.0, y: 0.0, z: 0.0 };
        let max = Point3D { index: 0, x: 1.0, y: 1.0, z: 1.0 };
        // Use 2x2x2 and loose ratio to keep test fast
        let result = voxel_refined(min, max, 2, 2, 2, &|_| true, 2.0);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_refine_tetrahedra_empty() {
        let result = refine_tetrahedra(&[], 2.0);
        assert!(result.is_empty());
    }

    #[test]
    fn test_refine_tetrahedra_single() {
        let tet = Tetrahedron {
            a: Point3D { index: 0, x: 0.0, y: 0.0, z: 0.0 },
            b: Point3D { index: 1, x: 1.0, y: 0.0, z: 0.0 },
            c: Point3D { index: 2, x: 0.5, y: 1.0, z: 0.0 },
            d: Point3D { index: 3, x: 0.5, y: 0.5, z: 1.0 },
        };
        let result = refine_tetrahedra(&[tet], 2.0);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_octree_refined_empty_domain() {
        let min = Point3D { index: 0, x: 0.0, y: 0.0, z: 0.0 };
        let max = Point3D { index: 0, x: 1.0, y: 1.0, z: 1.0 };
        let result = octree_refined(min, max, 2, &|_| false, 2.0);
        assert!(result.is_empty());
    }

    #[test]
    fn test_extract_unique_points() {
        let tet = Tetrahedron {
            a: Point3D { index: 0, x: 0.0, y: 0.0, z: 0.0 },
            b: Point3D { index: 1, x: 1.0, y: 0.0, z: 0.0 },
            c: Point3D { index: 2, x: 0.0, y: 1.0, z: 0.0 },
            d: Point3D { index: 3, x: 0.0, y: 0.0, z: 1.0 },
        };
        let points = extract_unique_points(&[tet]);
        assert_eq!(points.len(), 4);
    }
}

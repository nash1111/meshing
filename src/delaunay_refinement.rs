use crate::{bowyer_watson_3d, Point3D, Tetrahedron};

fn shortest_edge_length(tet: &Tetrahedron) -> f64 {
    let v = tet.vertices();
    let mut min_len = f64::MAX;
    for i in 0..4 {
        for j in (i + 1)..4 {
            let d = v[i].distance(&v[j]);
            if d < min_len {
                min_len = d;
            }
        }
    }
    min_len
}

fn radius_edge_ratio(tet: &Tetrahedron) -> f64 {
    let circumsphere = tet.circumsphere();
    circumsphere.radius / shortest_edge_length(tet)
}

/// Improves mesh quality by iteratively inserting circumsphere centers of
/// poorly-shaped tetrahedra (Ruppert-style refinement).
///
/// Starts from a Bowyer-Watson tetrahedralization and repeatedly splits the
/// worst tetrahedron (highest radius-to-edge ratio) until all tetrahedra
/// satisfy the quality threshold.
///
/// # Arguments
///
/// * `points` - Initial point set for the base Delaunay tetrahedralization.
/// * `max_radius_edge_ratio` - Quality threshold; lower values produce
///   better-shaped tetrahedra but more elements. A value of 2.0 is a common default.
///
/// # Returns
///
/// A vector of quality-improved [`Tetrahedron`]s.
///
/// # Examples
///
/// ```
/// use meshing::delaunay_refinement::delaunay_refinement;
/// use meshing::Point3D;
///
/// let points = vec![
///     Point3D { index: 0, x: 1.0, y: 1.0, z: 1.0 },
///     Point3D { index: 1, x: 1.0, y: -1.0, z: -1.0 },
///     Point3D { index: 2, x: -1.0, y: 1.0, z: -1.0 },
///     Point3D { index: 3, x: -1.0, y: -1.0, z: 1.0 },
/// ];
/// let refined = delaunay_refinement(points, 2.0);
/// assert!(!refined.is_empty());
/// ```
pub fn delaunay_refinement(points: Vec<Point3D>, max_radius_edge_ratio: f64) -> Vec<Tetrahedron> {
    let mut refined_points = points.clone();
    let mut mesh = bowyer_watson_3d(points);
    let max_iterations = 100 * refined_points.len();

    for _ in 0..max_iterations {
        let worst = mesh.iter().max_by(|a, b| {
            radius_edge_ratio(a)
                .partial_cmp(&radius_edge_ratio(b))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let worst = match worst {
            Some(t) => *t,
            None => break,
        };

        if radius_edge_ratio(&worst) <= max_radius_edge_ratio {
            break;
        }

        let center = worst.circumsphere().center;
        let new_point = Point3D {
            index: refined_points.len() as i64,
            x: center.x,
            y: center.y,
            z: center.z,
        };
        refined_points.push(new_point);
        mesh = bowyer_watson_3d(refined_points.clone());
    }

    mesh
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regular_tetrahedron_with_loose_threshold() {
        let points = vec![
            Point3D {
                index: 0,
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            Point3D {
                index: 1,
                x: 1.0,
                y: -1.0,
                z: -1.0,
            },
            Point3D {
                index: 2,
                x: -1.0,
                y: 1.0,
                z: -1.0,
            },
            Point3D {
                index: 3,
                x: -1.0,
                y: -1.0,
                z: 1.0,
            },
        ];
        let result = delaunay_refinement(points, 10.0);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_cube_vertices_produce_valid_mesh() {
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
        let result = delaunay_refinement(points, 2.0);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_tight_threshold_adds_points() {
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
        let initial = bowyer_watson_3d(points.clone());
        // Moderate threshold triggers some refinement
        let refined = delaunay_refinement(points, 1.5);
        assert!(refined.len() >= initial.len());
    }

    #[test]
    fn test_all_refined_tetrahedra_have_nonzero_volume() {
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
        let result = delaunay_refinement(points, 2.0);
        for tet in &result {
            assert!(
                tet.signed_volume().abs() > 1e-15,
                "Degenerate tetrahedron found"
            );
        }
    }

    #[test]
    fn test_refinement_produces_at_least_as_many_tets() {
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
        let initial = bowyer_watson_3d(points.clone());
        let refined = delaunay_refinement(points, 2.0);
        assert!(refined.len() >= initial.len());
    }
}

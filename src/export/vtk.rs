use crate::{Point3D, Tetrahedron};

/// Exports a tetrahedral mesh to VTK Legacy unstructured grid format (.vtk).
///
/// Produces an ASCII VTK file compatible with ParaView and other visualization
/// tools. Tetrahedra use VTK cell type 10.
///
/// # Arguments
///
/// * `tetrahedra` - The tetrahedral mesh to export.
/// * `title` - A descriptive title written into the VTK header.
///
/// # Returns
///
/// A string containing the complete VTK file content.
///
/// # Examples
///
/// ```
/// use meshing::export::tetrahedra_to_vtk;
/// use meshing::{Point3D, Tetrahedron};
///
/// let tet = Tetrahedron {
///     a: Point3D { index: 0, x: 0.0, y: 0.0, z: 0.0 },
///     b: Point3D { index: 1, x: 1.0, y: 0.0, z: 0.0 },
///     c: Point3D { index: 2, x: 0.0, y: 1.0, z: 0.0 },
///     d: Point3D { index: 3, x: 0.0, y: 0.0, z: 1.0 },
/// };
/// let vtk = tetrahedra_to_vtk(&[tet], "test");
/// assert!(vtk.contains("CELL_TYPES"));
/// ```
pub fn tetrahedra_to_vtk(tetrahedra: &[Tetrahedron], title: &str) -> String {
    // Collect unique vertices
    let mut vertices: Vec<(i64, Point3D)> = Vec::new();
    for tet in tetrahedra {
        for v in tet.vertices() {
            if !vertices.iter().any(|(idx, _)| *idx == v.index) {
                vertices.push((v.index, v));
            }
        }
    }
    vertices.sort_by_key(|(idx, _)| *idx);

    let num_points = vertices.len();
    let num_cells = tetrahedra.len();

    let mut result = String::new();

    // VTK header
    result.push_str("# vtk DataFile Version 3.0\n");
    result.push_str(title);
    result.push('\n');
    result.push_str("ASCII\n");
    result.push_str("DATASET UNSTRUCTURED_GRID\n");

    // Points
    result.push_str(&format!("POINTS {} double\n", num_points));
    for (_, v) in &vertices {
        result.push_str(&format!("{} {} {}\n", v.x, v.y, v.z));
    }

    // Cells: each tetrahedron has 4 vertices, so cell size entry = 5 (count + 4 indices)
    let cell_list_size = num_cells * 5;
    result.push_str(&format!("CELLS {} {}\n", num_cells, cell_list_size));
    for tet in tetrahedra {
        let a = vertices.iter().position(|(idx, _)| *idx == tet.a.index).unwrap();
        let b = vertices.iter().position(|(idx, _)| *idx == tet.b.index).unwrap();
        let c = vertices.iter().position(|(idx, _)| *idx == tet.c.index).unwrap();
        let d = vertices.iter().position(|(idx, _)| *idx == tet.d.index).unwrap();
        result.push_str(&format!("4 {} {} {} {}\n", a, b, c, d));
    }

    // Cell types: 10 = VTK_TETRA
    result.push_str(&format!("CELL_TYPES {}\n", num_cells));
    for _ in 0..num_cells {
        result.push_str("10\n");
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn single_tet() -> Tetrahedron {
        Tetrahedron {
            a: Point3D { index: 0, x: 0.0, y: 0.0, z: 0.0 },
            b: Point3D { index: 1, x: 1.0, y: 0.0, z: 0.0 },
            c: Point3D { index: 2, x: 0.0, y: 1.0, z: 0.0 },
            d: Point3D { index: 3, x: 0.0, y: 0.0, z: 1.0 },
        }
    }

    #[test]
    fn test_vtk_header() {
        let result = tetrahedra_to_vtk(&[single_tet()], "test mesh");
        assert!(result.starts_with("# vtk DataFile Version 3.0\n"));
        assert!(result.contains("test mesh"));
        assert!(result.contains("ASCII"));
        assert!(result.contains("DATASET UNSTRUCTURED_GRID"));
    }

    #[test]
    fn test_vtk_single_tet() {
        let result = tetrahedra_to_vtk(&[single_tet()], "single");
        assert!(result.contains("POINTS 4 double"));
        assert!(result.contains("CELLS 1 5"));
        assert!(result.contains("4 0 1 2 3"));
        assert!(result.contains("CELL_TYPES 1"));
        assert!(result.contains("10"));
    }

    #[test]
    fn test_vtk_empty() {
        let result = tetrahedra_to_vtk(&[], "empty");
        assert!(result.contains("POINTS 0 double"));
        assert!(result.contains("CELLS 0 0"));
        assert!(result.contains("CELL_TYPES 0"));
    }

    #[test]
    fn test_vtk_shared_vertices() {
        let p0 = Point3D { index: 0, x: 0.0, y: 0.0, z: 0.0 };
        let p1 = Point3D { index: 1, x: 1.0, y: 0.0, z: 0.0 };
        let p2 = Point3D { index: 2, x: 0.0, y: 1.0, z: 0.0 };
        let p3 = Point3D { index: 3, x: 0.0, y: 0.0, z: 1.0 };
        let p4 = Point3D { index: 4, x: 1.0, y: 1.0, z: 1.0 };

        let tet1 = Tetrahedron { a: p0, b: p1, c: p2, d: p3 };
        let tet2 = Tetrahedron { a: p1, b: p2, c: p3, d: p4 };
        let result = tetrahedra_to_vtk(&[tet1, tet2], "shared");
        assert!(result.contains("POINTS 5 double"));
        assert!(result.contains("CELLS 2 10"));
        assert!(result.contains("CELL_TYPES 2"));
    }

    #[test]
    fn test_vtk_coordinates() {
        let tet = Tetrahedron {
            a: Point3D { index: 0, x: 1.5, y: 2.5, z: 3.5 },
            b: Point3D { index: 1, x: 4.0, y: 5.0, z: 6.0 },
            c: Point3D { index: 2, x: 7.0, y: 8.0, z: 9.0 },
            d: Point3D { index: 3, x: 0.0, y: 0.0, z: 0.0 },
        };
        let result = tetrahedra_to_vtk(&[tet], "coords");
        assert!(result.contains("1.5 2.5 3.5"));
        assert!(result.contains("4 5 6"));
        assert!(result.contains("7 8 9"));
        assert!(result.contains("0 0 0"));
    }
}

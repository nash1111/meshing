use js_sys::Function;
use wasm_bindgen::prelude::*;

use crate::advancing_front::advancing_front;
use crate::delaunay_refinement::delaunay_refinement;
use crate::marching_cubes::marching_cubes;
use crate::octree::octree_mesh;
use crate::voxel_mesh::voxel_mesh;
use crate::{bowyer_watson, Face, Point2D, Point3D};

fn coords_to_points_3d(coords: &[f64]) -> Vec<Point3D> {
    coords
        .chunks(3)
        .enumerate()
        .map(|(i, c)| Point3D {
            index: i as i64,
            x: c[0],
            y: c[1],
            z: c[2],
        })
        .collect()
}

fn tet_indices(tets: &[crate::Tetrahedron]) -> Vec<[usize; 4]> {
    tets.iter()
        .map(|t| {
            [
                t.a.index as usize,
                t.b.index as usize,
                t.c.index as usize,
                t.d.index as usize,
            ]
        })
        .collect()
}

#[wasm_bindgen]
pub fn triangulate(coords: &[f64]) -> Result<JsValue, JsError> {
    if coords.len() % 2 != 0 {
        return Err(JsError::new(
            "coords must have an even number of elements (x1,y1,x2,y2,...)",
        ));
    }

    let points: Vec<Point2D> = coords
        .chunks(2)
        .enumerate()
        .map(|(i, chunk)| Point2D {
            x: chunk[0],
            y: chunk[1],
            index: i as i64,
        })
        .collect();

    let triangles = bowyer_watson(points);

    let result: Vec<[usize; 3]> = triangles
        .iter()
        .map(|t| [t.a.index as usize, t.b.index as usize, t.c.index as usize])
        .collect();

    serde_wasm_bindgen::to_value(&result).map_err(|e| JsError::new(&e.to_string()))
}

/// 3D Delaunay tetrahedralization via Bowyer-Watson.
///
/// `coords` is a flat array [x1,y1,z1, x2,y2,z2, ...].
/// Returns tetrahedra as [[a,b,c,d], ...] (vertex indices).
#[wasm_bindgen]
pub fn triangulate_3d(coords: &[f64]) -> Result<JsValue, JsError> {
    if coords.len() % 3 != 0 {
        return Err(JsError::new(
            "coords length must be a multiple of 3 (x1,y1,z1,x2,y2,z2,...)",
        ));
    }

    let points = coords_to_points_3d(coords);
    let tets = crate::bowyer_watson_3d(points);
    let result = tet_indices(&tets);

    serde_wasm_bindgen::to_value(&result).map_err(|e| JsError::new(&e.to_string()))
}

/// Advancing front tetrahedral meshing from a boundary surface.
///
/// `face_indices` is a flat array of triangle indices [a1,b1,c1, a2,b2,c2, ...].
/// `coords` is a flat array of vertex positions [x1,y1,z1, x2,y2,z2, ...].
/// Returns tetrahedra as [[a,b,c,d], ...].
#[wasm_bindgen]
pub fn advancing_front_mesh(face_indices: &[u32], coords: &[f64]) -> Result<JsValue, JsError> {
    if coords.len() % 3 != 0 {
        return Err(JsError::new(
            "coords length must be a multiple of 3 (x1,y1,z1,...)",
        ));
    }
    if face_indices.len() % 3 != 0 {
        return Err(JsError::new(
            "face_indices length must be a multiple of 3 (a1,b1,c1,...)",
        ));
    }

    let points = coords_to_points_3d(coords);

    let faces: Vec<Face> = face_indices
        .chunks(3)
        .map(|f| Face {
            a: points[f[0] as usize],
            b: points[f[1] as usize],
            c: points[f[2] as usize],
        })
        .collect();

    let tets = advancing_front(faces, points);
    let result = tet_indices(&tets);

    serde_wasm_bindgen::to_value(&result).map_err(|e| JsError::new(&e.to_string()))
}

/// Octree-based tetrahedral mesh generation.
///
/// `is_inside_fn` is a JS function(x, y, z) that returns true if the point is inside.
#[wasm_bindgen]
pub fn octree_mesh_generate(
    min_x: f64,
    min_y: f64,
    min_z: f64,
    max_x: f64,
    max_y: f64,
    max_z: f64,
    depth: usize,
    is_inside_fn: &Function,
) -> Result<JsValue, JsError> {
    let min = Point3D {
        index: 0,
        x: min_x,
        y: min_y,
        z: min_z,
    };
    let max = Point3D {
        index: 0,
        x: max_x,
        y: max_y,
        z: max_z,
    };

    let func = is_inside_fn.clone();
    let is_inside = move |p: &Point3D| -> bool {
        let this = JsValue::null();
        func.call3(
            &this,
            &JsValue::from(p.x),
            &JsValue::from(p.y),
            &JsValue::from(p.z),
        )
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    };

    let tets = octree_mesh(min, max, depth, &is_inside);
    let result = tet_indices(&tets);

    serde_wasm_bindgen::to_value(&result).map_err(|e| JsError::new(&e.to_string()))
}

/// Marching cubes isosurface extraction.
///
/// `scalar_field_fn` is a JS function(x, y, z) that returns a number.
/// Returns triangles as [[a_idx, b_idx, c_idx], ...] with vertices interleaved.
/// The result is an object { vertices: [x1,y1,z1,...], triangles: [[0,1,2], ...] }.
#[wasm_bindgen]
pub fn marching_cubes_generate(
    nx: usize,
    ny: usize,
    nz: usize,
    min_x: f64,
    min_y: f64,
    min_z: f64,
    max_x: f64,
    max_y: f64,
    max_z: f64,
    scalar_field_fn: &Function,
    iso_value: f64,
) -> Result<JsValue, JsError> {
    let min = Point3D {
        index: 0,
        x: min_x,
        y: min_y,
        z: min_z,
    };
    let max = Point3D {
        index: 0,
        x: max_x,
        y: max_y,
        z: max_z,
    };

    let func = scalar_field_fn.clone();
    let field = move |x: f64, y: f64, z: f64| -> f64 {
        let this = JsValue::null();
        func.call3(
            &this,
            &JsValue::from(x),
            &JsValue::from(y),
            &JsValue::from(z),
        )
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0)
    };

    let faces = marching_cubes(nx, ny, nz, min, max, &field, iso_value);

    // Collect unique vertices and build indexed triangle list
    let mut vertices: Vec<f64> = Vec::new();
    let mut tri_indices: Vec<[usize; 3]> = Vec::new();
    let mut vertex_map: Vec<(f64, f64, f64)> = Vec::new();

    for face in &faces {
        let mut idx = [0usize; 3];
        for (i, pt) in [face.a, face.b, face.c].iter().enumerate() {
            let pos = vertex_map.iter().position(|&(x, y, z)| {
                (x - pt.x).abs() < 1e-12 && (y - pt.y).abs() < 1e-12 && (z - pt.z).abs() < 1e-12
            });
            idx[i] = match pos {
                Some(existing) => existing,
                None => {
                    let new_idx = vertex_map.len();
                    vertex_map.push((pt.x, pt.y, pt.z));
                    vertices.push(pt.x);
                    vertices.push(pt.y);
                    vertices.push(pt.z);
                    new_idx
                }
            };
        }
        tri_indices.push(idx);
    }

    let result = serde::Serialize::serialize(
        &(vertices, tri_indices),
        serde_wasm_bindgen::Serializer::json_compatible(),
    )
    .map_err(|e| JsError::new(&e.to_string()))?;

    Ok(result)
}

/// Voxel-based tetrahedral mesh generation.
///
/// `is_inside_fn` is a JS function(x, y, z) that returns true if the point is inside.
#[wasm_bindgen]
pub fn voxel_mesh_generate(
    min_x: f64,
    min_y: f64,
    min_z: f64,
    max_x: f64,
    max_y: f64,
    max_z: f64,
    nx: usize,
    ny: usize,
    nz: usize,
    is_inside_fn: &Function,
) -> Result<JsValue, JsError> {
    let min = Point3D {
        index: 0,
        x: min_x,
        y: min_y,
        z: min_z,
    };
    let max = Point3D {
        index: 0,
        x: max_x,
        y: max_y,
        z: max_z,
    };

    let func = is_inside_fn.clone();
    let is_inside = move |p: &Point3D| -> bool {
        let this = JsValue::null();
        func.call3(
            &this,
            &JsValue::from(p.x),
            &JsValue::from(p.y),
            &JsValue::from(p.z),
        )
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    };

    let tets = voxel_mesh(min, max, nx, ny, nz, &is_inside);
    let result = tet_indices(&tets);

    serde_wasm_bindgen::to_value(&result).map_err(|e| JsError::new(&e.to_string()))
}

/// Delaunay refinement (Ruppert's algorithm) for mesh quality improvement.
///
/// `coords` is a flat array [x1,y1,z1, x2,y2,z2, ...].
/// `max_radius_edge_ratio` controls quality threshold (lower = better quality).
/// Returns tetrahedra as [[a,b,c,d], ...].
#[wasm_bindgen]
pub fn delaunay_refinement_mesh(
    coords: &[f64],
    max_radius_edge_ratio: f64,
) -> Result<JsValue, JsError> {
    if coords.len() % 3 != 0 {
        return Err(JsError::new(
            "coords length must be a multiple of 3 (x1,y1,z1,...)",
        ));
    }

    let points = coords_to_points_3d(coords);
    let tets = delaunay_refinement(points, max_radius_edge_ratio);
    let result = tet_indices(&tets);

    serde_wasm_bindgen::to_value(&result).map_err(|e| JsError::new(&e.to_string()))
}

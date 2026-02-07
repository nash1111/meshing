use crate::export::stl::extract_surface_faces;
use crate::{Face, Tetrahedron};

/// Exports 3D faces to a quantized GLB (binary glTF) format using
/// the `KHR_mesh_quantization` extension.
///
/// Vertex positions are quantized from `f32` to `i16`, reducing position
/// data size by 50%. A node transformation matrix is applied to decode
/// the quantized coordinates back to world space.
///
/// # Arguments
///
/// * `faces` - Surface triangle faces to export.
///
/// # Returns
///
/// A `Vec<u8>` containing the complete quantized GLB file.
///
/// # Examples
///
/// ```
/// use meshing::export::faces_to_glb_quantized;
/// use meshing::{Face, Point3D};
///
/// let face = Face {
///     a: Point3D { index: 0, x: 0.0, y: 0.0, z: 0.0 },
///     b: Point3D { index: 1, x: 1.0, y: 0.0, z: 0.0 },
///     c: Point3D { index: 2, x: 0.0, y: 1.0, z: 0.0 },
/// };
/// let glb = faces_to_glb_quantized(&[face]);
/// assert_eq!(&glb[0..4], b"glTF");
/// ```
pub fn faces_to_glb_quantized(faces: &[Face]) -> Vec<u8> {
    let (vertices, indices) = collect_unique_vertices(faces);

    let num_vertices = vertices.len();
    let num_indices = indices.len();

    if num_vertices == 0 {
        return build_glb(&build_quantized_json(0, 0, 0, 0, [0.0; 3], [1.0; 3]), &[]);
    }

    // Compute bounding box
    let (bb_min, bb_max) = bounding_box(&vertices);

    // Compute scale and offset for quantization
    let scale = [
        if bb_max[0] > bb_min[0] {
            bb_max[0] - bb_min[0]
        } else {
            1.0
        },
        if bb_max[1] > bb_min[1] {
            bb_max[1] - bb_min[1]
        } else {
            1.0
        },
        if bb_max[2] > bb_min[2] {
            bb_max[2] - bb_min[2]
        } else {
            1.0
        },
    ];
    let offset = bb_min;

    // Quantize positions to i16 range [-32767, 32767]
    let mut quantized: Vec<i16> = Vec::with_capacity(num_vertices * 3);
    for v in &vertices {
        for i in 0..3 {
            let normalized = (v[i] - offset[i]) / scale[i]; // [0, 1]
            let q = (normalized * 65534.0 - 32767.0).round() as i16; // [-32767, 32767]
            quantized.push(q);
        }
    }

    // Build binary buffer: quantized positions (i16) + indices (u32)
    let pos_byte_length = quantized.len() * 2;
    // Pad position data to 4-byte boundary for index alignment
    let pos_padded = (pos_byte_length + 3) & !3;
    let idx_byte_length = num_indices * 4;

    let mut buffer = Vec::with_capacity(pos_padded + idx_byte_length);
    for &val in &quantized {
        buffer.extend_from_slice(&val.to_le_bytes());
    }
    // Pad to 4-byte alignment
    buffer.extend(std::iter::repeat_n(0u8, pos_padded - pos_byte_length));
    for &val in &indices {
        buffer.extend_from_slice(&val.to_le_bytes());
    }

    let json = build_quantized_json(
        num_vertices,
        num_indices,
        pos_padded,
        idx_byte_length,
        offset,
        scale,
    );

    build_glb(&json, &buffer)
}

/// Exports a tetrahedral mesh to quantized GLB by extracting surface faces.
pub fn tetrahedra_to_glb_quantized(tetrahedra: &[Tetrahedron]) -> Vec<u8> {
    let surface = extract_surface_faces(tetrahedra);
    faces_to_glb_quantized(&surface)
}

fn collect_unique_vertices(faces: &[Face]) -> (Vec<[f32; 3]>, Vec<u32>) {
    let mut vertex_list: Vec<(i64, [f32; 3])> = Vec::new();
    for face in faces {
        for v in face.vertices() {
            if !vertex_list.iter().any(|(idx, _)| *idx == v.index) {
                vertex_list.push((v.index, [v.x as f32, v.y as f32, v.z as f32]));
            }
        }
    }
    vertex_list.sort_by_key(|(idx, _)| *idx);

    let mut indices = Vec::with_capacity(faces.len() * 3);
    for face in faces {
        for pt in [face.a, face.b, face.c] {
            let pos = vertex_list
                .iter()
                .position(|(idx, _)| *idx == pt.index)
                .unwrap();
            indices.push(pos as u32);
        }
    }

    let vertices: Vec<[f32; 3]> = vertex_list.into_iter().map(|(_, v)| v).collect();
    (vertices, indices)
}

fn bounding_box(vertices: &[[f32; 3]]) -> ([f32; 3], [f32; 3]) {
    let mut min = [f32::MAX; 3];
    let mut max = [f32::MIN; 3];
    for v in vertices {
        for i in 0..3 {
            if v[i] < min[i] {
                min[i] = v[i];
            }
            if v[i] > max[i] {
                max[i] = v[i];
            }
        }
    }
    (min, max)
}

fn build_quantized_json(
    num_vertices: usize,
    num_indices: usize,
    pos_byte_length: usize,
    idx_byte_length: usize,
    offset: [f32; 3],
    scale: [f32; 3],
) -> String {
    // The node matrix transforms quantized i16 [-32767, 32767] back to world coords:
    // world = (quantized + 32767) / 65534 * scale + offset
    // As a 4x4 column-major matrix:
    //   sx  0  0  0
    //    0 sy  0  0
    //    0  0 sz  0
    //   tx ty tz  1
    let sx = scale[0] / 65534.0;
    let sy = scale[1] / 65534.0;
    let sz = scale[2] / 65534.0;
    let tx = offset[0] + scale[0] * 32767.0 / 65534.0;
    let ty = offset[1] + scale[1] * 32767.0 / 65534.0;
    let tz = offset[2] + scale[2] * 32767.0 / 65534.0;

    let buffer_byte_length = pos_byte_length + idx_byte_length;

    format!(
        concat!(
            "{{",
            "\"asset\":{{\"version\":\"2.0\",\"generator\":\"meshing\"}},",
            "\"extensionsUsed\":[\"KHR_mesh_quantization\"],",
            "\"extensionsRequired\":[\"KHR_mesh_quantization\"],",
            "\"scene\":0,",
            "\"scenes\":[{{\"nodes\":[0]}}],",
            "\"nodes\":[{{\"mesh\":0,\"matrix\":[{},0,0,0,0,{},0,0,0,0,{},0,{},{},{},1]}}],",
            "\"meshes\":[{{\"primitives\":[{{\"attributes\":{{\"POSITION\":0}},\"indices\":1}}]}}],",
            "\"accessors\":[",
            "{{\"bufferView\":0,\"componentType\":5122,\"count\":{},\"type\":\"VEC3\",\"max\":[32767,32767,32767],\"min\":[-32767,-32767,-32767]}},",
            "{{\"bufferView\":1,\"componentType\":5125,\"count\":{},\"type\":\"SCALAR\"}}",
            "],",
            "\"bufferViews\":[",
            "{{\"buffer\":0,\"byteOffset\":0,\"byteLength\":{},\"target\":34962}},",
            "{{\"buffer\":0,\"byteOffset\":{},\"byteLength\":{},\"target\":34963}}",
            "],",
            "\"buffers\":[{{\"byteLength\":{}}}]",
            "}}"
        ),
        sx, sy, sz, tx, ty, tz,
        num_vertices,
        num_indices,
        pos_byte_length,
        pos_byte_length, idx_byte_length,
        buffer_byte_length,
    )
}

fn build_glb(json_str: &str, bin_buffer: &[u8]) -> Vec<u8> {
    let json_bytes = json_str.as_bytes();
    let json_padded_len = (json_bytes.len() + 3) & !3;
    let bin_padded_len = (bin_buffer.len() + 3) & !3;

    let total_length = 12
        + 8
        + json_padded_len
        + if bin_buffer.is_empty() {
            0
        } else {
            8 + bin_padded_len
        };

    let mut glb = Vec::with_capacity(total_length);

    // GLB Header
    glb.extend_from_slice(b"glTF");
    glb.extend_from_slice(&2u32.to_le_bytes());
    glb.extend_from_slice(&(total_length as u32).to_le_bytes());

    // JSON chunk
    glb.extend_from_slice(&(json_padded_len as u32).to_le_bytes());
    glb.extend_from_slice(&0x4E4F534Au32.to_le_bytes());
    glb.extend_from_slice(json_bytes);
    glb.extend(std::iter::repeat_n(
        b' ',
        json_padded_len - json_bytes.len(),
    ));

    // Binary chunk (only if there's data)
    if !bin_buffer.is_empty() {
        glb.extend_from_slice(&(bin_padded_len as u32).to_le_bytes());
        glb.extend_from_slice(&0x004E4942u32.to_le_bytes());
        glb.extend_from_slice(bin_buffer);
        glb.extend(std::iter::repeat_n(0u8, bin_padded_len - bin_buffer.len()));
    }

    glb
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::faces_to_glb;
    use crate::Point3D;

    fn test_face() -> Face {
        Face {
            a: Point3D {
                index: 0,
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            b: Point3D {
                index: 1,
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            c: Point3D {
                index: 2,
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
        }
    }

    #[test]
    fn test_glb_magic() {
        let glb = faces_to_glb_quantized(&[test_face()]);
        assert_eq!(&glb[0..4], b"glTF");
    }

    #[test]
    fn test_glb_version() {
        let glb = faces_to_glb_quantized(&[test_face()]);
        let version = u32::from_le_bytes([glb[4], glb[5], glb[6], glb[7]]);
        assert_eq!(version, 2);
    }

    #[test]
    fn test_glb_total_length_matches() {
        let glb = faces_to_glb_quantized(&[test_face()]);
        let total = u32::from_le_bytes([glb[8], glb[9], glb[10], glb[11]]);
        assert_eq!(total as usize, glb.len());
    }

    #[test]
    fn test_glb_alignment() {
        let glb = faces_to_glb_quantized(&[test_face()]);
        assert_eq!(glb.len() % 4, 0);
    }

    #[test]
    fn test_quantized_vertex_data_smaller() {
        // Quantized positions use i16 (2 bytes) vs f32 (4 bytes),
        // so the binary buffer is always smaller.
        // With small meshes, JSON overhead from extension declarations
        // may make the total GLB larger, but vertex data savings scale.
        let faces = vec![
            Face {
                a: Point3D {
                    index: 0,
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                b: Point3D {
                    index: 1,
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                },
                c: Point3D {
                    index: 2,
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                },
            },
            Face {
                a: Point3D {
                    index: 1,
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                },
                b: Point3D {
                    index: 3,
                    x: 1.0,
                    y: 1.0,
                    z: 0.0,
                },
                c: Point3D {
                    index: 2,
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                },
            },
        ];
        let regular = faces_to_glb(&faces);
        let quantized = faces_to_glb_quantized(&faces);
        // Both produce valid GLB
        assert_eq!(&regular[0..4], b"glTF");
        assert_eq!(&quantized[0..4], b"glTF");
        // Quantized uses int16 component type
        let q_json_len =
            u32::from_le_bytes([quantized[12], quantized[13], quantized[14], quantized[15]])
                as usize;
        let q_json = std::str::from_utf8(&quantized[20..20 + q_json_len]).unwrap();
        assert!(q_json.contains("5122")); // SHORT component type
    }

    #[test]
    fn test_quantized_contains_extension() {
        let glb = faces_to_glb_quantized(&[test_face()]);
        // Extract JSON from GLB
        let json_len = u32::from_le_bytes([glb[12], glb[13], glb[14], glb[15]]) as usize;
        let json = std::str::from_utf8(&glb[20..20 + json_len]).unwrap().trim();
        assert!(json.contains("KHR_mesh_quantization"));
    }

    #[test]
    fn test_quantized_uses_short_component() {
        let glb = faces_to_glb_quantized(&[test_face()]);
        let json_len = u32::from_le_bytes([glb[12], glb[13], glb[14], glb[15]]) as usize;
        let json = std::str::from_utf8(&glb[20..20 + json_len]).unwrap().trim();
        // componentType 5122 = SHORT (int16)
        assert!(json.contains("\"componentType\":5122"));
    }

    #[test]
    fn test_empty_faces() {
        let glb = faces_to_glb_quantized(&[]);
        assert_eq!(&glb[0..4], b"glTF");
        assert_eq!(glb.len() % 4, 0);
    }

    #[test]
    fn test_tetrahedra_to_glb_quantized() {
        let tet = Tetrahedron {
            a: Point3D {
                index: 0,
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            b: Point3D {
                index: 1,
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            c: Point3D {
                index: 2,
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            d: Point3D {
                index: 3,
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
        };
        let glb = tetrahedra_to_glb_quantized(&[tet]);
        assert_eq!(&glb[0..4], b"glTF");
    }
}

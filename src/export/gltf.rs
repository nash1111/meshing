use crate::export::stl::extract_surface_faces;
use crate::{Face, Point3D, Tetrahedron};

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

struct MeshData {
    positions: Vec<f32>,
    indices: Vec<u32>,
    min: [f32; 3],
    max: [f32; 3],
}

fn collect_mesh_data(faces: &[Face]) -> MeshData {
    let mut vertices: Vec<(i64, Point3D)> = Vec::new();
    for face in faces {
        for v in face.vertices() {
            if !vertices.iter().any(|(idx, _)| *idx == v.index) {
                vertices.push((v.index, v));
            }
        }
    }
    vertices.sort_by_key(|(idx, _)| *idx);

    let mut min = [f32::MAX; 3];
    let mut max = [f32::MIN; 3];
    let mut positions = Vec::with_capacity(vertices.len() * 3);
    for (_, v) in &vertices {
        let coords = [v.x as f32, v.y as f32, v.z as f32];
        for i in 0..3 {
            if coords[i] < min[i] {
                min[i] = coords[i];
            }
            if coords[i] > max[i] {
                max[i] = coords[i];
            }
        }
        positions.extend_from_slice(&coords);
    }

    let mut indices = Vec::with_capacity(faces.len() * 3);
    for face in faces {
        let a = vertices
            .iter()
            .position(|(idx, _)| *idx == face.a.index)
            .unwrap() as u32;
        let b = vertices
            .iter()
            .position(|(idx, _)| *idx == face.b.index)
            .unwrap() as u32;
        let c = vertices
            .iter()
            .position(|(idx, _)| *idx == face.c.index)
            .unwrap() as u32;
        indices.push(a);
        indices.push(b);
        indices.push(c);
    }

    if vertices.is_empty() {
        min = [0.0; 3];
        max = [0.0; 3];
    }

    MeshData {
        positions,
        indices,
        min,
        max,
    }
}

fn build_binary_buffer(data: &MeshData) -> Vec<u8> {
    let pos_bytes = data.positions.len() * 4;
    let idx_bytes = data.indices.len() * 4;
    let mut buffer = Vec::with_capacity(pos_bytes + idx_bytes);

    for &val in &data.positions {
        buffer.extend_from_slice(&val.to_le_bytes());
    }
    for &val in &data.indices {
        buffer.extend_from_slice(&val.to_le_bytes());
    }

    buffer
}

fn build_json(data: &MeshData, buffer_uri: Option<&str>, buffer_byte_length: usize) -> String {
    let num_vertices = data.positions.len() / 3;
    let num_indices = data.indices.len();
    let pos_byte_length = num_vertices * 12;
    let idx_byte_length = num_indices * 4;

    let buffer_line = match buffer_uri {
        Some(uri) => format!(
            "{{\"uri\":\"{}\",\"byteLength\":{}}}",
            uri, buffer_byte_length
        ),
        None => format!("{{\"byteLength\":{}}}", buffer_byte_length),
    };

    format!(
        concat!(
            "{{",
            "\"asset\":{{\"version\":\"2.0\",\"generator\":\"meshing\"}},",
            "\"scene\":0,",
            "\"scenes\":[{{\"nodes\":[0]}}],",
            "\"nodes\":[{{\"mesh\":0}}],",
            "\"meshes\":[{{\"primitives\":[{{\"attributes\":{{\"POSITION\":0}},\"indices\":1}}]}}],",
            "\"accessors\":[",
            "{{\"bufferView\":0,\"componentType\":5126,\"count\":{},\"type\":\"VEC3\",\"min\":[{},{},{}],\"max\":[{},{},{}]}},",
            "{{\"bufferView\":1,\"componentType\":5125,\"count\":{},\"type\":\"SCALAR\"}}",
            "],",
            "\"bufferViews\":[",
            "{{\"buffer\":0,\"byteOffset\":0,\"byteLength\":{},\"target\":34962}},",
            "{{\"buffer\":0,\"byteOffset\":{},\"byteLength\":{},\"target\":34963}}",
            "],",
            "\"buffers\":[{}]",
            "}}"
        ),
        num_vertices,
        data.min[0], data.min[1], data.min[2],
        data.max[0], data.max[1], data.max[2],
        num_indices,
        pos_byte_length,
        pos_byte_length, idx_byte_length,
        buffer_line
    )
}

/// Exports 3D faces to glTF 2.0 JSON format with embedded base64 binary data.
///
/// Returns a complete `.gltf` JSON string that can be written directly to a file.
/// The binary buffer is embedded as a base64 data URI.
///
/// # Examples
///
/// ```
/// use meshing::export::faces_to_gltf;
/// use meshing::{Face, Point3D};
///
/// let face = Face {
///     a: Point3D { index: 0, x: 0.0, y: 0.0, z: 0.0 },
///     b: Point3D { index: 1, x: 1.0, y: 0.0, z: 0.0 },
///     c: Point3D { index: 2, x: 0.0, y: 1.0, z: 0.0 },
/// };
/// let json = faces_to_gltf(&[face]);
/// assert!(json.contains("\"version\":\"2.0\""));
/// ```
pub fn faces_to_gltf(faces: &[Face]) -> String {
    let data = collect_mesh_data(faces);
    let buffer = build_binary_buffer(&data);
    let b64 = base64_encode(&buffer);
    let uri = format!("data:application/octet-stream;base64,{}", b64);
    build_json(&data, Some(&uri), buffer.len())
}

/// Exports 3D faces to GLB (binary glTF) format.
///
/// Returns the complete `.glb` file content as bytes. This is a self-contained
/// binary format that includes both JSON metadata and binary vertex/index data.
///
/// # Examples
///
/// ```
/// use meshing::export::faces_to_glb;
/// use meshing::{Face, Point3D};
///
/// let face = Face {
///     a: Point3D { index: 0, x: 0.0, y: 0.0, z: 0.0 },
///     b: Point3D { index: 1, x: 1.0, y: 0.0, z: 0.0 },
///     c: Point3D { index: 2, x: 0.0, y: 1.0, z: 0.0 },
/// };
/// let glb = faces_to_glb(&[face]);
/// // GLB magic number
/// assert_eq!(&glb[0..4], b"glTF");
/// ```
pub fn faces_to_glb(faces: &[Face]) -> Vec<u8> {
    let data = collect_mesh_data(faces);
    let bin_buffer = build_binary_buffer(&data);
    let json_str = build_json(&data, None, bin_buffer.len());

    // Pad JSON to 4-byte alignment
    let json_bytes = json_str.as_bytes();
    let json_padded_len = (json_bytes.len() + 3) & !3;

    // Pad binary to 4-byte alignment
    let bin_padded_len = (bin_buffer.len() + 3) & !3;

    let total_length = 12 + 8 + json_padded_len + 8 + bin_padded_len;

    let mut glb = Vec::with_capacity(total_length);

    // GLB Header
    glb.extend_from_slice(b"glTF"); // magic
    glb.extend_from_slice(&2u32.to_le_bytes()); // version
    glb.extend_from_slice(&(total_length as u32).to_le_bytes()); // total length

    // JSON chunk
    glb.extend_from_slice(&(json_padded_len as u32).to_le_bytes()); // chunk length
    glb.extend_from_slice(&0x4E4F534Au32.to_le_bytes()); // chunk type "JSON"
    glb.extend_from_slice(json_bytes);
    glb.extend(std::iter::repeat_n(
        b' ',
        json_padded_len - json_bytes.len(),
    ));

    // Binary chunk
    glb.extend_from_slice(&(bin_padded_len as u32).to_le_bytes()); // chunk length
    glb.extend_from_slice(&0x004E4942u32.to_le_bytes()); // chunk type "BIN\0"
    glb.extend_from_slice(&bin_buffer);
    glb.extend(std::iter::repeat_n(0u8, bin_padded_len - bin_buffer.len()));

    glb
}

/// Exports a tetrahedral mesh to glTF 2.0 JSON by extracting surface faces.
pub fn tetrahedra_to_gltf(tetrahedra: &[Tetrahedron]) -> String {
    let surface = extract_surface_faces(tetrahedra);
    faces_to_gltf(&surface)
}

/// Exports a tetrahedral mesh to GLB by extracting surface faces.
pub fn tetrahedra_to_glb(tetrahedra: &[Tetrahedron]) -> Vec<u8> {
    let surface = extract_surface_faces(tetrahedra);
    faces_to_glb(&surface)
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_base64_encode() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn test_gltf_json_structure() {
        let json = faces_to_gltf(&[test_face()]);
        assert!(json.contains("\"version\":\"2.0\""));
        assert!(json.contains("\"generator\":\"meshing\""));
        assert!(json.contains("\"POSITION\":0"));
        assert!(json.contains("\"indices\":1"));
        assert!(json.contains("data:application/octet-stream;base64,"));
    }

    #[test]
    fn test_gltf_empty() {
        let json = faces_to_gltf(&[]);
        assert!(json.contains("\"count\":0"));
    }

    #[test]
    fn test_glb_magic() {
        let glb = faces_to_glb(&[test_face()]);
        assert_eq!(&glb[0..4], b"glTF");
    }

    #[test]
    fn test_glb_version() {
        let glb = faces_to_glb(&[test_face()]);
        let version = u32::from_le_bytes([glb[4], glb[5], glb[6], glb[7]]);
        assert_eq!(version, 2);
    }

    #[test]
    fn test_glb_total_length() {
        let glb = faces_to_glb(&[test_face()]);
        let total = u32::from_le_bytes([glb[8], glb[9], glb[10], glb[11]]);
        assert_eq!(total as usize, glb.len());
    }

    #[test]
    fn test_glb_json_chunk_type() {
        let glb = faces_to_glb(&[test_face()]);
        // After 12-byte header + 4-byte chunk length, chunk type at offset 16
        let chunk_type = u32::from_le_bytes([glb[16], glb[17], glb[18], glb[19]]);
        assert_eq!(chunk_type, 0x4E4F534A); // "JSON"
    }

    #[test]
    fn test_glb_alignment() {
        let glb = faces_to_glb(&[test_face()]);
        // Total length should be 4-byte aligned
        assert_eq!(glb.len() % 4, 0);
    }

    #[test]
    fn test_tetrahedra_to_gltf() {
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
        let json = tetrahedra_to_gltf(&[tet]);
        assert!(json.contains("\"version\":\"2.0\""));
    }

    #[test]
    fn test_tetrahedra_to_glb() {
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
        let glb = tetrahedra_to_glb(&[tet]);
        assert_eq!(&glb[0..4], b"glTF");
        assert_eq!(glb.len() % 4, 0);
    }
}

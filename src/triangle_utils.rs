use crate::Triangle;

/// Returns `true` if any vertex of `triangle` is also a vertex of `super_triangle`.
pub fn triangle_contains_vertex_from_super_triangle(
    triangle: &Triangle,
    super_triangle: &Triangle,
) -> bool {
    let super_triangle_vertices = super_triangle.vertices();
    let triangle_vertices = triangle.vertices();
    for super_triangle_vertex in super_triangle_vertices {
        if super_triangle_vertex == triangle_vertices[0] {
            return true;
        }
        if super_triangle_vertex == triangle_vertices[1] {
            return true;
        }
        if super_triangle_vertex == triangle_vertices[2] {
            return true;
        }
    }
    false
}

/// Filters out all triangles that share a vertex with the super-triangle.
///
/// This is used as the final step of the Bowyer-Watson algorithm to remove
/// artifacts introduced by the super-triangle.
pub fn remove_triangles_with_vertices_from_super_triangle(
    triangles: &[Triangle],
    super_triangle: &Triangle,
) -> Vec<Triangle> {
    let mut res: Vec<Triangle> = Vec::new();

    for triangle in triangles {
        if !triangle_contains_vertex_from_super_triangle(triangle, super_triangle) {
            res.push(*triangle);
        }
    }
    res
}

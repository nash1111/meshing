use crate::Tetrahedron;

pub fn tetrahedron_contains_vertex_from_super_tetrahedron(
    tetrahedron: &Tetrahedron,
    super_tetrahedron: &Tetrahedron,
) -> bool {
    let super_vertices = super_tetrahedron.vertices();
    let tet_vertices = tetrahedron.vertices();
    for sv in super_vertices {
        for tv in tet_vertices {
            if sv == tv {
                return true;
            }
        }
    }
    false
}

pub fn remove_tetrahedra_with_vertices_from_super_tetrahedron(
    tetrahedra: &Vec<Tetrahedron>,
    super_tetrahedron: &Tetrahedron,
) -> Vec<Tetrahedron> {
    let mut res: Vec<Tetrahedron> = Vec::new();

    for tet in tetrahedra {
        if !tetrahedron_contains_vertex_from_super_tetrahedron(tet, super_tetrahedron) {
            res.push(*tet);
        }
    }
    res
}

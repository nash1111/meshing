use crate::{Face, Point3D, Tetrahedron};

pub fn create_super_tetrahedron(points: &Vec<Point3D>) -> Tetrahedron {
    if points.is_empty() {
        panic!("The input points vector should not be empty.");
    }

    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut min_z = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;
    let mut max_z = f64::MIN;

    for point in points {
        if point.x < min_x {
            min_x = point.x;
        }
        if point.y < min_y {
            min_y = point.y;
        }
        if point.z < min_z {
            min_z = point.z;
        }
        if point.x > max_x {
            max_x = point.x;
        }
        if point.y > max_y {
            max_y = point.y;
        }
        if point.z > max_z {
            max_z = point.z;
        }
    }

    let mid_x = (min_x + max_x) / 2.0;
    let mid_y = (min_y + max_y) / 2.0;
    let mid_z = (min_z + max_z) / 2.0;
    let dx = max_x - min_x;
    let dy = max_y - min_y;
    let dz = max_z - min_z;
    let d = dx.max(dy).max(dz);
    let margin = d * 100.0 + 100.0;

    let index = i64::MIN;

    let a = Point3D {
        index,
        x: mid_x - margin,
        y: mid_y - margin,
        z: mid_z - margin,
    };
    let b = Point3D {
        index: index + 1,
        x: mid_x + margin,
        y: mid_y - margin,
        z: mid_z - margin,
    };
    let c = Point3D {
        index: index + 2,
        x: mid_x,
        y: mid_y + margin,
        z: mid_z - margin,
    };
    let d = Point3D {
        index: index + 3,
        x: mid_x,
        y: mid_y,
        z: mid_z + margin,
    };

    Tetrahedron { a, b, c, d }
}

pub fn face_is_shared_by_tetrahedra(face: &Face, tetrahedra: &Vec<Tetrahedron>) -> bool {
    for tet in tetrahedra {
        let faces_of_tet = tet.faces();
        for face_of_tet in faces_of_tet {
            if face_of_tet == *face {
                return true;
            }
        }
    }
    false
}

pub fn retetrahedralize(face: &Face, point: &Point3D) -> Tetrahedron {
    Tetrahedron {
        a: face.a,
        b: face.b,
        c: face.c,
        d: *point,
    }
}

use meshing::{bowyer_watson_3d, Point3D};

fn main() {
    let cube = vec![
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

    let tetrahedra = bowyer_watson_3d(cube);
    println!("3D Delaunay tetrahedralization of a unit cube:");
    println!("  Number of tetrahedra: {}", tetrahedra.len());
    for (i, tet) in tetrahedra.iter().enumerate() {
        println!(
            "  Tet {}: ({}, {}, {}, {})",
            i, tet.a.index, tet.b.index, tet.c.index, tet.d.index
        );
    }
}

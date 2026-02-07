use criterion::{black_box, criterion_group, criterion_main, Criterion};
use meshing::advancing_front::advancing_front;
use meshing::delaunay_refinement::delaunay_refinement;
use meshing::marching_cubes::marching_cubes;
use meshing::octree::octree_mesh;
use meshing::voxel_mesh::voxel_mesh;
use meshing::{bowyer_watson_3d, Face, Point3D};

fn cube_points(n: usize) -> Vec<Point3D> {
    let mut points = Vec::new();
    let mut idx = 0i64;
    let step = 1.0 / n as f64;
    for i in 0..=n {
        for j in 0..=n {
            for k in 0..=n {
                points.push(Point3D {
                    index: idx,
                    x: i as f64 * step,
                    y: j as f64 * step,
                    z: k as f64 * step,
                });
                idx += 1;
            }
        }
    }
    points
}

fn bench_bowyer_watson_3d(c: &mut Criterion) {
    let points = cube_points(3);
    c.bench_function("bowyer_watson_3d (64 pts)", |b| {
        b.iter(|| bowyer_watson_3d(black_box(points.clone())))
    });
}

fn bench_advancing_front(c: &mut Criterion) {
    let p = [
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
            x: 1.0,
            y: 1.0,
            z: 0.0,
        },
        Point3D {
            index: 3,
            x: 0.0,
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
            x: 1.0,
            y: 1.0,
            z: 1.0,
        },
        Point3D {
            index: 7,
            x: 0.0,
            y: 1.0,
            z: 1.0,
        },
    ];
    let faces = vec![
        Face {
            a: p[0],
            b: p[1],
            c: p[2],
        },
        Face {
            a: p[0],
            b: p[2],
            c: p[3],
        },
        Face {
            a: p[4],
            b: p[6],
            c: p[5],
        },
        Face {
            a: p[4],
            b: p[7],
            c: p[6],
        },
        Face {
            a: p[0],
            b: p[5],
            c: p[1],
        },
        Face {
            a: p[0],
            b: p[4],
            c: p[5],
        },
        Face {
            a: p[2],
            b: p[7],
            c: p[3],
        },
        Face {
            a: p[2],
            b: p[6],
            c: p[7],
        },
        Face {
            a: p[0],
            b: p[3],
            c: p[7],
        },
        Face {
            a: p[0],
            b: p[7],
            c: p[4],
        },
        Face {
            a: p[1],
            b: p[5],
            c: p[6],
        },
        Face {
            a: p[1],
            b: p[6],
            c: p[2],
        },
    ];
    let points = p.to_vec();
    c.bench_function("advancing_front (cube)", |b| {
        b.iter(|| advancing_front(black_box(faces.clone()), black_box(points.clone())))
    });
}

fn bench_octree(c: &mut Criterion) {
    let min = Point3D {
        index: 0,
        x: -1.0,
        y: -1.0,
        z: -1.0,
    };
    let max = Point3D {
        index: 0,
        x: 1.0,
        y: 1.0,
        z: 1.0,
    };
    c.bench_function("octree_mesh (depth=3, sphere)", |b| {
        b.iter(|| {
            octree_mesh(black_box(min), black_box(max), 3, &|p| {
                p.x * p.x + p.y * p.y + p.z * p.z <= 1.0
            })
        })
    });
}

fn bench_marching_cubes(c: &mut Criterion) {
    let min = Point3D {
        index: 0,
        x: -2.0,
        y: -2.0,
        z: -2.0,
    };
    let max = Point3D {
        index: 0,
        x: 2.0,
        y: 2.0,
        z: 2.0,
    };
    let field = |x: f64, y: f64, z: f64| x * x + y * y + z * z - 1.0;
    c.bench_function("marching_cubes (20^3, sphere)", |b| {
        b.iter(|| marching_cubes(20, 20, 20, black_box(min), black_box(max), &field, 0.0))
    });
}

fn bench_voxel_mesh(c: &mut Criterion) {
    let min = Point3D {
        index: 0,
        x: -1.0,
        y: -1.0,
        z: -1.0,
    };
    let max = Point3D {
        index: 0,
        x: 1.0,
        y: 1.0,
        z: 1.0,
    };
    c.bench_function("voxel_mesh (8^3, sphere)", |b| {
        b.iter(|| {
            voxel_mesh(black_box(min), black_box(max), 8, 8, 8, &|p| {
                p.x * p.x + p.y * p.y + p.z * p.z <= 1.0
            })
        })
    });
}

fn bench_delaunay_refinement(c: &mut Criterion) {
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
    c.bench_function("delaunay_refinement (cube, ratio=2.0)", |b| {
        b.iter(|| delaunay_refinement(black_box(points.clone()), 2.0))
    });
}

criterion_group!(
    benches,
    bench_bowyer_watson_3d,
    bench_advancing_front,
    bench_octree,
    bench_marching_cubes,
    bench_voxel_mesh,
    bench_delaunay_refinement,
);
criterion_main!(benches);

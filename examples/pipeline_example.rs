use meshing::export::{faces_to_stl, tetrahedra_to_stl, tetrahedra_to_vtk};
use meshing::marching_cubes::marching_cubes;
use meshing::pipeline::{octree_refined, surface_to_volume, voxel_refined};
use meshing::Point3D;
use std::fs;

fn main() {
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

    // --- Marching Cubes: sphere isosurface ---
    println!("=== Marching Cubes (sphere) ===");
    let sphere = |x: f64, y: f64, z: f64| x * x + y * y + z * z - 1.0;
    let faces = marching_cubes(15, 15, 15, min, max, &sphere, 0.0);
    println!("  Faces: {}", faces.len());
    let stl = faces_to_stl(&faces, "marching_cubes_sphere");
    fs::write("examples/marching_cubes_sphere.stl", &stl).unwrap();
    println!("  -> examples/marching_cubes_sphere.stl");

    // --- Marching Cubes: torus isosurface ---
    println!("\n=== Marching Cubes (torus) ===");
    let torus = |x: f64, y: f64, z: f64| {
        let r_major = 1.0;
        let r_minor = 0.4;
        let q = ((x * x + y * y).sqrt() - r_major).powi(2) + z * z;
        q - r_minor * r_minor
    };
    let faces = marching_cubes(20, 20, 20, min, max, &torus, 0.0);
    println!("  Faces: {}", faces.len());
    let stl = faces_to_stl(&faces, "marching_cubes_torus");
    fs::write("examples/marching_cubes_torus.stl", &stl).unwrap();
    println!("  -> examples/marching_cubes_torus.stl");

    // --- Surface to Volume (sphere) ---
    println!("\n=== Surface to Volume (sphere) ===");
    let tets = surface_to_volume(8, 8, 8, min, max, &sphere, 0.0);
    println!("  Tetrahedra: {}", tets.len());
    let stl = tetrahedra_to_stl(&tets, "surface_to_volume");
    fs::write("examples/surface_to_volume.stl", &stl).unwrap();
    let vtk = tetrahedra_to_vtk(&tets, "surface_to_volume");
    fs::write("examples/surface_to_volume.vtk", &vtk).unwrap();
    println!("  -> examples/surface_to_volume.stl, .vtk");

    // --- Octree Refined ---
    println!("\n=== Octree Refined ===");
    let tets = octree_refined(
        min,
        max,
        2,
        &|p| p.x * p.x + p.y * p.y + p.z * p.z < 1.5 * 1.5,
        2.0,
    );
    println!("  Tetrahedra: {}", tets.len());
    let vtk = tetrahedra_to_vtk(&tets, "octree_refined");
    fs::write("examples/octree_refined.vtk", &vtk).unwrap();
    println!("  -> examples/octree_refined.vtk");

    // --- Voxel Refined ---
    println!("\n=== Voxel Refined ===");
    let tets = voxel_refined(
        min,
        max,
        3,
        3,
        3,
        &|p| p.x * p.x + p.y * p.y + p.z * p.z < 1.5 * 1.5,
        2.0,
    );
    println!("  Tetrahedra: {}", tets.len());
    let vtk = tetrahedra_to_vtk(&tets, "voxel_refined");
    fs::write("examples/voxel_refined.vtk", &vtk).unwrap();
    println!("  -> examples/voxel_refined.vtk");

    println!("\nDone! View STL files with MeshLab/Blender, VTK files with ParaView.");
}

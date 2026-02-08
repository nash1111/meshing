use meshing::export::{
    faces_to_glb, faces_to_glb_quantized, faces_to_obj, faces_to_stl, tetrahedra_to_vtk,
};
use meshing::marching_cubes::marching_cubes;
use meshing::pipeline::surface_to_volume;
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
    let sphere = |x: f64, y: f64, z: f64| x * x + y * y + z * z - 1.0;

    // Generate sphere surface mesh with marching cubes
    println!("Generating sphere surface with marching cubes (resolution 20x20x20)...");
    let faces = marching_cubes(20, 20, 20, min, max, &sphere, 0.0);
    println!("  Surface faces: {}", faces.len());

    // Export surface to STL
    let stl = faces_to_stl(&faces, "sphere");
    fs::write("examples/sphere.stl", &stl).unwrap();
    println!("  Wrote examples/sphere.stl ({} bytes)", stl.len());

    // Export surface to OBJ
    let obj = faces_to_obj(&faces);
    fs::write("examples/sphere.obj", &obj).unwrap();
    println!("  Wrote examples/sphere.obj ({} bytes)", obj.len());

    // Export surface to GLB
    let glb = faces_to_glb(&faces);
    fs::write("examples/sphere.glb", &glb).unwrap();
    println!("  Wrote examples/sphere.glb ({} bytes)", glb.len());

    // Export surface to quantized GLB
    let glb_q = faces_to_glb_quantized(&faces);
    fs::write("examples/sphere_quantized.glb", &glb_q).unwrap();
    println!(
        "  Wrote examples/sphere_quantized.glb ({} bytes, {:.0}% of regular)",
        glb_q.len(),
        glb_q.len() as f64 / glb.len() as f64 * 100.0
    );

    // Generate volume mesh with surface_to_volume pipeline
    println!("\nGenerating volume mesh with surface_to_volume pipeline (8x8x8)...");
    let tetrahedra = surface_to_volume(8, 8, 8, min, max, &sphere, 0.0);
    println!("  Volume tetrahedra: {}", tetrahedra.len());

    // Export volume to VTK
    let vtk = tetrahedra_to_vtk(&tetrahedra, "sphere_volume");
    fs::write("examples/sphere_volume.vtk", &vtk).unwrap();
    println!("  Wrote examples/sphere_volume.vtk ({} bytes)", vtk.len());

    println!("\nDone! Output files can be viewed with:");
    println!("  STL/OBJ/GLB -> MeshLab, Blender, https://gltf-viewer.donmccurdy.com/");
    println!("  VTK         -> ParaView");
}

use meshing::export::extract_surface_faces;
use meshing::marching_cubes::marching_cubes;
use meshing::octree::octree_mesh;
use meshing::{Face, Point3D};
use plotters::prelude::*;

fn face_center_depth(face: &Face, yaw: f64, pitch: f64) -> f64 {
    // Compute average depth after rotation for painter's algorithm sorting
    let cx = (face.a.x + face.b.x + face.c.x) / 3.0;
    let cy = (face.a.y + face.b.y + face.c.y) / 3.0;
    let cz = (face.a.z + face.b.z + face.c.z) / 3.0;
    // Approximate depth in camera space
    let rotated_z = -cx * yaw.sin() + cz * yaw.cos();
    let depth = -cy * pitch.sin() + rotated_z * pitch.cos();
    depth
}

fn face_normal(face: &Face) -> (f64, f64, f64) {
    let ux = face.b.x - face.a.x;
    let uy = face.b.y - face.a.y;
    let uz = face.b.z - face.a.z;
    let vx = face.c.x - face.a.x;
    let vy = face.c.y - face.a.y;
    let vz = face.c.z - face.a.z;
    let nx = uy * vz - uz * vy;
    let ny = uz * vx - ux * vz;
    let nz = ux * vy - uy * vx;
    let len = (nx * nx + ny * ny + nz * nz).sqrt();
    if len < 1e-15 {
        return (0.0, 0.0, 1.0);
    }
    (nx / len, ny / len, nz / len)
}

fn shade_color(face: &Face, base: RGBColor) -> RGBColor {
    let (nx, ny, nz) = face_normal(face);
    // Light direction (normalized): from upper-right-front
    let lx: f64 = 0.4;
    let ly: f64 = 0.6;
    let lz: f64 = 0.7;
    let len = (lx * lx + ly * ly + lz * lz).sqrt();
    let dot = (nx * lx + ny * ly + nz * lz) / len;
    let intensity = 0.3 + 0.7 * dot.abs(); // ambient + diffuse
    let r = (base.0 as f64 * intensity).min(255.0) as u8;
    let g = (base.1 as f64 * intensity).min(255.0) as u8;
    let b = (base.2 as f64 * intensity).min(255.0) as u8;
    RGBColor(r, g, b)
}

fn render_faces(
    faces: &[Face],
    filename: &str,
    title: &str,
    base_color: RGBColor,
    range: f64,
    yaw: f64,
    pitch: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(filename, (800, 800)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 24))
        .build_cartesian_3d(-range..range, -range..range, -range..range)?;

    chart.with_projection(|mut pb| {
        pb.yaw = yaw;
        pb.pitch = pitch;
        pb.scale = 0.85;
        pb.into_matrix()
    });

    chart.configure_axes().draw()?;

    // Sort faces back-to-front (painter's algorithm)
    let mut sorted_faces: Vec<&Face> = faces.iter().collect();
    sorted_faces.sort_by(|a, b| {
        face_center_depth(a, yaw, pitch)
            .partial_cmp(&face_center_depth(b, yaw, pitch))
            .unwrap()
    });

    for face in &sorted_faces {
        let color = shade_color(face, base_color);
        let pts = vec![
            (face.a.x, face.a.y, face.a.z),
            (face.b.x, face.b.y, face.b.z),
            (face.c.x, face.c.y, face.c.z),
        ];
        chart.draw_series(std::iter::once(Polygon::new(pts, color.filled())))?;
    }

    root.present()?;
    println!("  Wrote {}", filename);
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    // --- Sphere (Marching Cubes) ---
    println!("Rendering sphere (marching cubes)...");
    let sphere_fn = |x: f64, y: f64, z: f64| x * x + y * y + z * z - 1.0;
    let sphere_faces = marching_cubes(20, 20, 20, min, max, &sphere_fn, 0.0);
    println!("  Faces: {}", sphere_faces.len());
    render_faces(
        &sphere_faces,
        "examples/marching_cubes_sphere.png",
        "Marching Cubes: Sphere",
        RGBColor(70, 130, 220),
        2.0,
        0.6,
        0.3,
    )?;

    // --- Torus (Marching Cubes) ---
    println!("Rendering torus (marching cubes)...");
    let torus_fn = |x: f64, y: f64, z: f64| {
        let r_major = 1.0;
        let r_minor = 0.4;
        let q = ((x * x + y * y).sqrt() - r_major).powi(2) + z * z;
        q - r_minor * r_minor
    };
    let torus_faces = marching_cubes(25, 25, 25, min, max, &torus_fn, 0.0);
    println!("  Faces: {}", torus_faces.len());
    render_faces(
        &torus_faces,
        "examples/marching_cubes_torus.png",
        "Marching Cubes: Torus",
        RGBColor(220, 100, 70),
        2.0,
        0.5,
        0.4,
    )?;

    // --- Gyroid (Marching Cubes) ---
    println!("Rendering gyroid (marching cubes)...");
    let gyroid_fn = |x: f64, y: f64, z: f64| {
        let s = 3.0;
        (s * x).sin() * (s * y).cos()
            + (s * y).sin() * (s * z).cos()
            + (s * z).sin() * (s * x).cos()
    };
    let gyroid_faces = marching_cubes(30, 30, 30, min, max, &gyroid_fn, 0.0);
    println!("  Faces: {}", gyroid_faces.len());
    render_faces(
        &gyroid_faces,
        "examples/marching_cubes_gyroid.png",
        "Marching Cubes: Gyroid",
        RGBColor(100, 190, 100),
        2.0,
        0.7,
        0.35,
    )?;

    // --- Voxel Mesh (sphere) ---
    println!("Rendering voxel mesh (sphere)...");
    let voxel_tets = meshing::voxel_mesh::voxel_mesh(min, max, 4, 4, 4, &|p| {
        p.x * p.x + p.y * p.y + p.z * p.z < 1.5 * 1.5
    });
    let voxel_faces = extract_surface_faces(&voxel_tets);
    println!(
        "  Tetrahedra: {}, Surface faces: {}",
        voxel_tets.len(),
        voxel_faces.len()
    );
    render_faces(
        &voxel_faces,
        "examples/voxel_mesh.png",
        "Voxel Mesh (4x4x4)",
        RGBColor(180, 130, 220),
        2.0,
        0.5,
        0.3,
    )?;

    // --- Octree mesh ---
    println!("Rendering octree mesh...");
    let octree_tets = octree_mesh(min, max, 3, &|p| {
        p.x * p.x + p.y * p.y + p.z * p.z < 1.5 * 1.5
    });
    let octree_faces = extract_surface_faces(&octree_tets);
    println!(
        "  Tetrahedra: {}, Surface faces: {}",
        octree_tets.len(),
        octree_faces.len()
    );
    render_faces(
        &octree_faces,
        "examples/octree_mesh.png",
        "Octree Mesh (depth=3)",
        RGBColor(220, 180, 60),
        2.0,
        0.6,
        0.25,
    )?;

    println!("\nDone! All images saved to examples/");
    Ok(())
}

//! Minimal two-triangle 3D fabric hinge example with an optional seam.
use std::collections::BTreeSet;
use std::env;
use std::fs;
use textillab::unfold::{TriangleMesh, Vec3};

fn main() -> Result<(), String> {
    let mut seams = BTreeSet::new();
    if env::args().any(|arg| arg == "--split") {
        seams.insert((0, 2));
    }
    let mesh = TriangleMesh {
        vertices: vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(120.0, 0.0, 0.0),
            Vec3::new(120.0, 80.0, 0.0),
            Vec3::new(0.0, 80.0, 35.0),
        ],
        faces: vec![[0, 1, 2], [0, 2, 3]],
        seams,
    };
    let patches = mesh.unfold()?;
    let destination = env::args()
        .find(|s| s.ends_with(".svg"))
        .unwrap_or_else(|| "unfold-example.svg".into());
    fs::write(&destination, patches.svg()).map_err(|e| e.to_string())?;
    println!(
        "Exported {} patch(es) to {}",
        patches.patches.len(),
        destination
    );
    Ok(())
}

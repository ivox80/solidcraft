//! Import a Wavefront OBJ, list mesh edges, choose cut seams and unfold.
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use textillab::obj_import::{mesh_edges, parse_obj, parse_seam};

fn usage() {
    println!(
        "Usage: cargo run --bin textillab-obj -- <input.obj> [--edges] [--seam 1:3] [--interactive] [--out pattern.svg]\n         All OBJ position coordinates are interpreted in the input model's units.\n         Cut seam vertex IDs are 1-based OBJ indices.\n         --interactive: enter seam pairs as 1:3, blank line finishes."
    );
}

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() || args.iter().any(|s| s == "--help" || s == "-h") {
        usage();
        return Ok(());
    }
    let path = &args[0];
    let source = fs::read_to_string(path).map_err(|e| format!("Cannot read {path}: {e}"))?;
    let mut mesh = parse_obj(&source)?;
    let mut output = String::from("unfolded.svg");
    let mut list_edges = false;
    let mut interactive = false;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--edges" => list_edges = true,
            "--interactive" => interactive = true,
            "--out" | "--seam" => {
                let option = &args[i];
                i += 1;
                let value = args
                    .get(i)
                    .ok_or_else(|| format!("{option} requires a value"))?;
                if option == "--seam" {
                    let seam = parse_seam(value, &mesh)?;
                    mesh.seams.insert(seam);
                } else {
                    output = value.clone();
                }
            }
            other => return Err(format!("Unknown option '{other}'")),
        }
        i += 1;
    }
    let edges = mesh_edges(&mesh);
    println!(
        "Loaded {} vertices, {} triangles, {} distinct edges",
        mesh.vertices.len(),
        mesh.faces.len(),
        edges.len()
    );
    if list_edges || interactive {
        println!("OBJ edge IDs (shared edges marked internal):");
        for edge in &edges {
            println!(
                "  {}:{}  {}",
                edge.a + 1,
                edge.b + 1,
                if edge.adjacent_faces.len() == 2 {
                    "internal"
                } else {
                    "boundary"
                }
            );
        }
    }
    if interactive {
        loop {
            print!("Add seam pair, empty line to unfold: ");
            io::stdout().flush().map_err(|e| e.to_string())?;
            let mut line = String::new();
            if io::stdin()
                .read_line(&mut line)
                .map_err(|e| e.to_string())?
                == 0
                || line.trim().is_empty()
            {
                break;
            }
            match parse_seam(line.trim(), &mesh) {
                Ok(edge) => {
                    mesh.seams.insert(edge);
                    println!("Selected {}:{}", edge.0 + 1, edge.1 + 1);
                }
                Err(error) => eprintln!("{error}"),
            }
        }
    }
    let result = mesh.unfold()?;
    if let Some(parent) = Path::new(&output).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
    }
    fs::write(&output, result.svg()).map_err(|e| e.to_string())?;
    println!(
        "Unfolded {} patch(es), {} seam(s). Saved: {}",
        result.patches.len(),
        mesh.seams.len(),
        output
    );
    Ok(())
}

use std::env;
use std::path::Path;
use textillab::{Material, Pattern};

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let output_dir = match args.get(1).map(String::as_str) {
        Some("smoke-exports") => "smoke-exports",
        Some("exports") => args.get(2).map(String::as_str).unwrap_or("exports"),
        _ => {
            println!("Usage: textillab-demo [smoke-exports | exports [output-folder]]");
            return Ok(());
        }
    };
    let pattern = Pattern::rectangle(
        "Machine cover",
        400.0,
        250.0,
        12.0,
        Material::nylon_420d_example(),
    )?;
    pattern.write_exports(Path::new(output_dir))?;
    println!("Wrote SVG and OBJ to {output_dir}");
    Ok(())
}

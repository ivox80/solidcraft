//! Generate a dimensional SVG garment block from body measurements (cm).
//! Example: cargo run --bin textillab-pattern -- jacket chest=108 waist=92 ease=12 --out jacket.svg
use std::env;
use std::fs;
use std::path::Path;
use textillab::garments::{export_svg, Garment, Measurements};

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() || args[0] == "--help" {
        println!(
            "Usage: textillab-pattern <top|jacket|trousers|glove|beanie|hat|skirt> [measure_cm=value] [--allowance mm] [--density g/m2] [--out file.svg]\n\
             Measurements: chest waist hip back shoulder sleeve inseam head hand handLength ease.\n\
             Example: textillab-pattern trousers hip=110 waist=88 inseam=82 --allowance 10 --out trousers.svg\n\
             CAUTION: Concept pattern; fit, grading and sewing are not validated."
        );
        return Ok(());
    }
    let garment: Garment = args[0].parse()?;
    let mut measures = Measurements::default();
    let (mut allowance, mut density) = (10.0, 210.0);
    let mut out = format!("{}-block.svg", garment.name());
    let mut index = 1;
    while index < args.len() {
        let argument = &args[index];
        if matches!(argument.as_str(), "--allowance" | "--density" | "--out") {
            index += 1;
            let value = args
                .get(index)
                .ok_or_else(|| format!("Missing value after {argument}"))?;
            match argument.as_str() {
                "--allowance" => allowance = value.parse().map_err(|_| "Invalid allowance")?,
                "--density" => density = value.parse().map_err(|_| "Invalid density")?,
                "--out" => out.clone_from(value),
                _ => {}
            }
        } else {
            let (name, value) = argument
                .split_once('=')
                .ok_or_else(|| format!("Expected measure=value, got {argument}"))?;
            let cm: f64 = value
                .parse()
                .map_err(|_| format!("Invalid value for {name}"))?;
            measures.set_cm(name, cm)?;
        }
        index += 1;
    }
    let report = export_svg(garment, measures, allowance, density)?;
    if let Some(parent) = Path::new(&out).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
    }
    fs::write(&out, &report.svg).map_err(|e| e.to_string())?;
    println!(
        "{}: {} piece(s), {:.3} m² cut area, {:.1} g estimated fabric mass; saved {}",
        garment.name(),
        report.pieces_to_cut,
        report.cut_area_mm2 / 1_000_000.0,
        report.estimated_mass_g,
        out
    );
    Ok(())
}

//! Generate a dimensional SVG garment block from body measurements (cm).
//! Example: cargo run --bin textillab-pattern -- jacket chest=108 waist=92 ease=12 --out jacket.svg
use std::env;
use std::fs;
use std::path::Path;
use textillab::drafting_rules::{self, Fit, Settings, Style};
use textillab::garments::{export_svg, Garment, Measurements};
use textillab::hood_rules::{
    self, Measurements as HoodMeasurements, Options as HoodOptions, Profile as HoodProfile,
    SewingMode as HoodMode, Style as HoodStyle,
};
use textillab::pants_rules::{self, PantsMeasures};

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() || args[0] == "--help" {
        println!(
            "Usage: textillab-pattern <hood|top|jacket|trousers|glove|beanie|hat|skirt> [measure_cm=value] [--allowance mm] [--density g/m2] [--out file.svg]\n\\
             Bodice v1.0 options: --fit fitted|regular|comfort|loose|oversize|custom, --style bodice|shirt|blouse|jacket|coat|sleeveless, --cap-ease mm, --torso-mm d, --hip-mm d, --hem-mm d, --sleeve-mm d\n\
             Measurements: chest waist hip back shoulder sleeve inseam head hand handLength ease.\n\
             Pants: waist hip inseam outseam crotchDepth waistToHip waistToKnee thigh knee hem hipEase waistEase dartIntake crotchExtension (all cm).\n\
             Example: textillab-pattern trousers hip=110 waist=88 inseam=82 --allowance 10 --out trousers.svg\n\
             CAUTION: Concept pattern; fit, grading and sewing are not validated."
        );
        return Ok(());
    }
    if args[0] == "hood" {
        return hood_cli(&args[1..]);
    }
    let garment: Garment = args[0].parse()?;
    let mut measures = Measurements::default();
    let mut pants = PantsMeasures::default();
    let (mut allowance, mut density) = (10.0, 210.0);
    let mut draft_settings = Settings::default();
    let mut out = format!("{}-block.svg", garment.name());
    let mut index = 1;
    while index < args.len() {
        let argument = &args[index];
        if matches!(
            argument.as_str(),
            "--allowance"
                | "--density"
                | "--out"
                | "--fit"
                | "--style"
                | "--cap-ease"
                | "--torso-mm"
                | "--hip-mm"
                | "--hem-mm"
                | "--sleeve-mm"
        ) {
            index += 1;
            let value = args
                .get(index)
                .ok_or_else(|| format!("Missing value after {argument}"))?;
            match argument.as_str() {
                "--allowance" => allowance = value.parse().map_err(|_| "Invalid allowance")?,
                "--density" => density = value.parse().map_err(|_| "Invalid density")?,
                "--out" => out.clone_from(value),
                "--fit" => draft_settings.fit = Fit::from_key(value)?,
                "--style" => draft_settings.style = Style::from_key(value)?,
                "--cap-ease" => {
                    draft_settings.target_cap_ease_mm =
                        value.parse().map_err(|_| "Invalid target ease")?
                }
                "--torso-mm" => {
                    draft_settings.alterations.torso_mm =
                        value.parse().map_err(|_| "Invalid torso adjustment")?
                }
                "--hip-mm" => {
                    draft_settings.alterations.hip_mm =
                        value.parse().map_err(|_| "Invalid hip adjustment")?
                }
                "--hem-mm" => {
                    draft_settings.alterations.hem_mm =
                        value.parse().map_err(|_| "Invalid hem adjustment")?
                }
                "--sleeve-mm" => {
                    draft_settings.alterations.sleeve_mm =
                        value.parse().map_err(|_| "Invalid sleeve adjustment")?
                }
                _ => {}
            }
        } else {
            let (name, value) = argument
                .split_once('=')
                .ok_or_else(|| format!("Expected measure=value, got {argument}"))?;
            let cm: f64 = value
                .parse()
                .map_err(|_| format!("Invalid value for {name}"))?;
            if garment == Garment::Trousers {
                pants.set_cm(name, cm)?;
            } else {
                measures.set_cm(name, cm)?;
            }
        }
        index += 1;
    }
    let report = if garment == Garment::Top {
        let drafted = drafting_rules::generate(measures, draft_settings, allowance, density)?;
        println!(
            "Construction system: TextilLab Experimental 1.0 | Armhole {:.2} mm | Sleeve cap {:.2} mm | Ease {:.2} mm (target {:.2}) | Side Δ {:.2} mm",
            drafted.report.total_armhole_mm, drafted.report.sleeve_cap_mm,
            drafted.report.sleeve_ease_mm, drafted.report.target_ease_mm,
            drafted.report.side_difference_mm,
        );
        if !drafted.report.reachable {
            eprintln!("WARNING: Target sleeve ease is out of the permitted cap-height range");
        }
        drafted.pattern
    } else if garment == Garment::Trousers {
        let drafted = pants_rules::draft(pants)?;
        println!(
            "TextilLab experimental pants | Side seam delta {:.2} mm | Inseam delta {:.2} mm | 10 construction rules",
            drafted.checks.side_diff_mm, drafted.checks.inseam_diff_mm
        );
        pants_rules::export(pants, allowance, density)?
    } else {
        export_svg(garment, measures, allowance, density)?
    };
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

fn hood_cli(args: &[String]) -> Result<(), String> {
    // First determine the body profile so input values always override its
    // independently supplied *example* defaults, regardless of CLI order.
    let mut group = HoodProfile::Women;
    for window in args.windows(2) {
        if window[0] == "--profile" {
            group = HoodProfile::parse(&window[1])?;
        }
    }
    let mut measures = HoodMeasurements::for_profile(group);
    let mut opts = HoodOptions {
        profile: group,
        ..HoodOptions::default()
    };
    let mut out = String::from("hood-block.svg");
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--material-stretch-verified" {
            opts.material_stretch_verified = true;
        } else if arg == "--drawstrings" {
            opts.drawstrings = true;
        } else if matches!(
            arg.as_str(),
            "--profile"
                | "--style"
                | "--neck-mode"
                | "--planned-ease"
                | "--stretch"
                | "--allowance"
                | "--density"
                | "--out"
        ) {
            i += 1;
            let value = args
                .get(i)
                .ok_or_else(|| format!("Missing value for {arg}"))?;
            match arg.as_str() {
                "--profile" => {}
                "--style" => opts.style = HoodStyle::parse(value)?,
                "--neck-mode" => opts.mode = HoodMode::parse(value)?,
                "--planned-ease" => {
                    opts.planned_ease_mm = value.parse().map_err(|_| "Invalid ease in mm")?
                }
                "--stretch" => {
                    opts.material_stretch_pct =
                        value.parse().map_err(|_| "Invalid stretch percent")?
                }
                "--allowance" => {
                    opts.seam_allowance_mm = value.parse().map_err(|_| "Invalid seam allowance")?
                }
                "--density" => opts.density_g_m2 = value.parse().map_err(|_| "Invalid density")?,
                "--out" => out.clone_from(value),
                _ => unreachable!(),
            }
        } else {
            let (key, value) = arg
                .split_once('=')
                .ok_or_else(|| format!("Expected hood measure=value, got {arg}"))?;
            let number: f64 = value.parse().map_err(|_| format!("Invalid value: {key}"))?;
            measures.set_cm(key, number)?;
        }
        i += 1;
    }
    let detailed = hood_rules::draft(measures, opts)?;
    let report = hood_rules::export(measures, opts)?;
    if let Some(parent) = Path::new(&out).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
    }
    fs::write(&out, &report.svg).map_err(|e| e.to_string())?;
    println!(
        "Hood: {} pieces | original experimental geometry, NOT production validated",
        report.pieces_to_cut
    );
    for seam in &detailed.seams {
        println!(
            "{}: hood {:.3} mm | garment {:.3} mm | delta {:+.3} mm | {}",
            seam.id,
            seam.hood_mm,
            seam.garment_mm,
            seam.delta_mm,
            if seam.matched { "OK" } else { "ERROR" }
        );
    }
    println!("SVG: {}", out);
    Ok(())
}

//! SVG cutting-pattern preview derived from unfolded triangular mesh patches.
//! Units: input geometry and seam allowance in millimetres, density in g/m².

use crate::unfold::{FlatPatch, TriangleMesh};
use crate::{Material, Pattern, Point};
use std::collections::BTreeMap;
use std::fmt::Write;

#[derive(Clone, Debug)]
pub struct StudioExport {
    pub svg: String,
    pub pieces: usize,
    pub sewn_area_mm2: f64,
    pub cut_area_mm2: f64,
    pub estimated_mass_g: f64,
}

fn polygon_area(points: &[Point]) -> f64 {
    points
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let q = points[(i + 1) % points.len()];
            p.x * q.y - q.x * p.y
        })
        .sum::<f64>()
        / 2.0
}

/// Extract the closed boundary of one simply-connected patch by mesh vertex ID.
/// Interior edges occur twice and are removed; seam edges form separate patches.
pub fn patch_boundary(mesh: &TriangleMesh, patch: &FlatPatch) -> Result<Vec<Point>, String> {
    let mut count: BTreeMap<(usize, usize), usize> = BTreeMap::new();
    let mut flat = BTreeMap::<usize, Point>::new();
    for (&face_id, tri) in patch.face_ids.iter().zip(&patch.triangles) {
        let face = mesh
            .faces
            .get(face_id)
            .ok_or("Patch references an invalid triangle")?;
        for i in 0..3 {
            let a = face[i];
            let b = face[(i + 1) % 3];
            *count.entry((a.min(b), a.max(b))).or_default() += 1;
            flat.insert(a, tri[i]);
        }
    }
    let mut adjacent: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for ((a, b), seen) in count {
        if seen == 1 {
            adjacent.entry(a).or_default().push(b);
            adjacent.entry(b).or_default().push(a);
        } else if seen > 2 {
            return Err("Patch has non-manifold triangle edges".into());
        }
    }
    if adjacent.len() < 3 || adjacent.values().any(|v| v.len() != 2) {
        return Err("Patch boundary is not a single simple loop".into());
    }
    let start = *adjacent.keys().next().ok_or("No boundary")?;
    let mut current = start;
    let mut previous = None;
    let mut vertices = Vec::with_capacity(adjacent.len());
    loop {
        vertices.push(current);
        let neighbours = &adjacent[&current];
        let next = if Some(neighbours[0]) == previous {
            neighbours[1]
        } else {
            neighbours[0]
        };
        if next == start {
            break;
        }
        if vertices.len() >= adjacent.len() {
            return Err("Boundary has loops or branches".into());
        }
        previous = Some(current);
        current = next;
    }
    if vertices.len() != adjacent.len() {
        return Err("Patch has multiple boundary loops or holes".into());
    }
    let outline: Vec<Point> = vertices
        .iter()
        .map(|index| flat[index])
        .collect();
    if polygon_area(&outline).abs() <= 1e-8 {
        return Err("Degenerate boundary polygon".into());
    }
    Ok(outline)
}

fn polygon_svg(points: &[Point], stroke: &str, extra: &str) -> String {
    let mut result = format!(
        "<polygon fill=\"none\" stroke=\"{}\" stroke-width=\"0.65\" {} points=\"",
        stroke, extra
    );
    for point in points {
        let _ = write!(result, "{:.5},{:.5} ", point.x, point.y);
    }
    result.push_str("\"/>");
    result
}

pub fn make_cut_preview(
    mesh: &TriangleMesh,
    seam_mm: f64,
    density_g_m2: f64,
) -> Result<StudioExport, String> {
    if !seam_mm.is_finite() || !(0.0..=200.0).contains(&seam_mm) {
        return Err("Nahtzugabe muss zwischen 0 und 200 mm liegen".into());
    }
    if !density_g_m2.is_finite() || !(1.0..=5000.0).contains(&density_g_m2) {
        return Err("Flächengewicht muss zwischen 1 und 5000 g/m² liegen".into());
    }
    let unfolded = mesh.unfold()?;
    let mut shapes = Vec::new();
    let mut sewn_area_mm2 = 0.0;
    let mut cut_area_mm2 = 0.0;
    let mut cursor = 10.0;
    let mut tallest: f64 = 40.0;

    for (index, patch) in unfolded.patches.iter().enumerate() {
        let outline = patch_boundary(mesh, patch)?;
        let allowance = if seam_mm == 0.0 {
            outline.clone()
        } else {
            Pattern {
                name: format!("Patch {}", index + 1),
                outline: outline.clone(),
                seam_allowance_mm: seam_mm,
                material: Material {
                    name: "Textile".to_string(),
                    areal_density_g_m2: density_g_m2,
                    thickness_mm: 0.5,
                },
            }
            .cut_outline()
            .map_err(|e| format!("Patch {}: {} (for seam allowance use convex patterns)", index + 1, e))?
        };
        let min_x = allowance
            .iter()
            .chain(outline.iter())
            .map(|p| p.x)
            .fold(f64::INFINITY, f64::min);
        let max_x = allowance
            .iter()
            .chain(outline.iter())
            .map(|p| p.x)
            .fold(f64::NEG_INFINITY, f64::max);
        let min_y = allowance
            .iter()
            .chain(outline.iter())
            .map(|p| p.y)
            .fold(f64::INFINITY, f64::min);
        let max_y = allowance
            .iter()
            .chain(outline.iter())
            .map(|p| p.y)
            .fold(f64::NEG_INFINITY, f64::max);
        let _ = writeln!(
            shapes,
            "<g transform=\"translate({:.5} {:.5})\">",
            cursor - min_x,
            18.0 - min_y
        );
        let _ = writeln!(
            shapes,
            "<text x=\"{:.5}\" y=\"{:.5}\" font-size=\"5\" fill=\"#c8d7e8\">Teil {}</text>",
            min_x,
            min_y - 5.0,
            index + 1
        );
        let _ = writeln!(
            shapes,
            "{}",
            polygon_svg(&allowance, "#fc9a66", "")
        );
        let _ = writeln!(
            shapes,
            "{}",
            polygon_svg(&outline, "#76d3d6", "stroke-dasharray=\"3 2\"")
        );
        shapes.push_str("</g>\n");
        sewn_area_mm2 += polygon_area(&outline).abs();
        cut_area_mm2 += polygon_area(&allowance).abs();
        tallest = tallest.max(max_y - min_y + 36.0);
        cursor += max_x - min_x + 24.0;
    }

    let width = cursor.max(50.0);
    let height = tallest.max(50.0);
    let estimated_mass_g = cut_area_mm2 * density_g_m2 / 1_000_000.0;
    let svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {width:.3} {height:.3}\" width=\"{width:.3}mm\" height=\"{height:.3}mm\" data-pieces=\"{}\" data-sewn-area=\"{sewn_area_mm2:.3}\" data-cut-area=\"{cut_area_mm2:.3}\" data-mass=\"{estimated_mass_g:.3}\">\n<rect width=\"100%\" height=\"100%\" fill=\"#14202e\"/>\n{shapes}</svg>\n",
        unfolded.patches.len()
    );
    Ok(StudioExport {
        svg,
        pieces: unfolded.patches.len(),
        sewn_area_mm2,
        cut_area_mm2,
        estimated_mass_g,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::obj_import::parse_obj;

    const QUAD: &str =
        "v 0 0 0\nv 100 0 0\nv 100 60 0\nv 0 60 0\nf 1 2 3 4\n";

    #[test]
    fn rectangle_has_correct_allowance_and_weight() {
        let mesh = parse_obj(QUAD).unwrap();
        let result = make_cut_preview(&mesh, 10.0, 200.0).unwrap();
        assert_eq!(result.pieces, 1);
        assert!((result.sewn_area_mm2 - 6000.0).abs() < 0.0001);
        assert!((result.cut_area_mm2 - 9600.0).abs() < 0.0001);
        assert!((result.estimated_mass_g - 1.92).abs() < 0.0001);
    }

    #[test]
    fn diagonal_seam_creates_two_parts() {
        let mut mesh = parse_obj(QUAD).unwrap();
        mesh.seams.insert((0, 2));
        let result = make_cut_preview(&mesh, 3.0, 200.0).unwrap();
        assert_eq!(result.pieces, 2);
        assert!(result.svg.contains("Teil 2"));
    }

    #[test]
    fn reject_invalid_material_parameters() {
        let mesh = parse_obj(QUAD).unwrap();
        assert!(make_cut_preview(&mesh, -1.0, 200.0).is_err());
        assert!(make_cut_preview(&mesh, 10.0, f64::NAN).is_err());
    }
}

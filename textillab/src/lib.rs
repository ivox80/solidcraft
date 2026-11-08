//! Minimal, dependency-free 2D textile patterns with flat 3D mesh export.
//! Coordinates and seam allowances are measured in millimetres.

use std::fmt::Write;
use std::fs;
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Debug)]
pub struct Material {
    pub name: String,
    pub areal_density_g_m2: f64,
    pub thickness_mm: f64,
}

impl Material {
    pub fn nylon_420d_example() -> Self {
        Self {
            name: "Nylon 420D - illustrative properties".into(),
            areal_density_g_m2: 210.0,
            thickness_mm: 0.35,
        }
    }

    fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty()
            || !self.areal_density_g_m2.is_finite()
            || !self.thickness_mm.is_finite()
            || self.areal_density_g_m2 <= 0.0
            || self.thickness_mm <= 0.0
        {
            return Err("Invalid material properties".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Pattern {
    pub name: String,
    pub outline: Vec<Point>,
    pub seam_allowance_mm: f64,
    pub material: Material,
}

impl Pattern {
    pub fn rectangle(
        name: &str,
        width_mm: f64,
        height_mm: f64,
        seam_allowance_mm: f64,
        material: Material,
    ) -> Result<Self, String> {
        if !width_mm.is_finite()
            || !height_mm.is_finite()
            || !seam_allowance_mm.is_finite()
            || width_mm <= 0.0
            || height_mm <= 0.0
            || seam_allowance_mm < 0.0
        {
            return Err("Invalid rectangle dimensions".into());
        }
        let pattern = Self {
            name: name.into(),
            outline: vec![
                Point::new(0.0, 0.0),
                Point::new(width_mm, 0.0),
                Point::new(width_mm, height_mm),
                Point::new(0.0, height_mm),
            ],
            seam_allowance_mm,
            material,
        };
        pattern.validate()?;
        Ok(pattern)
    }

    pub fn validate(&self) -> Result<(), String> {
        self.material.validate()?;
        if self.name.trim().is_empty()
            || !self.seam_allowance_mm.is_finite()
            || self.seam_allowance_mm < 0.0
            || self.outline.len() < 3
            || !self
                .outline
                .iter()
                .all(|p| p.x.is_finite() && p.y.is_finite())
        {
            return Err("Invalid pattern".into());
        }
        let signed = signed_area(&self.outline);
        if signed.abs() < 1e-8 {
            return Err("Degenerate polygon".into());
        }
        let orientation = signed.signum();
        for i in 0..self.outline.len() {
            let a = self.outline[i];
            let b = self.outline[(i + 1) % self.outline.len()];
            let c = self.outline[(i + 2) % self.outline.len()];
            let cross = (b.x - a.x) * (c.y - b.y) - (b.y - a.y) * (c.x - b.x);
            if cross * orientation <= 1e-8 {
                return Err("Only strictly convex polygons are supported".into());
            }
        }
        Ok(())
    }

    pub fn area_mm2(&self) -> f64 {
        signed_area(&self.outline).abs()
    }

    pub fn mass_g(&self) -> f64 {
        self.area_mm2() * self.material.areal_density_g_m2 / 1_000_000.0
    }

    pub fn cut_outline(&self) -> Result<Vec<Point>, String> {
        self.validate()?;
        let n = self.outline.len();
        let side = signed_area(&self.outline).signum();
        let distance = self.seam_allowance_mm;
        let mut lines = Vec::with_capacity(n);
        for i in 0..n {
            let a = self.outline[i];
            let b = self.outline[(i + 1) % n];
            let dx = b.x - a.x;
            let dy = b.y - a.y;
            let length = dx.hypot(dy);
            if length < 1e-10 {
                return Err("Zero-length edge".into());
            }
            let nx = side * dy / length;
            let ny = -side * dx / length;
            lines.push((
                Point::new(a.x + nx * distance, a.y + ny * distance),
                Point::new(b.x + nx * distance, b.y + ny * distance),
            ));
        }
        let mut result = Vec::with_capacity(n);
        for i in 0..n {
            let previous = lines[(i + n - 1) % n];
            let current = lines[i];
            let r = Point::new(previous.1.x - previous.0.x, previous.1.y - previous.0.y);
            let s = Point::new(current.1.x - current.0.x, current.1.y - current.0.y);
            let cross = r.x * s.y - r.y * s.x;
            if cross.abs() < 1e-10 {
                return Err("Parallel offset lines".into());
            }
            let q = Point::new(current.0.x - previous.0.x, current.0.y - previous.0.y);
            let t = (q.x * s.y - q.y * s.x) / cross;
            let p = Point::new(previous.0.x + t * r.x, previous.0.y + t * r.y);
            if !p.x.is_finite() || !p.y.is_finite() {
                return Err("Offset overflow".into());
            }
            result.push(p);
        }
        Ok(result)
    }

    pub fn cut_area_mm2(&self) -> Result<f64, String> {
        Ok(signed_area(&self.cut_outline()?).abs())
    }

    pub fn svg(&self) -> Result<String, String> {
        let outer = self.cut_outline()?;
        let all = self.outline.iter().chain(outer.iter());
        let (mut min_x, mut min_y) = (f64::INFINITY, f64::INFINITY);
        let (mut max_x, mut max_y) = (f64::NEG_INFINITY, f64::NEG_INFINITY);
        for point in all {
            min_x = min_x.min(point.x);
            min_y = min_y.min(point.y);
            max_x = max_x.max(point.x);
            max_y = max_y.max(point.y);
        }
        let width = max_x - min_x + 20.0;
        let height = max_y - min_y + 20.0;
        let mut output = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"{} {} {} {}\" width=\"{}mm\" height=\"{}mm\">\n",
            min_x - 10.0, min_y - 10.0, width, height, width, height
        );
        for (points, color, dash) in [
            (&outer, "#d04b35", ""),
            (&self.outline, "#28699e", " stroke-dasharray=\"5 3\""),
        ] {
            write!(
                output,
                "<polygon fill=\"none\" stroke=\"{}\" stroke-width=\"0.6\"{} points=\"",
                color, dash
            )
            .map_err(|e| e.to_string())?;
            for point in points {
                write!(output, "{:.3},{:.3} ", point.x, point.y).map_err(|e| e.to_string())?;
            }
            output.push_str("\"/>\n");
        }
        output.push_str("</svg>\n");
        Ok(output)
    }

    pub fn obj(&self) -> Result<String, String> {
        self.validate()?;
        let mut output = String::from("# Flat fabric mesh, units: mm\n");
        for point in &self.outline {
            writeln!(output, "v {:.6} {:.6} 0.0", point.x, point.y).map_err(|e| e.to_string())?;
        }
        for i in 2..self.outline.len() {
            writeln!(output, "f 1 {} {}", i, i + 1).map_err(|e| e.to_string())?;
        }
        Ok(output)
    }

    pub fn write_exports(&self, folder: &Path) -> Result<(), String> {
        fs::create_dir_all(folder).map_err(|e| e.to_string())?;
        fs::write(folder.join("pattern.svg"), self.svg()?).map_err(|e| e.to_string())?;
        fs::write(folder.join("pattern.obj"), self.obj()?).map_err(|e| e.to_string())?;
        Ok(())
    }
}

fn signed_area(points: &[Point]) -> f64 {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rectangle_area_mass_and_cut_allowance() {
        let pattern =
            Pattern::rectangle("Panel", 200.0, 100.0, 10.0, Material::nylon_420d_example())
                .unwrap();
        assert!((pattern.area_mm2() - 20_000.0).abs() < 1e-7);
        assert!((pattern.cut_area_mm2().unwrap() - 26_400.0).abs() < 1e-7);
        assert!((pattern.mass_g() - 4.2).abs() < 1e-7);
    }

    #[test]
    fn reversed_winding_same_cut_area() {
        let mut pattern =
            Pattern::rectangle("Panel", 200.0, 100.0, 10.0, Material::nylon_420d_example())
                .unwrap();
        let before = pattern.cut_area_mm2().unwrap();
        pattern.outline.reverse();
        assert!((before - pattern.cut_area_mm2().unwrap()).abs() < 1e-7);
    }

    #[test]
    fn invalid_dimensions_rejected() {
        assert!(
            Pattern::rectangle("Panel", -1.0, 100.0, 5.0, Material::nylon_420d_example()).is_err()
        );
    }

    #[test]
    fn svg_and_obj_generated() {
        let pattern =
            Pattern::rectangle("Panel", 200.0, 100.0, 10.0, Material::nylon_420d_example())
                .unwrap();
        assert!(pattern.svg().unwrap().contains("<svg"));
        assert!(pattern.obj().unwrap().contains("f 1 2 3"));
    }
}

/// Developable triangular surface unfolding with explicit cut edges.
pub mod unfold;

/// Wavefront OBJ mesh loading and edge selection utilities.
pub mod obj_import;

/// TextilLab Studio panel boundary and SVG preview generation.
pub mod studio_patterns;

/// Measurement-based clothing starter blocks (concept drafts, not fitted patterns).
pub mod garments;

/// Detailed front/back VT/RT bodice prototype with darts and technical drawing.
pub mod bodice_v2;

/// Experimental named construction rules, alterations and seam-balance reports.
pub mod drafting_rules;

/// Measurement-driven trousers draft with traceable experimental rule records.
pub mod pants_rules;

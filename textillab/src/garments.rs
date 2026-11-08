//! Parametric garment starter blocks in millimetres.
//! These are *drafting concepts*, not fitted or production-approved patterns.
//! Curves are sampled as polylines; grainlines and real tailoring constraints
//! are deliberately not represented as if they had been verified.

use crate::Point;
use std::f64::consts::PI;
use std::fmt::Write;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Garment {
    Top,
    Jacket,
    Trousers,
    Glove,
    Beanie,
    Hat,
    Skirt,
}
impl Garment {
    pub const ALL: [Garment; 7] = [
        Self::Top,
        Self::Jacket,
        Self::Trousers,
        Self::Glove,
        Self::Beanie,
        Self::Hat,
        Self::Skirt,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::Jacket => "jacket",
            Self::Trousers => "trousers",
            Self::Glove => "glove",
            Self::Beanie => "beanie",
            Self::Hat => "hat",
            Self::Skirt => "skirt",
        }
    }
}
impl std::str::FromStr for Garment {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|garment| garment.name() == s)
            .ok_or_else(|| format!("Unknown garment type: {s}"))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Measurements {
    pub chest_mm: f64,
    pub waist_mm: f64,
    pub hip_mm: f64,
    pub back_mm: f64,
    pub shoulder_mm: f64,
    pub sleeve_mm: f64,
    pub inseam_mm: f64,
    pub head_mm: f64,
    pub hand_mm: f64,
    pub hand_length_mm: f64,
    pub ease_mm: f64,
}
impl Default for Measurements {
    fn default() -> Self {
        Self {
            chest_mm: 1000.0,
            waist_mm: 840.0,
            hip_mm: 1020.0,
            back_mm: 650.0,
            shoulder_mm: 440.0,
            sleeve_mm: 620.0,
            inseam_mm: 800.0,
            head_mm: 570.0,
            hand_mm: 220.0,
            hand_length_mm: 190.0,
            ease_mm: 80.0,
        }
    }
}
impl Measurements {
    pub fn set_cm(&mut self, key: &str, centimetres: f64) -> Result<(), String> {
        if !centimetres.is_finite() || !(1.0..=300.0).contains(&centimetres) {
            return Err(format!("Invalid measure {key}: {centimetres} cm"));
        }
        let mm = centimetres * 10.0;
        match key {
            "chest" => self.chest_mm = mm,
            "waist" => self.waist_mm = mm,
            "hip" => self.hip_mm = mm,
            "back" => self.back_mm = mm,
            "shoulder" => self.shoulder_mm = mm,
            "sleeve" => self.sleeve_mm = mm,
            "inseam" => self.inseam_mm = mm,
            "head" => self.head_mm = mm,
            "hand" => self.hand_mm = mm,
            "handLength" => self.hand_length_mm = mm,
            "ease" => self.ease_mm = mm,
            _ => return Err(format!("Unknown measurement: {key}")),
        }
        Ok(())
    }
    fn validate(&self) -> Result<(), String> {
        let all = [
            self.chest_mm,
            self.waist_mm,
            self.hip_mm,
            self.back_mm,
            self.shoulder_mm,
            self.sleeve_mm,
            self.inseam_mm,
            self.head_mm,
            self.hand_mm,
            self.hand_length_mm,
            self.ease_mm,
        ];
        if all
            .iter()
            .any(|x| !x.is_finite() || !(10.0..=3000.0).contains(x))
        {
            return Err("All body measurements must be 1–300 cm".into());
        }
        if self.shoulder_mm > self.chest_mm * 0.8 || self.waist_mm > self.hip_mm * 1.6 {
            return Err("Check body measurements (shoulders/waist)".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Piece {
    pub name: &'static str,
    pub outline: Vec<Point>,
    pub holes: Vec<Vec<Point>>,
    pub copies: u32,
    pub on_fold: bool,
    pub note: &'static str,
}
fn pt(x: f64, y: f64) -> Point {
    Point::new(x, y)
}
fn piece(
    name: &'static str,
    outline: Vec<Point>,
    copies: u32,
    on_fold: bool,
    note: &'static str,
) -> Piece {
    Piece {
        name,
        outline,
        holes: Vec::new(),
        copies,
        on_fold,
        note,
    }
}
fn curve(out: &mut Vec<Point>, a: Point, b: Point, c: Point, count: usize) {
    for i in 1..=count {
        let t = i as f64 / count as f64;
        let u = 1.0 - t;
        out.push(pt(
            u * u * a.x + 2.0 * u * t * b.x + t * t * c.x,
            u * u * a.y + 2.0 * u * t * b.y + t * t * c.y,
        ));
    }
}
fn bodice(m: Measurements, front: bool, coat: bool) -> Vec<Point> {
    let ease = if coat { 1.7 } else { 1.0 };
    let width = (m.chest_mm + m.ease_mm * ease) / 4.0;
    let waist = (m.waist_mm + m.ease_mm * ease) / 4.0;
    let length = m.back_mm * if coat { 1.16 } else { 1.0 };
    let neck_w = m.chest_mm * 0.077;
    let neck_d = m.chest_mm * if front { 0.092 } else { 0.029 };
    let arm_d = m.chest_mm * 0.225 + if coat { 22.0 } else { 0.0 };
    let shoulder = (m.shoulder_mm / 2.0 - neck_w).clamp(52.0, 170.0);
    let mut a = vec![pt(0.0, neck_d)];
    curve(
        &mut a,
        pt(0.0, neck_d),
        pt(neck_w * 0.55, neck_d * 0.45),
        pt(neck_w, 0.0),
        5,
    );
    a.push(pt(neck_w + shoulder, 21.0));
    curve(
        &mut a,
        pt(neck_w + shoulder, 21.0),
        pt(width + 20.0, arm_d * 0.45),
        pt(width, arm_d),
        8,
    );
    a.extend([pt(waist, length), pt(0.0, length)]);
    a
}
fn sleeve(m: Measurements, coat: bool) -> Vec<Point> {
    let arm = (m.chest_mm + m.ease_mm) / 3.25 + if coat { 32.0 } else { 0.0 };
    let cap = (m.chest_mm * 0.115).clamp(105.0, 170.0);
    let wrist = m.hand_mm * 1.5 + if coat { 30.0 } else { 0.0 };
    let mut a = vec![pt(0.0, cap)];
    curve(
        &mut a,
        pt(0.0, cap),
        pt(arm * 0.12, cap * 0.12),
        pt(arm * 0.5, 0.0),
        6,
    );
    curve(
        &mut a,
        pt(arm * 0.5, 0.0),
        pt(arm * 0.86, cap * 0.12),
        pt(arm, cap),
        6,
    );
    a.extend([
        pt(arm * 0.5 + wrist * 0.5, m.sleeve_mm),
        pt(arm * 0.5 - wrist * 0.5, m.sleeve_mm),
    ]);
    a
}
fn trousers(m: Measurements) -> Vec<Piece> {
    let rise = (m.inseam_mm * 0.33).max(210.0);
    let length = m.inseam_mm + rise;
    let w = (m.hip_mm + m.ease_mm) / 4.0;
    let hem = w * 0.64;
    let front = vec![
        pt(0.0, 18.0),
        pt(w * 0.91, 0.0),
        pt(w + 7.0, rise * 0.48),
        pt(w + 15.0, rise),
        pt(w * 0.76 + hem * 0.17, length),
        pt(w * 0.76 - hem * 0.83, length),
        pt(w * 0.25, rise + 65.0),
        pt(-w * 0.15, rise),
        pt(-w * 0.03, rise * 0.4),
    ];
    let back = vec![
        pt(0.0, 10.0),
        pt(w * 1.08, -35.0),
        pt(w * 1.15, rise * 0.45),
        pt(w * 1.25, rise),
        pt(w * 0.81 + hem * 0.30, length),
        pt(w * 0.81 - hem * 0.74, length),
        pt(w * 0.19, rise + 64.0),
        pt(-w * 0.29, rise),
        pt(-w * 0.03, rise * 0.4),
    ];
    let band = (m.waist_mm + m.ease_mm) * 0.55;
    vec![
        piece(
            "Vorderhose",
            front,
            2,
            false,
            "Crotch shaping and darts require fitting",
        ),
        piece("Hinterhose", back, 2, false, "Back rise needs fitting"),
        piece(
            "Bund",
            vec![pt(0.0, 0.0), pt(band, 0.0), pt(band, 90.0), pt(0.0, 90.0)],
            2,
            false,
            "Straight draft waistband",
        ),
    ]
}
fn glove(m: Measurements) -> Vec<Piece> {
    let width = m.hand_mm * 0.55;
    let h = m.hand_length_mm;
    let mut contour = vec![pt(0.0, h), pt(0.0, h * 0.31)];
    for (i, height) in [0.29, 0.115, 0.06, 0.19].into_iter().enumerate() {
        let x0 = i as f64 * width / 4.0;
        let x1 = (i + 1) as f64 * width / 4.0;
        let tip = h * height;
        contour.push(pt(x0 + 2.0, tip + 5.0));
        curve(
            &mut contour,
            pt(x0 + 2.0, tip + 5.0),
            pt((x0 + x1) * 0.5, tip - 12.0),
            pt(x1 - 2.0, tip + 5.0),
            5,
        );
        if i < 3 {
            contour.push(pt(x1 - 1.0, h * 0.4));
        }
    }
    contour.push(pt(width, h));
    let thumb = vec![
        pt(0.0, 45.0),
        pt(m.hand_mm * 0.12, 0.0),
        pt(m.hand_mm * 0.24, 45.0),
        pt(m.hand_mm * 0.21, h * 0.53),
        pt(m.hand_mm * 0.03, h * 0.53),
    ];
    vec![
        piece(
            "Handflaeche",
            contour.clone(),
            2,
            false,
            "Finger lengths are rough estimates",
        ),
        piece(
            "Handruecken",
            contour,
            2,
            false,
            "Finger gussets are missing",
        ),
        piece("Daumen", thumb, 2, false, "Thumb gusset is illustrative"),
    ]
}
fn beanie(m: Measurements) -> Vec<Piece> {
    let circumference = m.head_mm + m.ease_mm * 0.1;
    let width = circumference / 4.0;
    let height = (m.head_mm * 0.32).clamp(165.0, 260.0);
    let mut wedge = vec![
        pt(0.0, height),
        pt(width, height),
        pt(width * 0.93, height * 0.6),
    ];
    curve(
        &mut wedge,
        pt(width * 0.93, height * 0.6),
        pt(width * 0.85, height * 0.1),
        pt(width * 0.5, 0.0),
        8,
    );
    curve(
        &mut wedge,
        pt(width * 0.5, 0.0),
        pt(width * 0.13, height * 0.1),
        pt(width * 0.07, height * 0.6),
        8,
    );
    vec![
        piece("Muetzensegment", wedge, 4, false, "Four wedge panels"),
        piece(
            "Buendchen",
            vec![
                pt(0.0, 0.0),
                pt(circumference * 0.5, 0.0),
                pt(circumference * 0.5, 75.0),
                pt(0.0, 75.0),
            ],
            2,
            false,
            "Folded knit cuff",
        ),
    ]
}
fn circle(cx: f64, cy: f64, radius: f64, segments: usize) -> Vec<Point> {
    (0..segments)
        .map(|i| {
            let angle = i as f64 * 2.0 * PI / segments as f64;
            pt(cx + radius * angle.cos(), cy + radius * angle.sin())
        })
        .collect()
}
fn hat(m: Measurements) -> Vec<Piece> {
    let circumference = m.head_mm + m.ease_mm * 0.12;
    let side = circumference / 6.0;
    let radius = circumference / (2.0 * PI);
    let mut crown = vec![pt(0.0, 135.0), pt(side, 135.0), pt(side * 0.88, 48.0)];
    curve(
        &mut crown,
        pt(side * 0.88, 48.0),
        pt(side * 0.65, -10.0),
        pt(side * 0.5, 0.0),
        4,
    );
    curve(
        &mut crown,
        pt(side * 0.5, 0.0),
        pt(side * 0.35, -10.0),
        pt(side * 0.12, 48.0),
        4,
    );
    let mut brim = piece(
        "Krempe",
        circle(radius + 70.0, radius + 70.0, radius + 70.0, 64),
        1,
        false,
        "Brim with a central opening",
    );
    brim.holes
        .push(circle(radius + 70.0, radius + 70.0, radius, 64));
    vec![
        piece(
            "Kronensegment",
            crown,
            6,
            false,
            "Six panels, approximate dome",
        ),
        piece(
            "Hutdeckel",
            circle(radius, radius, radius, 48),
            1,
            false,
            "Check top-to-side seam length",
        ),
        brim,
    ]
}
fn skirt(m: Measurements) -> Vec<Piece> {
    let waist = (m.waist_mm + m.ease_mm) / 4.0;
    let hip = (m.hip_mm + m.ease_mm) / 4.0;
    let length = m.back_mm * 0.87;
    let panel = vec![
        pt(0.0, 0.0),
        pt(waist, 0.0),
        pt(hip, 205.0),
        pt(hip + 35.0, length),
        pt(0.0, length),
    ];
    let band = (m.waist_mm + m.ease_mm) * 0.5;
    vec![
        piece("Rock vorne", panel.clone(), 1, true, "Center cut on fold"),
        piece("Rock hinten", panel, 1, true, "Back darts omitted"),
        piece(
            "Bund",
            vec![pt(0.0, 0.0), pt(band, 0.0), pt(band, 90.0), pt(0.0, 90.0)],
            2,
            false,
            "Waistband",
        ),
    ]
}

pub fn draft(kind: Garment, m: Measurements) -> Result<Vec<Piece>, String> {
    m.validate()?;
    Ok(match kind {
        Garment::Top => vec![
            piece(
                "Vorderteil",
                bodice(m, true, false),
                1,
                true,
                "Neck/armhole are approximations",
            ),
            piece(
                "Rueckenteil",
                bodice(m, false, false),
                1,
                true,
                "Center cut on fold",
            ),
            piece(
                "Aermel",
                sleeve(m, false),
                2,
                false,
                "Sleeve cap requires fitting",
            ),
        ],
        Garment::Jacket => vec![
            piece(
                "Jacke vorne",
                bodice(m, true, true),
                2,
                false,
                "Front fastening not drafted",
            ),
            piece(
                "Jacke hinten",
                bodice(m, false, true),
                1,
                true,
                "Center cut on fold",
            ),
            piece(
                "Jackenaermel",
                sleeve(m, true),
                2,
                false,
                "Sleeve design concept",
            ),
            piece(
                "Kragen",
                vec![
                    pt(0.0, 0.0),
                    pt(m.chest_mm * 0.38, 0.0),
                    pt(m.chest_mm * 0.38, 95.0),
                    pt(0.0, 95.0),
                ],
                1,
                false,
                "Placeholder straight collar",
            ),
        ],
        Garment::Trousers => trousers(m),
        Garment::Glove => glove(m),
        Garment::Beanie => beanie(m),
        Garment::Hat => hat(m),
        Garment::Skirt => skirt(m),
    })
}

fn area(poly: &[Point]) -> f64 {
    poly.iter()
        .enumerate()
        .map(|(i, a)| {
            let b = poly[(i + 1) % poly.len()];
            a.x * b.y - b.x * a.y
        })
        .sum::<f64>()
        * 0.5
}
fn offset(poly: &[Point], distance: f64) -> Result<Vec<Point>, String> {
    if distance == 0.0 {
        return Ok(poly.to_vec());
    }
    let sign = area(poly).signum();
    let mut lines = Vec::with_capacity(poly.len());
    for (i, a) in poly.iter().enumerate() {
        let b = poly[(i + 1) % poly.len()];
        let dx = b.x - a.x;
        let dy = b.y - a.y;
        let len = dx.hypot(dy);
        if len < 1e-7 {
            return Err("Zero length polygon edge".into());
        }
        let nx = sign * dy / len * distance;
        let ny = -sign * dx / len * distance;
        lines.push((pt(a.x + nx, a.y + ny), pt(b.x + nx, b.y + ny)));
    }
    let mut result = Vec::with_capacity(poly.len());
    for i in 0..lines.len() {
        let prev = lines[(i + lines.len() - 1) % lines.len()];
        let curr = lines[i];
        let rx = prev.1.x - prev.0.x;
        let ry = prev.1.y - prev.0.y;
        let sx = curr.1.x - curr.0.x;
        let sy = curr.1.y - curr.0.y;
        let cross = rx * sy - ry * sx;
        let point = if cross.abs() < 1e-8 {
            curr.0
        } else {
            let qx = curr.0.x - prev.0.x;
            let qy = curr.0.y - prev.0.y;
            let t = (qx * sy - qy * sx) / cross;
            let hit = pt(prev.0.x + t * rx, prev.0.y + t * ry);
            if (hit.x - poly[i].x).hypot(hit.y - poly[i].y) > (5.0 * distance).max(30.0) {
                pt((curr.0.x + prev.1.x) * 0.5, (curr.0.y + prev.1.y) * 0.5)
            } else {
                hit
            }
        };
        if !point.x.is_finite() || !point.y.is_finite() {
            return Err("Non-finite offset".into());
        }
        result.push(point);
    }
    Ok(result)
}
fn svg_points(poly: &[Point]) -> String {
    let mut s = String::new();
    for p in poly {
        let _ = write!(s, "{:.3},{:.3} ", p.x, p.y);
    }
    s
}

#[derive(Clone, Debug)]
pub struct GarmentExport {
    pub svg: String,
    pub parts: Vec<Piece>,
    pub pieces_to_cut: u32,
    pub sewn_area_mm2: f64,
    pub cut_area_mm2: f64,
    pub estimated_mass_g: f64,
}
pub fn export_svg(
    kind: Garment,
    m: Measurements,
    seam_mm: f64,
    density_g_m2: f64,
) -> Result<GarmentExport, String> {
    if !seam_mm.is_finite() || !(0.0..=80.0).contains(&seam_mm) {
        return Err("Seam allowance must be between 0 and 80 mm".into());
    }
    if !density_g_m2.is_finite() || !(1.0..=5000.0).contains(&density_g_m2) {
        return Err("Areal density must be between 1 and 5000 g/m2".into());
    }
    let parts = draft(kind, m)?;
    let mut shapes = String::new();
    let mut sewn = 0.0;
    let mut cut = 0.0;
    let mut x = 18.0;
    let mut y = 22.0;
    let mut row_height: f64 = 0.0;
    let mut max_width: f64 = 0.0;
    for part in &parts {
        let outer = offset(&part.outline, seam_mm)?;
        let points = part
            .outline
            .iter()
            .chain(outer.iter())
            .chain(part.holes.iter().flatten());
        let (mut min_x, mut min_y, mut max_x, mut max_y) = (
            f64::INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NEG_INFINITY,
        );
        for p in points {
            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);
            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
        }
        let w = max_x - min_x;
        let h = max_y - min_y;
        if x + w > 1450.0 && x > 18.0 {
            x = 18.0;
            y += row_height + 48.0;
            row_height = 0.0;
        }
        let _ = writeln!(
            shapes,
            r#"<g transform="translate({:.3} {:.3})">"#,
            x - min_x,
            y - min_y
        );
        let _ = writeln!(
            shapes,
            r##"<polygon fill="#203b4b" stroke="#efa16f" stroke-width="1" points="{}"/>"##,
            svg_points(&outer)
        );
        let _ = writeln!(
            shapes,
            r##"<polygon fill="none" stroke="#72d7db" stroke-dasharray="4 2" stroke-width="0.9" points="{}"/>"##,
            svg_points(&part.outline)
        );
        for hole in &part.holes {
            let _ = writeln!(
                shapes,
                r##"<polygon fill="#14202e" stroke="#efa16f" stroke-width="0.9" points="{}"/>"##,
                svg_points(hole)
            );
        }
        let _ = writeln!(
            shapes,
            r##"<text x="{:.3}" y="{:.3}" font-size="13" fill="#d4e5f3">{} · {}x{}</text>"##,
            min_x,
            min_y - 6.0,
            part.name,
            part.copies,
            if part.on_fold { " Stoffbruch" } else { "" }
        );
        shapes.push_str("</g>\n");
        let holes_area = part.holes.iter().map(|h| area(h).abs()).sum::<f64>();
        sewn += (area(&part.outline).abs() - holes_area) * f64::from(part.copies);
        cut += (area(&outer).abs() - holes_area) * f64::from(part.copies);
        x += w + 46.0;
        row_height = row_height.max(h + 28.0);
        max_width = max_width.max(x);
    }
    let width = max_width.clamp(600.0, 1550.0);
    let height = y + row_height + 35.0;
    let qty = parts.iter().map(|part| part.copies).sum();
    let mass = cut * density_g_m2 / 1_000_000.0;
    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width:.3} {height:.3}" width="{width:.3}mm" height="{height:.3}mm" data-pieces="{qty}" data-sewn-area="{sewn:.3}" data-cut-area="{cut:.3}" data-mass="{mass:.3}"><rect width="100%" height="100%" fill="#14202e"/>{shapes}<text x="18" y="{:.3}" fill="#edb36d" font-size="12">KONZEPT-GRUNDSCHNITT: Nicht ohne Anprobe zuschneiden</text></svg>"##,
        height - 11.0,
    );
    Ok(GarmentExport {
        svg,
        parts,
        pieces_to_cut: qty,
        sewn_area_mm2: sewn,
        cut_area_mm2: cut,
        estimated_mass_g: mass,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_seven_blocks_render() {
        for kind in Garment::ALL {
            let out = export_svg(kind, Measurements::default(), 10.0, 210.0).unwrap();
            assert!(out.parts.len() >= 2, "{}", kind.name());
            assert!(out.cut_area_mm2 > out.sewn_area_mm2, "{}", kind.name());
            assert!(out.svg.contains("<svg"));
            assert!(!out.svg.contains("NaN"));
        }
    }
    #[test]
    fn body_size_affects_cut_pattern() {
        let mut m = Measurements::default();
        let original = export_svg(Garment::Trousers, m, 10.0, 210.0).unwrap();
        m.set_cm("hip", 120.0).unwrap();
        let enlarged = export_svg(Garment::Trousers, m, 10.0, 210.0).unwrap();
        assert!(enlarged.cut_area_mm2 > original.cut_area_mm2);
    }
    #[test]
    fn material_weight_scales_with_density() {
        let a = export_svg(Garment::Top, Measurements::default(), 10.0, 200.0).unwrap();
        let b = export_svg(Garment::Top, Measurements::default(), 10.0, 400.0).unwrap();
        assert!((b.estimated_mass_g / a.estimated_mass_g - 2.0).abs() < 1e-9);
    }
    #[test]
    fn invalid_values_rejected() {
        let mut m = Measurements::default();
        assert!(m.set_cm("head", f64::NAN).is_err());
        assert!(m.set_cm("unknown", 25.0).is_err());
        assert!(export_svg(Garment::Hat, m, -4.0, 210.0).is_err());
    }
}

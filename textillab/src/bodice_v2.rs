//! TextilLab VT/RT bodice v2 construction -- technical drafting concept.
//! Anatomical inputs are in millimetres. The resulting SVG is not an
//! industrially verified sewing pattern and needs fitting before cutting.
use crate::garments::{area, offset, svg_points, GarmentExport, Measurements, Piece};
use crate::Point;
use std::fmt::Write;

#[derive(Clone, Debug)]
pub struct BodicePanel {
    pub piece: Piece,
    pub svg_path: String,
    pub waist_dart: [Point; 4],
    pub shoulder_dart_legs: [Point; 2],
    pub shoulder_dart_apex: Point,
    pub bust_point: Point,
    pub waist_y: f64,
    pub bust_y: f64,
    pub hip_y: f64,
    pub hem_y: f64,
    pub grain_x: f64,
}

struct Shape {
    outline: Vec<Point>,
    path: String,
    current: Point,
}
impl Shape {
    fn start(x: f64, y: f64) -> Self {
        Self {
            outline: vec![Point::new(x, y)],
            path: format!("M {x:0.3} {y:0.3}"),
            current: Point::new(x, y),
        }
    }
    fn line(&mut self, point: Point) {
        if (self.current.x - point.x).hypot(self.current.y - point.y) > 1e-8 {
            self.outline.push(point);
            let _ = write!(self.path, " L {:0.3} {:0.3}", point.x, point.y);
        }
        self.current = point;
    }
    fn cubic(&mut self, b: Point, c: Point, d: Point, steps: usize) {
        let a = self.current;
        for i in 1..=steps {
            let t = i as f64 / steps as f64;
            let u = 1.0 - t;
            self.outline.push(Point::new(
                u * u * u * a.x + 3.0 * u * u * t * b.x + 3.0 * u * t * t * c.x + t * t * t * d.x,
                u * u * u * a.y + 3.0 * u * u * t * b.y + 3.0 * u * t * t * c.y + t * t * t * d.y,
            ));
        }
        let _ = write!(
            self.path,
            " C {:0.3} {:0.3} {:0.3} {:0.3} {:0.3} {:0.3}",
            b.x, b.y, c.x, c.y, d.x, d.y
        );
        self.current = d;
    }
    fn close(&mut self) {
        self.path.push_str(" Z");
    }
}

fn draft_panel(m: Measurements, front: bool) -> BodicePanel {
    let waist_y = 475.0;
    let top_y = waist_y
        - if front {
            m.front_length_mm
        } else {
            m.back_waist_mm
        };
    let bust_y = waist_y - m.front_length_mm + m.bust_height_mm;
    // Shared front/back underarm balance line; different body lengths must not shift the seam.
    let arm_y = waist_y - m.front_length_mm + m.armhole_depth_mm;
    let hip_y = waist_y + m.waist_to_hip_mm;
    let hem_y = waist_y + m.hem_below_waist_mm;
    let chest_w = (m.chest_mm + m.ease_mm) / 4.0 * if front { 1.008 } else { 0.992 };
    let waist_dart_w = if front {
        (m.chest_mm * 0.029).max(20.0)
    } else {
        (m.chest_mm * 0.025).max(16.0)
    };
    let waist_side = (m.waist_mm + m.ease_mm) / 4.0 + waist_dart_w / 2.0;
    let hip_side = (m.hip_mm + m.ease_mm) / 4.0;
    let neck_w = m.neck_mm / 5.75;
    let neck_d = if front {
        m.neck_mm / 5.4 + 13.0
    } else {
        m.neck_mm / 17.0 + 7.0
    };
    let shoulder_x = (m.shoulder_mm / 2.0).clamp(neck_w + 90.0, chest_w - 17.0);
    let slope = m.shoulder_slope_mm;
    let mut shape = Shape::start(0.0, top_y + neck_d);
    shape.cubic(
        Point::new(neck_w * 0.28, top_y + neck_d),
        Point::new(neck_w * 0.66, top_y + neck_d * 0.03),
        Point::new(neck_w, top_y),
        24,
    );
    shape.line(Point::new(shoulder_x, top_y + slope));
    shape.cubic(
        Point::new(shoulder_x + 8.0, top_y + slope + (arm_y - top_y - slope) * 0.12),
        Point::new(chest_w - 47.0, arm_y - 12.0),
        Point::new(chest_w, arm_y),
        30,
    );
    shape.cubic(
        Point::new(chest_w + 1.0, arm_y + 24.0),
        Point::new(chest_w + 2.0, bust_y - 15.0),
        Point::new(chest_w, bust_y),
        12,
    );
    shape.cubic(
        Point::new(chest_w - 7.0, bust_y + 72.0),
        Point::new(waist_side + 8.0, waist_y - 60.0),
        Point::new(waist_side, waist_y),
        24,
    );
    shape.cubic(
        Point::new(waist_side - 6.0, waist_y + 63.0),
        Point::new(hip_side - 6.0, hip_y - 55.0),
        Point::new(hip_side, hip_y),
        24,
    );
    shape.cubic(
        Point::new(hip_side, hip_y + 20.0),
        Point::new(hip_side, hem_y - 35.0),
        Point::new(hip_side, hem_y),
        14,
    );
    shape.line(Point::new(0.0, hem_y));
    shape.close();
    let bust_x = (chest_w * 0.53).min(m.bust_span_mm / 2.0);
    let bust_point = Point::new(bust_x, bust_y);
    let dart_x = if front {
        bust_x
    } else {
        (chest_w * 0.54).min(m.bust_span_mm * 0.56)
    };
    let down = (hip_y - waist_y) * 0.8;
    let lower_dart = [
        Point::new(dart_x, waist_y - if front { 145.0 } else { 165.0 }),
        Point::new(dart_x + waist_dart_w / 2.0, waist_y),
        Point::new(dart_x, waist_y + down.min(170.0)),
        Point::new(dart_x - waist_dart_w / 2.0, waist_y),
    ];
    let shoulder_dart_x = neck_w + (shoulder_x - neck_w) * if front { 0.51 } else { 0.57 };
    let shoulder_dart_w = if front {
        (m.chest_mm * 0.033).max(19.0)
    } else {
        (m.chest_mm * 0.014).max(10.0)
    };
    let a = shoulder_dart_x - shoulder_dart_w / 2.0;
    let b = shoulder_dart_x + shoulder_dart_w / 2.0;
    let shoulder_y = |x: f64| top_y + (x - neck_w) / (shoulder_x - neck_w) * slope;
    BodicePanel {
        piece: Piece {
            name: if front {
                "VT – Vorderteil"
            } else {
                "RT – Rückenteil"
            },
            outline: shape.outline,
            holes: Vec::new(),
            copies: 1,
            on_fold: true,
            note: if front {
                "Center front, shoulder/bust dart, waist dart"
            } else {
                "Center back, shoulder dart, waist dart"
            },
        },
        svg_path: shape.path,
        waist_dart: lower_dart,
        shoulder_dart_legs: [Point::new(a, shoulder_y(a)), Point::new(b, shoulder_y(b))],
        shoulder_dart_apex: if front {
            Point::new(bust_x, bust_y - 12.0)
        } else {
            Point::new(shoulder_dart_x - 15.0, top_y + 165.0)
        },
        bust_point,
        waist_y,
        bust_y,
        hip_y,
        hem_y,
        grain_x: chest_w * 0.62,
    }
}

/// Drafts a front and back block with their own center-fold and dart marks.
pub fn draft(m: Measurements) -> Result<Vec<BodicePanel>, String> {
    m.validate()?;
    if m.neck_mm > m.shoulder_mm * 1.25 || m.bust_span_mm >= m.chest_mm * 0.75 {
        return Err("Invalid neck, shoulder or bust-point relationship".into());
    }
    Ok(vec![draft_panel(m, true), draft_panel(m, false)])
}

#[derive(Clone, Debug)]
pub struct SleevePanel {
    pub piece: Piece,
    pub svg_path: String,
    pub cap_y: f64,
    pub cuff_y: f64,
    pub grain_x: f64,
    pub notches: [Point; 2],
}

/// Two symmetric cut copies from a single, slightly asymmetric set-in sleeve.
/// The cap remains a fitting approximation until armhole seam lengths are balanced.
pub fn draft_sleeve(m: Measurements) -> Result<SleevePanel, String> {
    draft_sleeve_with_cap(m, None)
}

/// Override is the cap height in mm, used by the seam-length optimizer.
pub fn draft_sleeve_with_cap(m: Measurements, height_override: Option<f64>) -> Result<SleevePanel, String> {
    m.validate()?;
    if m.upper_arm_mm <= m.wrist_mm || m.upper_arm_mm > m.chest_mm * 0.8 {
        return Err("Upper arm and wrist circumference are inconsistent".into());
    }
    let bicep_width = m.upper_arm_mm + m.ease_mm * 0.25;
    let cuff_width = m.wrist_mm + (m.ease_mm * 0.22).max(45.0);
    let cap = height_override.unwrap_or((m.armhole_depth_mm * 0.60).clamp(105.0, 190.0));
    if !cap.is_finite() || !(60.0..=260.0).contains(&cap) {
        return Err("Invalid sleeve cap height".into());
    }
    let length = m.sleeve_mm;
    if length <= cap + 110.0 {
        return Err("Sleeve length is too short for the sleeve cap".into());
    }
    let center = bicep_width * 0.5;
    let mut shape = Shape::start(0.0, cap);
    shape.cubic(
        Point::new(bicep_width * 0.10, cap * 0.38),
        Point::new(center * 0.50, cap * 0.08),
        Point::new(center, 0.0),
        24,
    );
    shape.cubic(
        Point::new(center + bicep_width * 0.23, -1.0),
        Point::new(bicep_width * 0.93, cap * 0.50),
        Point::new(bicep_width, cap),
        24,
    );
    shape.cubic(
        Point::new(bicep_width * 0.94, cap + 120.0),
        Point::new(center + cuff_width * 0.55, length - 85.0),
        Point::new(center + cuff_width * 0.50, length),
        20,
    );
    shape.line(Point::new(center - cuff_width * 0.50, length));
    shape.cubic(
        Point::new(center - cuff_width * 0.55, length - 85.0),
        Point::new(bicep_width * 0.06, cap + 120.0),
        Point::new(0.0, cap),
        20,
    );
    shape.close();
    // The final cubic ends at the start, which the polygon loop already closes.
    shape.outline.pop();
    Ok(SleevePanel {
        piece: Piece {
            name: "Ärmel – langer Ärmel",
            outline: shape.outline,
            holes: Vec::new(),
            copies: 2,
            on_fold: false,
            note: "Two sleeves; sleeve cap seam length needs fitting",
        },
        svg_path: shape.path,
        cap_y: cap,
        cuff_y: length,
        grain_x: center,
        notches: [
            Point::new(bicep_width * 0.21, cap * 0.27),
            Point::new(bicep_width * 0.81, cap * 0.31),
        ],
    })
}

/// Fully enumerated garment pattern pieces: front, back, and two sleeves.
pub fn draft_parts(m: Measurements) -> Result<Vec<Piece>, String> {
    let panels = draft(m)?;
    let sleeve = draft_sleeve(m)?;
    let mut result: Vec<Piece> = panels.into_iter().map(|p| p.piece).collect();
    result.push(sleeve.piece);
    Ok(result)
}

fn render_sleeve(sleeve: &SleevePanel, dx: f64, dy: f64, seam_mm: f64) -> Result<String, String> {
    let mut s = format!("<g transform=\"translate({dx:.3} {dy:.3})\">\n");
    let _ = writeln!(
        s,
        "<path d=\"{}\" fill=\"#264554\" fill-opacity=\"0.22\" stroke=\"#75d3d6\" stroke-dasharray=\"4 2\" stroke-width=\"1.4\" stroke-linejoin=\"round\"/>",
        sleeve.svg_path
    );
    if seam_mm > 0.0 {
        let cut = offset(&sleeve.piece.outline, seam_mm)?;
        let _ = writeln!(
            s,
            "<polygon fill=\"none\" stroke=\"#efa16f\" stroke-width=\"1.6\" stroke-linejoin=\"round\" points=\"{}\"/>",
            svg_points(&cut)
        );
    }
    let center = sleeve.grain_x;
    let top = sleeve.cap_y + 80.0;
    let end = sleeve.cuff_y - 50.0;
    let _ = writeln!(
        s,
        "<path d=\"M {center:.3} {top:.3} V {end:.3}\" stroke=\"#68c6d5\" stroke-width=\"1.5\"/>"
    );
    let _ = writeln!(
        s,
        "<path d=\"M {:.3} {:.3} L {center:.3} {end:.3} L {:.3} {:.3}\" stroke=\"#68c6d5\" stroke-width=\"1.5\" fill=\"none\"/>",
        center-6.0,end-13.0,center+6.0,end-13.0
    );
    let width = sleeve.piece.outline.iter().map(|p| p.x).fold(0.0, f64::max);
    let cap = sleeve.cap_y;
    let _ = writeln!(
        s,
        "<path d=\"M -12 {cap:.3} H {:.3}\" stroke=\"#53768c\" stroke-dasharray=\"4 5\" stroke-width=\"0.65\" opacity=\"0.6\"/>",
        width+12.0
    );
    for (i, notch) in sleeve.notches.iter().enumerate() {
        let size = if i == 0 { 3.4 } else { 2.6 };
        let _ = writeln!(
            s,
            "<circle cx=\"{:.3}\" cy=\"{:.3}\" r=\"{size:.3}\" fill=\"#ef9f74\"/>",
            notch.x, notch.y
        );
    }
    let _ = writeln!(
        s,
        "<text x=\"{center:.3}\" y=\"{:.3}\" text-anchor=\"middle\" font-size=\"22\" font-weight=\"700\" fill=\"#dcebf3\">ÄR</text>",
        sleeve.cuff_y - 22.0
    );
    let middle = (sleeve.cap_y + sleeve.cuff_y) * 0.5;
    let tag_x = center + 24.0;
    let _ = writeln!(
        s,
        "<text x=\"{tag_x:.3}\" y=\"{middle:.3}\" text-anchor=\"middle\" font-size=\"12\" fill=\"#68c6d5\" transform=\"rotate(-90 {tag_x:.3} {middle:.3})\">FDL</text>"
    );
    s.push_str("</g>\n");
    Ok(s)
}

fn render_panel(
    panel: &BodicePanel,
    horizontal_offset: f64,
    mirrored: bool,
    seam_mm: f64,
) -> Result<String, String> {
    let mut s = String::new();
    let cx = if mirrored {
        panel
            .piece
            .outline
            .iter()
            .map(|p| p.x)
            .fold(f64::NEG_INFINITY, f64::max)
    } else {
        0.0
    };
    let tx = if mirrored {
        horizontal_offset + cx
    } else {
        horizontal_offset
    };
    let flip = if mirrored { " scale(-1 1)" } else { "" };
    let _ = writeln!(s, "<g transform=\"translate({tx:0.3} 0){flip}\">");
    let _=writeln!(s,"<path d=\"{}\" fill=\"#264554\" fill-opacity=\"0.22\" stroke=\"#75d3d6\" stroke-dasharray=\"4 2\" stroke-width=\"1.4\" stroke-linejoin=\"round\"/>",panel.svg_path);
    if seam_mm > 0.0 {
        let cut = offset(&panel.piece.outline, seam_mm)?;
        let _ = writeln!(
            s,
            "<polygon fill=\"none\" stroke=\"#efa16f\" stroke-width=\"1.6\" points=\"{}\"/>",
            svg_points(&cut)
        );
    }
    let _ = writeln!(
        s,
        "<polygon fill=\"none\" stroke=\"#f29a87\" stroke-width=\"1.3\" points=\"{}\"/>",
        svg_points(&panel.waist_dart)
    );
    for leg in &panel.shoulder_dart_legs {
        let end = panel.shoulder_dart_apex;
        let _=writeln!(s,"<path d=\"M {:0.3} {:0.3} L {:0.3} {:0.3}\" fill=\"none\" stroke=\"#f29a87\" stroke-width=\"1.4\"/>",
            leg.x,leg.y,end.x,end.y);
    }
    let top = panel.bust_y + 60.0;
    let bottom = panel.hem_y - 65.0;
    let x = panel.grain_x;
    let _=writeln!(s,"<path d=\"M {x:0.3} {top:0.3} V {bottom:0.3}\" fill=\"none\" stroke=\"#68c6d5\" stroke-width=\"1.5\"/>");
    let _=writeln!(s,"<path d=\"M {:0.3} {:0.3} L {x:0.3} {bottom:0.3} L {:0.3} {:0.3}\" fill=\"none\" stroke=\"#68c6d5\" stroke-width=\"2\"/>",
        x-6.0,bottom-13.0,x+6.0,bottom-13.0);
    let b = panel.bust_point;
    let _ = writeln!(
        s,
        "<circle cx=\"{:0.3}\" cy=\"{:0.3}\" r=\"2.6\" fill=\"#ef9f74\"/>",
        b.x, b.y
    );
    s.push_str("</g>\n");
    let label_x = if mirrored {
        horizontal_offset + cx * 0.5
    } else {
        horizontal_offset + 110.0
    };
    let label = if mirrored { "RT" } else { "VT" };
    let _=writeln!(s,"<text x=\"{label_x:0.3}\" y=\"{:0.3}\" text-anchor=\"middle\" font-size=\"24\" fill=\"#dcebf3\" font-weight=\"700\">{label}</text>",panel.hem_y-35.0);
    let label_x = label_x + 34.0;
    let label_y = panel.waist_y + 120.0;
    let _=writeln!(s,"<text x=\"{label_x:0.3}\" y=\"{label_y:0.3}\" font-size=\"13\" fill=\"#68c6d5\" transform=\"rotate(-90 {label_x:0.3} {label_y:0.3})\">FDL</text>");
    Ok(s)
}

pub fn export(m: Measurements, seam_mm: f64, density_g_m2: f64) -> Result<GarmentExport, String> {
    export_with_options(m, seam_mm, density_g_m2, None, true)
}

pub fn export_with_options(
    m: Measurements,
    seam_mm: f64,
    density_g_m2: f64,
    cap_height_override: Option<f64>,
    include_sleeves: bool,
) -> Result<GarmentExport, String> {
    if !seam_mm.is_finite() || !(0.0..=80.0).contains(&seam_mm) {
        return Err("Seam allowance must be between 0 and 80 mm".into());
    }
    if !density_g_m2.is_finite() || !(1.0..=5000.0).contains(&density_g_m2) {
        return Err("Material density must be between 1 and 5000 g/m2".into());
    }
    let panels = draft(m)?;
    let sleeve = draft_sleeve_with_cap(m, cap_height_override)?;
    let front_width = panels[0]
        .piece
        .outline
        .iter()
        .map(|p| p.x)
        .fold(0.0, f64::max);
    let back_width = panels[1]
        .piece
        .outline
        .iter()
        .map(|p| p.x)
        .fold(0.0, f64::max);
    let front_dx = 75.0;
    let back_dx = front_dx + front_width + 100.0;
    let sleeve_dx = back_dx + back_width + 100.0;
    let sleeve_dy = 95.0;
    let sleeve_width = sleeve.piece.outline.iter().map(|p| p.x).fold(0.0, f64::max);
    let width = if include_sleeves { sleeve_dx + sleeve_width + 95.0 } else {back_dx + back_width + 110.0};
    let height = panels
        .iter()
        .map(|p| p.hem_y)
        .fold(if include_sleeves {sleeve.cuff_y+sleeve_dy} else {0.0}, f64::max)
        + 90.0;
    let mut sewn = 0.0;
    let mut cut = 0.0;
    for panel in &panels {
        sewn += area(&panel.piece.outline).abs();
        cut += area(&offset(&panel.piece.outline, seam_mm)?).abs();
    }
    if include_sleeves {
        sewn += 2.0 * area(&sleeve.piece.outline).abs();
        cut += 2.0 * area(&offset(&sleeve.piece.outline, seam_mm)?).abs();
    }
    let mass = cut * density_g_m2 / 1_000_000.0;
    let count = if include_sleeves { 4 } else { 2 };
    let sleeves_count = if include_sleeves { 2 } else { 0 };
    let mut svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width:.3}mm\" height=\"{height:.3}mm\" viewBox=\"0 0 {width:.3} {height:.3}\" data-bodice-version=\"2\" data-pieces=\"{count}\" data-sleeves=\"{sleeves_count}\" data-sewn-area=\"{sewn:.3}\" data-cut-area=\"{cut:.3}\" data-mass=\"{mass:.3}\">\n"
    );
    svg.push_str("<rect width=\"100%\" height=\"100%\" fill=\"#14202e\"/>\n");
    for (label, y) in [
        ("Brust", panels[0].bust_y),
        ("Taille", panels[0].waist_y),
        ("Hüfte", panels[0].hip_y),
        ("Saum", panels[0].hem_y),
    ] {
        let _ = writeln!(
            svg,
            "<path d=\"M 20 {y:.3} H {:.3}\" stroke=\"#53768c\" stroke-width=\"0.65\" stroke-dasharray=\"4 5\" opacity=\"0.62\"/>",
            back_dx+back_width+25.0
        );
        let _ = writeln!(
            svg,
            "<text x=\"{:.3}\" y=\"{:.3}\" font-size=\"12\" fill=\"#a3b9c9\">{label}</text>",
            back_dx + back_width + 29.0,
            y - 6.0
        );
    }
    svg.push_str(&render_panel(&panels[0], front_dx, false, seam_mm)?);
    svg.push_str(&render_panel(&panels[1], back_dx, true, seam_mm)?);
    if include_sleeves {svg.push_str(&render_sleeve(&sleeve, sleeve_dx, sleeve_dy, seam_mm)?);}
    let _ = writeln!(
        svg,
        "<text x=\"{front_dx:.3}\" y=\"30\" font-size=\"17\" font-weight=\"700\" fill=\"#e4eff7\">TEXTILLAB · OBERTEIL · VT / RT / ÄRMEL</text>"
    );
    let _ = writeln!(
        svg,
        "<text x=\"{front_dx:.3}\" y=\"51\" font-size=\"12\" fill=\"#9bafc1\">Türkis: Nahtlinie · Orange: Zuschnitt · FDL / Abnäher / Referenzlinien</text>"
    );
    if include_sleeves {
    let _ = writeln!(
        svg,
        "<text x=\"{sleeve_dx:.3}\" y=\"{:.3}\" font-size=\"14\" font-weight=\"600\" fill=\"#dcebf3\">ÄR · 2× zuschneiden</text>",
        sleeve_dy-35.0
    );
    }
    let _ = writeln!(
        svg,
        "<text x=\"{front_dx:.3}\" y=\"{:.3}\" font-size=\"11\" fill=\"#9bafc1\">KONZEPT-GRUNDSCHNITT · Armkugellänge und Abnäher vor Fertigung prüfen</text>",
        height-24.0
    );
    svg.push_str("</svg>\n");
    Ok(GarmentExport {
        svg,
        parts: {
            let mut all: Vec<Piece> = panels.into_iter().map(|p| p.piece).collect();
            if include_sleeves {all.push(sleeve.piece);}
            all
        },
        pieces_to_cut: count,
        sewn_area_mm2: sewn,
        cut_area_mm2: cut,
        estimated_mass_g: mass,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn front_and_back_are_separate_and_have_real_curve_segments() {
        let result = draft(Measurements::default()).unwrap();
        assert_eq!(result.len(), 2);
        assert_ne!(result[0].piece.outline, result[1].piece.outline);
        for panel in result {
            assert!(panel.piece.outline.len() > 100);
            assert!(panel.svg_path.matches(" C ").count() >= 5);
            assert!(panel.waist_dart.len() == 4);
            assert!(panel.shoulder_dart_apex.y > panel.shoulder_dart_legs[0].y);
        }
    }
    #[test]
    fn render_has_technical_reference_guides() {
        let result = export(Measurements::default(), 10.0, 210.0).unwrap();
        for label in [
            "VT",
            "RT",
            "Brust",
            "Taille",
            "Hüfte",
            "Saum",
            "FDL",
            "data-bodice-version=\"2\"",
        ] {
            assert!(result.svg.contains(label), "{label}");
        }
        assert_eq!(result.pieces_to_cut, 4);
        assert_eq!(result.parts.len(), 3);
        assert!(result.svg.contains(r#"data-sleeves="2""#));
        assert!(result.svg.contains("#14202e"));
        assert!(result.cut_area_mm2 > result.sewn_area_mm2);
    }
    #[test]
    fn changing_body_measurements_recomputes_geometry() {
        let first = export(Measurements::default(), 10.0, 210.0).unwrap();
        let mut m = Measurements::default();
        m.set_cm("chest", 112.0).unwrap();
        m.set_cm("frontLength", 49.0).unwrap();
        let second = export(m, 10.0, 210.0).unwrap();
        assert!(second.cut_area_mm2 > first.cut_area_mm2);
        assert_ne!(first.svg, second.svg);
    }
    #[test]
    fn sleeves_are_included_and_react_to_user_measurements() {
        let default_measurements = Measurements::default();
        let sleeve = draft_sleeve(default_measurements).unwrap();
        assert_eq!(sleeve.piece.copies, 2);
        assert!(sleeve.piece.outline.len() > 70);
        assert!(sleeve.svg_path.matches(" C ").count() >= 4);
        assert_eq!(draft_parts(default_measurements).unwrap().len(), 3);
        let mut changed = default_measurements;
        changed.set_cm("sleeve", 70.0).unwrap();
        changed.set_cm("upperArm", 39.0).unwrap();
        changed.set_cm("wrist", 21.0).unwrap();
        assert_ne!(
            sleeve.piece.outline,
            draft_sleeve(changed).unwrap().piece.outline
        );
        assert!(
            export(changed, 10.0, 210.0).unwrap().cut_area_mm2
                > export(default_measurements, 10.0, 210.0)
                    .unwrap()
                    .cut_area_mm2
        );
    }
    #[test]
    fn invalid_bodice_measurements_rejected() {
        let mut m = Measurements::default();
        m.set_cm("bustHeight", 50.0).unwrap();
        assert!(draft(m).is_err());
        assert!(export(Measurements::default(), -1.0, 210.0).is_err());
    }
}

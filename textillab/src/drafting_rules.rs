//! TextilLab Experimental Construction System 1.0
//!
//! Structured own-house rules for base-body modification, named references,
//! lengthening/shortening and numerical sleeve-cap/armscye balance.
//!
//! IMPORTANT: Seam lengths are mathematical checks only. Body fitting, dart
//! closure, material behaviour, seam allowance intersections and original
//! proprietary cutting system specifications are NOT validated.

use crate::bodice_v2::{self, BodicePanel, SleevePanel};
use crate::garments::{GarmentExport, Measurements};
use crate::Point;
use std::fmt::Write;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fit {
    Fitted,
    Regular,
    Comfort,
    Loose,
    Oversize,
    Custom,
}
impl Fit {
    pub fn from_key(key: &str) -> Result<Self, String> {
        match key {
            "fitted" => Ok(Self::Fitted),
            "regular" => Ok(Self::Regular),
            "comfort" => Ok(Self::Comfort),
            "loose" => Ok(Self::Loose),
            "oversize" => Ok(Self::Oversize),
            "custom" => Ok(Self::Custom),
            _ => Err(format!("Unknown fit profile: {key}")),
        }
    }
    pub fn ease_mm(self) -> Option<f64> {
        match self {
            Self::Fitted => Some(40.0),
            Self::Regular => Some(80.0),
            Self::Comfort => Some(120.0),
            Self::Loose => Some(180.0),
            Self::Oversize => Some(240.0),
            Self::Custom => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
    Bodice,
    Shirt,
    Blouse,
    Jacket,
    Coat,
    Sleeveless,
}
impl Style {
    pub fn from_key(key: &str) -> Result<Self, String> {
        match key {
            "bodice" => Ok(Self::Bodice),
            "shirt" => Ok(Self::Shirt),
            "blouse" => Ok(Self::Blouse),
            "jacket" => Ok(Self::Jacket),
            "coat" => Ok(Self::Coat),
            "sleeveless" => Ok(Self::Sleeveless),
            _ => Err(format!("Unknown style: {key}")),
        }
    }
    pub fn hem_mm(self) -> f64 {
        match self {
            Self::Blouse => 60.0,
            Self::Jacket => 150.0,
            Self::Coat => 450.0,
            _ => 0.0,
        }
    }
    pub fn sleeves(self) -> bool {
        self != Self::Sleeveless
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Alterations {
    pub torso_mm: f64,
    pub hip_mm: f64,
    pub hem_mm: f64,
    pub sleeve_mm: f64,
}
impl Default for Alterations {
    fn default() -> Self {
        Self {
            torso_mm: 0.0,
            hip_mm: 0.0,
            hem_mm: 0.0,
            sleeve_mm: 0.0,
        }
    }
}
impl Alterations {
    fn validate(self) -> Result<(), String> {
        for (name, value) in [
            ("torso", self.torso_mm),
            ("hip", self.hip_mm),
            ("hem", self.hem_mm),
            ("sleeve", self.sleeve_mm),
        ] {
            if !value.is_finite() || !(-120.0..=350.0).contains(&value) {
                return Err(format!("Alteration {name} must be -120..350 mm"));
            }
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Settings {
    pub fit: Fit,
    pub style: Style,
    pub alterations: Alterations,
    pub target_cap_ease_mm: f64,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            fit: Fit::Regular,
            style: Style::Bodice,
            alterations: Alterations::default(),
            target_cap_ease_mm: 22.0,
        }
    }
}
impl Settings {
    fn validate(self) -> Result<(), String> {
        self.alterations.validate()?;
        if !self.target_cap_ease_mm.is_finite() || !(0.0..=45.0).contains(&self.target_cap_ease_mm)
        {
            return Err("Target sleeve ease must be 0..45 mm".into());
        }
        Ok(())
    }
}

/// Maps alterations to named, anatomically scoped dimensions. These are
/// redrafted in the existing point/Bezier system, not scaled raster geometry.
pub fn apply(m: Measurements, settings: Settings) -> Result<Measurements, String> {
    settings.validate()?;
    let mut m = m;
    if let Some(ease) = settings.fit.ease_mm() {
        m.ease_mm = ease;
    }
    m.front_length_mm += settings.alterations.torso_mm;
    m.back_waist_mm += settings.alterations.torso_mm;
    m.waist_to_hip_mm += settings.alterations.hip_mm;
    m.hem_below_waist_mm +=
        settings.alterations.hip_mm + settings.alterations.hem_mm + settings.style.hem_mm();
    m.sleeve_mm += settings.alterations.sleeve_mm;
    m.validate()?;
    Ok(m)
}
fn length(points: &[Point], start: usize, end: usize) -> Result<f64, String> {
    if start >= end || end >= points.len() {
        return Err("Incompatible sampling of pattern curves".into());
    }
    Ok(points[start..=end]
        .windows(2)
        .map(|p| (p[1].x - p[0].x).hypot(p[1].y - p[0].y))
        .sum())
}

#[derive(Clone, Debug)]
pub struct SeamReport {
    pub front_armhole_mm: f64,
    pub back_armhole_mm: f64,
    pub total_armhole_mm: f64,
    pub sleeve_cap_mm: f64,
    pub sleeve_ease_mm: f64,
    pub side_difference_mm: f64,
    pub cap_height_mm: f64,
    pub target_ease_mm: f64,
    pub reachable: bool,
}
pub fn seam_report(
    front: &BodicePanel,
    back: &BodicePanel,
    sleeve: &SleevePanel,
    target: f64,
    reachable: bool,
) -> Result<SeamReport, String> {
    let front_armhole = length(&front.piece.outline, 25, 55)?;
    let back_armhole = length(&back.piece.outline, 25, 55)?;
    let sleeve_cap = length(&sleeve.piece.outline, 0, 48)?;
    let fside = length(&front.piece.outline, 55, 129)?;
    let bside = length(&back.piece.outline, 55, 129)?;
    let total = front_armhole + back_armhole;
    Ok(SeamReport {
        front_armhole_mm: front_armhole,
        back_armhole_mm: back_armhole,
        total_armhole_mm: total,
        sleeve_cap_mm: sleeve_cap,
        sleeve_ease_mm: sleeve_cap - total,
        side_difference_mm: (fside - bside).abs(),
        cap_height_mm: sleeve.cap_y,
        target_ease_mm: target,
        reachable,
    })
}

pub fn balance_cap(
    m: Measurements,
    target_ease_mm: f64,
) -> Result<(SleevePanel, SeamReport), String> {
    let panels = bodice_v2::draft(m)?;
    let generate = |height: f64| -> Result<(SleevePanel, SeamReport), String> {
        let sleeve = bodice_v2::draft_sleeve_with_cap(m, Some(height))?;
        let report = seam_report(&panels[0], &panels[1], &sleeve, target_ease_mm, true)?;
        Ok((sleeve, report))
    };
    let (slo, lo) = generate(80.0)?;
    let (shi, hi) = generate(215.0)?;
    if target_ease_mm < lo.sleeve_ease_mm || target_ease_mm > hi.sleeve_ease_mm {
        let (s, mut r) = if (target_ease_mm - lo.sleeve_ease_mm).abs()
            < (target_ease_mm - hi.sleeve_ease_mm).abs()
        {
            (slo, lo)
        } else {
            (shi, hi)
        };
        r.reachable = false;
        return Ok((s, r));
    }
    let mut low = 80.0;
    let mut high = 215.0;
    for _ in 0..28 {
        let height = (low + high) * 0.5;
        let (sleeve, report) = generate(height)?;
        let difference = report.sleeve_ease_mm - target_ease_mm;
        if difference.abs() < 0.06 {
            return Ok((sleeve, report));
        }
        if difference > 0.0 {
            high = height;
        } else {
            low = height;
        }
    }
    generate((low + high) * 0.5)
}

#[derive(Clone, Debug)]
pub struct RuleValue {
    pub id: &'static str,
    pub name: &'static str,
    pub formula: &'static str,
    pub value_mm: f64,
}
pub fn rule_values(m: Measurements, seam: &SeamReport) -> Vec<RuleValue> {
    vec![
        RuleValue {
            id: "R01",
            name: "Brust-Teilungsweite",
            formula: "(Br+Mehrweite)/4",
            value_mm: (m.chest_mm + m.ease_mm) * 0.25,
        },
        RuleValue {
            id: "R02",
            name: "Taillen-Teilungsweite",
            formula: "(Ta+Mehrweite)/4",
            value_mm: (m.waist_mm + m.ease_mm) * 0.25,
        },
        RuleValue {
            id: "R03",
            name: "Hüft-Teilungsweite",
            formula: "(Hü+Mehrweite)/4",
            value_mm: (m.hip_mm + m.ease_mm) * 0.25,
        },
        RuleValue {
            id: "R04",
            name: "Armlochhöhe",
            formula: "Armlochtiefe",
            value_mm: m.armhole_depth_mm,
        },
        RuleValue {
            id: "R05",
            name: "Vorderlänge",
            formula: "Hals-Taille",
            value_mm: m.front_length_mm,
        },
        RuleValue {
            id: "R06",
            name: "Rückenlänge",
            formula: "Hals-Taille",
            value_mm: m.back_waist_mm,
        },
        RuleValue {
            id: "R07",
            name: "Ärmelkugel",
            formula: "Armloch+Einhalteweite",
            value_mm: seam.sleeve_cap_mm,
        },
        RuleValue {
            id: "R08",
            name: "Seitennaht-Differenz",
            formula: "|VT-RT|",
            value_mm: seam.side_difference_mm,
        },
    ]
}
fn point_svg(s: &mut String, x: f64, y: f64, name: &str) {
    let _ = writeln!(
        s,
        "<circle cx=\"{x:.3}\" cy=\"{y:.3}\" r=\"2.5\" fill=\"#7edbe2\"/>\
         <text x=\"{:.3}\" y=\"{:.3}\" fill=\"#c2dfe6\" font-size=\"10\">{name}</text>",
        x + 6.0,
        y - 6.0
    );
}
fn horizontal(s: &mut String, x0: f64, x1: f64, y: f64, color: &str) {
    let _ = writeln!(
        s,
        "<path d=\"M {x0:.3} {y:.3} H {x1:.3}\" fill=\"none\" stroke=\"{color}\" stroke-width=\"0.8\" stroke-dasharray=\"5 5\"/>"
    );
}
fn dimension_y(s: &mut String, x: f64, y0: f64, y1: f64, id: &str, value: f64) {
    let _ = writeln!(
        s,
        "<g class=\"dimension\" data-dimension=\"{id}\">\
         <path d=\"M {x:.3} {y0:.3} V {y1:.3}\" fill=\"none\" stroke=\"#abdbe9\" stroke-width=\"1\"/>\
         <text x=\"{:.3}\" y=\"{:.3}\" fill=\"#e9f2f7\" font-size=\"11\">{id} {value:.1} mm</text></g>",
        x + 10.0,
        (y0 + y1) * 0.5
    );
}
pub fn overlay_svg(
    src: &str,
    m: Measurements,
    seam: &SeamReport,
    include_sleeve: bool,
) -> Result<String, String> {
    let panels = bodice_v2::draft(m)?;
    let sleeve = bodice_v2::draft_sleeve_with_cap(m, Some(seam.cap_height_mm))?;
    let fw = panels[0]
        .piece
        .outline
        .iter()
        .map(|p| p.x)
        .fold(0.0, f64::max);
    let bw = panels[1]
        .piece
        .outline
        .iter()
        .map(|p| p.x)
        .fold(0.0, f64::max);
    let fx = 75.0;
    let bx = fx + fw + 100.0;
    let ax = bx + bw + 100.0;
    let ay = 95.0;
    let arw = sleeve.piece.outline.iter().map(|p| p.x).fold(0.0, f64::max);
    let mut o = String::from(
        "<g id=\"textillab-construction-overlay\" data-ruleset=\"textillab-experimental-1.0\">\n",
    );
    for (x, panel, front) in [(fx, &panels[0], true), (bx, &panels[1], false)] {
        let width = panel.piece.outline.iter().map(|p| p.x).fold(0.0, f64::max);
        let left = if front { x } else { x + width };
        let bust = panel.bust_y;
        for y in [bust, panel.waist_y, panel.hip_y, panel.hem_y] {
            horizontal(&mut o, x - 11.0, x + width + 11.0, y, "#426879");
        }
        let p = &panel.piece.outline;
        for (ix, name) in [
            (0, if front { "MV" } else { "MH" }),
            (24, "HN"),
            (25, "SP"),
            (55, "AL"),
        ] {
            let point = p[ix];
            let xat = if front {
                x + point.x
            } else {
                x + width - point.x
            };
            point_svg(&mut o, xat, point.y, name);
        }
        let bp = panel.bust_point;
        point_svg(
            &mut o,
            if front { x + bp.x } else { x + width - bp.x },
            bp.y,
            "BP",
        );
        let id = if front { "VL" } else { "RL" };
        let height = if front {
            m.front_length_mm
        } else {
            m.back_waist_mm
        };
        dimension_y(
            &mut o,
            if front { left - 39.0 } else { left + 39.0 },
            475.0 - height,
            475.0,
            id,
            height,
        );
        horizontal(
            &mut o,
            x + 8.0,
            x + width - 8.0,
            (bust + 475.0) * 0.5,
            "#c48ce0",
        );
        horizontal(
            &mut o,
            x + 8.0,
            x + width - 8.0,
            (475.0 + panel.hip_y) * 0.5,
            "#c48ce0",
        );
        horizontal(
            &mut o,
            x + 8.0,
            x + width - 8.0,
            (panel.hip_y + panel.hem_y) * 0.5,
            "#c48ce0",
        );
    }
    if include_sleeve {
        let cap = sleeve.cap_y + ay;
        let elbow = sleeve.cuff_y * 0.63 + ay;
        let hem = sleeve.cuff_y + ay;
        for (y, tag) in [(cap, "OB"), (elbow, "ELL"), (hem, "AS")] {
            horizontal(&mut o, ax - 10.0, ax + arw + 10.0, y, "#426879");
            let _ = writeln!(
                o,
                "<text x=\"{:.3}\" y=\"{:.3}\" font-size=\"10\" fill=\"#a9c4d3\">{tag}</text>",
                ax + arw + 12.0,
                y - 4.0
            );
        }
        point_svg(&mut o, ax + sleeve.grain_x, ay, "Ä0");
        point_svg(&mut o, ax + sleeve.grain_x, hem, "ÄS");
        dimension_y(&mut o, ax + arw + 31.0, ay, hem, "ÄL", m.sleeve_mm);
        horizontal(&mut o, ax + 8.0, ax + arw - 8.0, elbow, "#c48ce0");
    }
    let _=writeln!(o,
        "<text x=\"{ax:.3}\" y=\"{:.3}\" fill=\"#bad2df\" font-size=\"11\">AL {:.1} mm · AK {:.1} mm · EW {:.1} mm · Seitennaht Δ {:.1} mm</text>",
        ay+sleeve.cuff_y+38.0,seam.total_armhole_mm,seam.sleeve_cap_mm,seam.sleeve_ease_mm,seam.side_difference_mm
    );
    o.push_str("</g>\n");
    if let Some(index) = src.rfind("</svg>") {
        let mut output = src.to_string();
        output.insert_str(index, &o);
        Ok(output)
    } else {
        Err("SVG output is incomplete".into())
    }
}

#[derive(Clone, Debug)]
pub struct DraftResult {
    pub pattern: GarmentExport,
    pub report: SeamReport,
    pub rules: Vec<RuleValue>,
    pub measurements: Measurements,
}
pub fn generate(
    measurements: Measurements,
    settings: Settings,
    allowance_mm: f64,
    density_g_m2: f64,
) -> Result<DraftResult, String> {
    let m = apply(measurements, settings)?;
    let (_sleeve, report) = balance_cap(m, settings.target_cap_ease_mm)?;
    let mut pattern = bodice_v2::export_with_options(
        m,
        allowance_mm,
        density_g_m2,
        Some(report.cap_height_mm),
        settings.style.sleeves(),
    )?;
    pattern.svg = overlay_svg(&pattern.svg, m, &report, settings.style.sleeves())?;
    Ok(DraftResult {
        rules: rule_values(m, &report),
        pattern,
        report,
        measurements: m,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fit_and_style_change_actual_bodice_dimensions() {
        let m = Measurements::default();
        let base = generate(m, Settings::default(), 10.0, 210.0).unwrap();
        let settings = Settings {
            fit: Fit::Loose,
            style: Style::Coat,
            alterations: Alterations {
                torso_mm: 20.0,
                hip_mm: 30.0,
                hem_mm: 40.0,
                sleeve_mm: 25.0,
            },
            ..Settings::default()
        };
        let longer = generate(m, settings, 10.0, 210.0).unwrap();
        assert!(longer.pattern.cut_area_mm2 > base.pattern.cut_area_mm2);
        assert_eq!(longer.measurements.ease_mm, 180.0);
        assert!((longer.measurements.front_length_mm - 480.0).abs() < 1e-8);
        assert!((longer.measurements.hem_below_waist_mm - 810.0).abs() < 1e-8);
    }
    #[test]
    fn sleeve_cap_is_balanced_numerically_to_target() {
        let out = generate(Measurements::default(), Settings::default(), 10.0, 210.0).unwrap();
        assert!(out.report.reachable);
        assert!((out.report.sleeve_ease_mm - 22.0).abs() < 0.1);
        assert!(out.report.side_difference_mm < 5.0);
        assert!(out.pattern.svg.contains("data-ruleset="));
        assert_eq!(out.rules.len(), 8);
    }
    #[test]
    fn sleeveless_variant_removes_sleeve_panels_and_mass() {
        let base = generate(Measurements::default(), Settings::default(), 10.0, 210.0).unwrap();
        let sleeveless = generate(
            Measurements::default(),
            Settings {
                style: Style::Sleeveless,
                ..Settings::default()
            },
            10.0,
            210.0,
        )
        .unwrap();
        assert_eq!(sleeveless.pattern.pieces_to_cut, 2);
        assert_eq!(sleeveless.pattern.parts.len(), 2);
        assert!(sleeveless.pattern.cut_area_mm2 < base.pattern.cut_area_mm2);
    }
    #[test]
    fn alteration_limits_are_enforced() {
        let bad = Settings {
            alterations: Alterations {
                torso_mm: 400.0,
                ..Alterations::default()
            },
            ..Settings::default()
        };
        assert!(generate(Measurements::default(), bad, 10.0, 210.0).is_err());
        let nan = Settings {
            target_cap_ease_mm: f64::NAN,
            ..Settings::default()
        };
        assert!(generate(Measurements::default(), nan, 10.0, 210.0).is_err());
    }
}

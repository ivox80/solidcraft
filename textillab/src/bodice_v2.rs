//! TextilLab VT/RT bodice v2 construction -- technical drafting concept.
//! Anatomical inputs are in millimetres. The resulting SVG is not an
//! industrially verified sewing pattern and needs fitting before cutting.
use crate::Point;
use crate::garments::{GarmentExport, Measurements, Piece, area, offset, svg_points};
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
            path: format!("M {x:.3} {y:.3}"),
            current: Point::new(x, y),
        }
    }
    fn line(&mut self, point: Point) {
        if (self.current.x-point.x).hypot(self.current.y-point.y)>1e-8 {
            self.outline.push(point);
            let _ = write!(self.path, " L {:.3} {:.3}", point.x, point.y);
        }
        self.current = point;
    }
    fn cubic(&mut self, b: Point, c: Point, d: Point, steps: usize) {
        let a = self.current;
        for i in 1..=steps {
            let t = i as f64 / steps as f64;
            let u = 1.0 - t;
            self.outline.push(Point::new(
                u*u*u*a.x+3.0*u*u*t*b.x+3.0*u*t*t*c.x+t*t*t*d.x,
                u*u*u*a.y+3.0*u*u*t*b.y+3.0*u*t*t*c.y+t*t*t*d.y,
            ));
        }
        let _=write!(self.path," C {:.3} {:.3} {:.3} {:.3} {:.3} {:.3}",b.x,b.y,c.x,c.y,d.x,d.y);
        self.current=d;
    }
    fn close(&mut self) {
        self.path.push_str(" Z");
    }
}

fn draft_panel(m: Measurements, front: bool) -> BodicePanel {
    let waist_y = 475.0;
    let top_y = waist_y - if front {m.front_length_mm} else {m.back_waist_mm};
    let bust_y = waist_y - m.front_length_mm + m.bust_height_mm;
    let arm_y = top_y + m.armhole_depth_mm;
    let hip_y = waist_y+m.waist_to_hip_mm;
    let hem_y = waist_y+m.hem_below_waist_mm;
    let chest_w = (m.chest_mm+m.ease_mm)/4.0 * if front {1.008}else{0.992};
    let waist_dart_w = if front {(m.chest_mm*0.029).max(20.0)}else{(m.chest_mm*0.025).max(16.0)};
    let waist_side=(m.waist_mm+m.ease_mm)/4.0+waist_dart_w/2.0;
    let hip_side=(m.hip_mm+m.ease_mm)/4.0;
    let neck_w=m.neck_mm/5.75;
    let neck_d= if front {m.neck_mm/5.4+13.0}else{m.neck_mm/17.0+7.0};
    let shoulder_x=(m.shoulder_mm/2.0).clamp(neck_w+90.0,chest_w-17.0);
    let slope=m.shoulder_slope_mm;
    let mut shape=Shape::start(0.0,top_y+neck_d);
    shape.cubic(
        Point::new(neck_w*.28,top_y+neck_d),
        Point::new(neck_w*.66,top_y+neck_d*.03),
        Point::new(neck_w,top_y),24
    );
    shape.line(Point::new(shoulder_x,top_y+slope));
    shape.cubic(
        Point::new(shoulder_x+8.0,top_y+slope+arm_y*.12),
        Point::new(chest_w-47.0,arm_y-12.0),
        Point::new(chest_w,arm_y),30
    );
    shape.cubic(
        Point::new(chest_w+1.0,arm_y+24.0),
        Point::new(chest_w+2.0,bust_y-15.0),
        Point::new(chest_w,bust_y),12
    );
    shape.cubic(
        Point::new(chest_w-7.0,bust_y+72.0),
        Point::new(waist_side+8.0,waist_y-60.0),
        Point::new(waist_side,waist_y),24
    );
    shape.cubic(
        Point::new(waist_side-6.0,waist_y+63.0),
        Point::new(hip_side-6.0,hip_y-55.0),
        Point::new(hip_side,hip_y),24
    );
    shape.cubic(
        Point::new(hip_side,hip_y+20.0),
        Point::new(hip_side,hem_y-35.0),
        Point::new(hip_side,hem_y),14
    );
    shape.line(Point::new(0.0,hem_y));
    shape.close();
    let bust_x=(chest_w*.53).min(m.bust_span_mm/2.0);
    let bust_point=Point::new(bust_x,bust_y);
    let dart_x=if front {bust_x}else{(chest_w*.54).min(m.bust_span_mm*.56)};
    let down=(hip_y-waist_y)*.8;
    let lower_dart= [
        Point::new(dart_x,waist_y-if front {145.0}else{165.0}),
        Point::new(dart_x+waist_dart_w/2.0,waist_y),
        Point::new(dart_x,waist_y+down.min(170.0)),
        Point::new(dart_x-waist_dart_w/2.0,waist_y),
    ];
    let shoulder_dart_x=neck_w+(shoulder_x-neck_w)*if front {.51}else{.57};
    let shoulder_dart_w=if front {(m.chest_mm*.033).max(19.0)}else{(m.chest_mm*.014).max(10.0)};
    let a=shoulder_dart_x-shoulder_dart_w/2.0;
    let b=shoulder_dart_x+shoulder_dart_w/2.0;
    let shoulder_y=|x:f64| top_y+(x-neck_w)/(shoulder_x-neck_w)*slope;
    BodicePanel {
        piece:Piece {
            name:if front {"VT – Vorderteil"}else{"RT – Rückenteil"},
            outline:shape.outline,
            holes:Vec::new(),
            copies:1,
            on_fold:true,
            note:if front {"Center front, shoulder/bust dart, waist dart"}
                 else {"Center back, shoulder dart, waist dart"},
        },
        svg_path:shape.path,
        waist_dart:lower_dart,
        shoulder_dart_legs:[Point::new(a,shoulder_y(a)),Point::new(b,shoulder_y(b))],
        shoulder_dart_apex:if front {Point::new(bust_x,bust_y-12.0)}
                               else {Point::new(shoulder_dart_x-15.0,top_y+165.0)},
        bust_point,
        waist_y,bust_y,hip_y,hem_y,
        grain_x:chest_w*.62,
    }
}

/// Drafts a front and back block with their own center-fold and dart marks.
pub fn draft(m: Measurements) -> Result<Vec<BodicePanel>, String> {
    m.validate()?;
    if m.neck_mm>m.shoulder_mm*1.25 || m.bust_span_mm>=m.chest_mm*.75 {
        return Err("Invalid neck, shoulder or bust-point relationship".into());
    }
    Ok(vec![draft_panel(m,true),draft_panel(m,false)])
}

fn render_panel(panel:&BodicePanel,offset:f64,mirrored:bool,seam_mm:f64) -> Result<String,String> {
    let mut s=String::new();
    let cx=if mirrored {panel.piece.outline.iter().map(|p|p.x).fold(f64::NEG_INFINITY,f64::max)}else{0.0};
    let tx=if mirrored {offset+cx}else{offset};
    let flip=if mirrored {" scale(-1 1)"}else{""};
    let _=writeln!(s,"<g transform=\"translate({tx:.3} 0){flip}\">");
    let _=writeln!(s,"<path d=\"{}\" fill=\"#cb6953\" fill-opacity=\".04\" stroke=\"#a85145\" stroke-width=\"1.8\" stroke-linejoin=\"round\"/>",panel.svg_path);
    if seam_mm>0.0 {
        let cut=offset(&panel.piece.outline,seam_mm)?;
        let _=writeln!(s,"<polygon fill=\"none\" stroke=\"#e5a16c\" stroke-width=\".85\" stroke-dasharray=\"5 3\" points=\"{}\"/>",svg_points(&cut));
    }
    let _=writeln!(s,"<polygon fill=\"none\" stroke=\"#b9594d\" stroke-width=\"1.3\" points=\"{}\"/>",svg_points(&panel.waist_dart));
    for leg in &panel.shoulder_dart_legs {
        let end=panel.shoulder_dart_apex;
        let _=writeln!(s,"<path d=\"M {:.3} {:.3} L {:.3} {:.3}\" fill=\"none\" stroke=\"#b9594d\" stroke-width=\"1.4\"/>",
            leg.x,leg.y,end.x,end.y);
    }
    let top=panel.bust_y+60.0;
    let bottom=panel.hem_y-65.0;
    let x=panel.grain_x;
    let _=writeln!(s,"<path d=\"M {x:.3} {top:.3} V {bottom:.3}\" fill=\"none\" stroke=\"#4d8c9d\" stroke-width=\"1.5\"/>");
    let _=writeln!(s,"<path d=\"M {:.3} {:.3} L {x:.3} {bottom:.3} L {:.3} {:.3}\" fill=\"none\" stroke=\"#4d8c9d\" stroke-width=\"2\"/>",
        x-6.0,bottom-13.0,x+6.0,bottom-13.0);
    let b=panel.bust_point;
    let _=writeln!(s,"<circle cx=\"{:.3}\" cy=\"{:.3}\" r=\"2.6\" fill=\"#8d4037\"/>",b.x,b.y);
    s.push_str("</g>\n");
    let label_x=if mirrored {offset+cx*.5}else{offset+110.0};
    let label=if mirrored {"RT"}else{"VT"};
    let _=writeln!(s,"<text x=\"{label_x:.3}\" y=\"{:.3}\" text-anchor=\"middle\" font-size=\"24\" fill=\"#304a53\" font-weight=\"700\">{label}</text>",panel.hem_y-35.0);
    let label_x=label_x+34.0;
    let label_y=panel.waist_y+120.0;
    let _=writeln!(s,"<text x=\"{label_x:.3}\" y=\"{label_y:.3}\" font-size=\"13\" fill=\"#4d8c9d\" transform=\"rotate(-90 {label_x:.3} {label_y:.3})\">FDL</text>");
    Ok(s)
}

pub fn export(m:Measurements,seam_mm:f64,density_g_m2:f64)->Result<GarmentExport,String>{
    if !seam_mm.is_finite() || !(0.0..=80.0).contains(&seam_mm) {
        return Err("Seam allowance must be between 0 and 80 mm".into());
    }
    if !density_g_m2.is_finite() || !(1.0..=5000.0).contains(&density_g_m2) {
        return Err("Material density must be between 1 and 5000 g/m2".into());
    }
    let panels=draft(m)?;
    let back_width=panels[1].piece.outline.iter().map(|p|p.x).fold(f64::NEG_INFINITY,f64::max);
    let width=515.0+back_width+115.0;
    let height=panels.iter().map(|p|p.hem_y).fold(0.0,f64::max)+80.0;
    let mut sewn=0.0;
    let mut cut=0.0;
    for panel in &panels {
        sewn+=area(&panel.piece.outline).abs();
        cut+=area(&offset(&panel.piece.outline,seam_mm)?).abs();
    }
    let mass=cut*density_g_m2/1_000_000.0;
    let mut svg=format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width:.3}mm\" height=\"{height:.3}mm\" viewBox=\"0 0 {width:.3} {height:.3}\" data-bodice-version=\"2\" data-pieces=\"2\" data-sewn-area=\"{sewn:.3}\" data-cut-area=\"{cut:.3}\" data-mass=\"{mass:.3}\">\n"
    );
    svg.push_str("<rect width=\"100%\" height=\"100%\" fill=\"#d9c4a5\"/>\n");
    for (label,y) in [
        ("Brust",panels[0].bust_y),("Taille",panels[0].waist_y),
        ("Hüfte",panels[0].hip_y),("Saum",panels[0].hem_y),
    ] {
        let _=writeln!(svg,"<path d=\"M 20 {y:.3} H {:.3}\" stroke=\"#776b5c\" stroke-width=\".65\" opacity=\".48\"/>",width-70.0);
        let _=writeln!(svg,"<text x=\"{:.3}\" y=\"{:.3}\" font-size=\"13\" fill=\"#52616c\">{label}</text>",width-60.0,y-5.0);
    }
    svg.push_str(&render_panel(&panels[0],75.0,false,seam_mm)?);
    svg.push_str(&render_panel(&panels[1],515.0,true,seam_mm)?);
    svg.push_str("<text x=\"75\" y=\"29\" font-size=\"15\" font-weight=\"700\" fill=\"#3b454e\">TEXTILLAB · VT / RT GRUNDSCHNITT v2</text>\n");
    svg.push_str("<text x=\"75\" y=\"49\" font-size=\"11\" fill=\"#62584e\">Abnäher, Hilfslinien &amp; Fadenlauf · Maße in mm · keine freigegebene Fertigungsvorlage</text>\n</svg>\n");
    Ok(GarmentExport {
        svg,
        parts:panels.into_iter().map(|p|p.piece).collect(),
        pieces_to_cut:2,
        sewn_area_mm2:sewn,
        cut_area_mm2:cut,
        estimated_mass_g:mass,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn front_and_back_are_separate_and_have_real_curve_segments() {
        let result=draft(Measurements::default()).unwrap();
        assert_eq!(result.len(),2);
        assert_ne!(result[0].piece.outline,result[1].piece.outline);
        for panel in result {
            assert!(panel.piece.outline.len()>100);
            assert!(panel.svg_path.matches(" C ").count()>=5);
            assert!(panel.waist_dart.len()==4);
            assert!(panel.shoulder_dart_apex.y > panel.shoulder_dart_legs[0].y);
        }
    }
    #[test]
    fn render_has_technical_reference_guides() {
        let result=export(Measurements::default(),10.0,210.0).unwrap();
        for label in ["VT","RT","Brust","Taille","Hüfte","Saum","FDL","data-bodice-version=\"2\""] {
            assert!(result.svg.contains(label),"{label}");
        }
        assert_eq!(result.pieces_to_cut,2);
        assert!(result.cut_area_mm2>result.sewn_area_mm2);
    }
    #[test]
    fn changing_body_measurements_recomputes_geometry() {
        let first=export(Measurements::default(),10.0,210.0).unwrap();
        let mut m=Measurements::default();
        m.set_cm("chest",112.0).unwrap();
        m.set_cm("frontLength",49.0).unwrap();
        let second=export(m,10.0,210.0).unwrap();
        assert!(second.cut_area_mm2>first.cut_area_mm2);
        assert_ne!(first.svg,second.svg);
    }
    #[test]
    fn invalid_bodice_measurements_rejected() {
        let mut m=Measurements::default();
        m.set_cm("bustHeight",50.0).unwrap();
        assert!(draft(m).is_err());
        assert!(export(Measurements::default(),-1.0,210.0).is_err());
    }
}

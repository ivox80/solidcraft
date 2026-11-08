//! TextilLab v1.1 – independently designed prototype pants drafting rules.
//!
//! Uses named construction parameters, sampled cubic curves and seam-length
//! diagnostics. Does NOT reproduce or claim to implement Müller & Sohn rules.
//! Actual fit, dart closure, seam allowances and production use require review.

use crate::garments::{self, GarmentExport, Measurements, Piece};
use crate::Point;
use std::fmt::Write;

fn pt(x: f64, y: f64) -> Point {
    Point::new(x, y)
}
fn dist(a: Point, b: Point) -> f64 {
    (a.x - b.x).hypot(a.y - b.y)
}
fn path_length(poly: &[Point], begin: usize, end: usize) -> f64 {
    poly[begin..=end].windows(2).map(|p| dist(p[0], p[1])).sum()
}
#[derive(Clone, Copy, Debug)]
pub struct PantsMeasures {
    pub waist_mm: f64,
    pub hip_mm: f64,
    pub inseam_mm: f64,
    pub outseam_mm: f64,
    pub crotch_depth_mm: f64,
    pub waist_to_hip_mm: f64,
    pub waist_to_knee_mm: f64,
    pub thigh_mm: f64,
    pub knee_mm: f64,
    pub hem_mm: f64,
    pub hip_ease_mm: f64,
    pub waist_ease_mm: f64,
    pub dart_intake_mm: f64,
    pub crotch_extension_mm: f64,
}
impl Default for PantsMeasures {
    fn default() -> Self {
        Self {
            waist_mm:840.0,hip_mm:1020.0,inseam_mm:800.0,outseam_mm:1080.0,
            crotch_depth_mm:280.0,waist_to_hip_mm:200.0,waist_to_knee_mm:600.0,
            thigh_mm:640.0,knee_mm:480.0,hem_mm:440.0,
            hip_ease_mm:40.0,waist_ease_mm:20.0,dart_intake_mm:20.0,
            crotch_extension_mm:0.0,
        }
    }
}
impl PantsMeasures {
    pub fn from_general(m: Measurements) -> Self {
        Self {
            waist_mm:m.waist_mm,hip_mm:m.hip_mm,inseam_mm:m.inseam_mm,
            waist_to_hip_mm:m.waist_to_hip_mm,..Self::default()
        }
    }
    pub fn set_cm(&mut self,key:&str,cm:f64)->Result<(),String>{
        if !cm.is_finite() {return Err(format!("Invalid measure: {key}"));}
        let mm=cm*10.0;
        match key {
            "waist"=>self.waist_mm=mm,
            "hip"=>self.hip_mm=mm,
            "inseam"=>self.inseam_mm=mm,
            "outseam"=>self.outseam_mm=mm,
            "crotchDepth"=>self.crotch_depth_mm=mm,
            "waistToHip"=>self.waist_to_hip_mm=mm,
            "waistToKnee"=>self.waist_to_knee_mm=mm,
            "thigh"=>self.thigh_mm=mm,
            "knee"=>self.knee_mm=mm,
            "hem"=>self.hem_mm=mm,
            "hipEase"=>self.hip_ease_mm=mm,
            "waistEase"=>self.waist_ease_mm=mm,
            "dartIntake"=>self.dart_intake_mm=mm,
            "crotchExtension"=>self.crotch_extension_mm=mm,
            _=>return Err(format!("Unknown pants measure: {key}"))
        }
        Ok(())
    }
    pub fn validate(self)->Result<(),String>{
        for (name,x) in [
            ("waist",self.waist_mm),("hip",self.hip_mm),
            ("inseam",self.inseam_mm),("outseam",self.outseam_mm),
            ("crotchDepth",self.crotch_depth_mm),("waistToHip",self.waist_to_hip_mm),
            ("waistToKnee",self.waist_to_knee_mm),("thigh",self.thigh_mm),
            ("knee",self.knee_mm),("hem",self.hem_mm)
        ] {
            if !x.is_finite() || !(100.0..=2000.0).contains(&x) {
                return Err(format!("Pants body measure {name} must be 100..2000 mm"));
            }
        }
        for (name,x) in [
            ("hipEase",self.hip_ease_mm),("waistEase",self.waist_ease_mm),
            ("dartIntake",self.dart_intake_mm)
        ]{
            if !x.is_finite() || !(0.0..=130.0).contains(&x) {
                return Err(format!("Pants allowance {name} must be 0..130 mm"));
            }
        }
        if !self.crotch_extension_mm.is_finite() ||
            !(-20.0..=70.0).contains(&self.crotch_extension_mm) {
            return Err("Crotch extension must be -20..70 mm".into());
        }
        let rise=self.outseam_mm-self.inseam_mm;
        if !(190.0..=450.0).contains(&rise)
            || (rise-self.crotch_depth_mm).abs()>50.0 {
            return Err("Incompatible crotch depth, outside and inside leg lengths".into());
        }
        if self.waist_to_hip_mm>=rise-35.0
            || self.waist_to_knee_mm<=rise+95.0
            || self.waist_to_knee_mm>=self.outseam_mm-110.0 {
            return Err("Incompatible hip, crotch, knee and hem levels".into());
        }
        if self.hip_mm<self.waist_mm*.67 || self.waist_mm>self.hip_mm*1.35
            || self.knee_mm<self.hem_mm*.70
            || self.knee_mm>self.thigh_mm*1.2 {
            return Err("Inconsistent waist, hip, thigh, knee or ankle ratios".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug)]
pub struct Levels {
    pub hip: f64,
    pub rise: f64,
    pub knee: f64,
    pub hem: f64,
}
#[derive(Clone, Debug)]
pub struct Seams {
    pub side_mm:f64,
    pub inseam_mm:f64,
    pub rise_mm:f64,
    pub waist_mm:f64,
}
#[derive(Clone, Debug)]
pub struct PantsPanel {
    pub piece:Piece,
    pub svg_path:String,
    pub id:&'static str,
    pub levels:Levels,
    pub seams:Seams,
    pub marks:Vec<(&'static str,Point)>,
    pub darts:Vec<Point>,
}
#[derive(Clone, Debug)]
pub struct Checks {
    pub side_diff_mm:f64,
    pub inseam_diff_mm:f64,
}
#[derive(Clone, Debug)]
pub struct RuleRecord{
    pub id:&'static str,
    pub name:&'static str,
    pub formula:&'static str,
    pub value_mm:f64,
    pub source:&'static str
}
#[derive(Clone, Debug)]
pub struct PantsDraft {
    pub panels:[PantsPanel;2],
    pub checks:Checks,
    pub rules:Vec<RuleRecord>
}
struct Curve {
    poly:Vec<Point>,path:String
}
impl Curve {
    fn start(a:Point)->Self {
        Self{poly:vec![a],path:format!("M {:.3} {:.3}",a.x,a.y)}
    }
    fn line(&mut self,b:Point){
        if dist(*self.poly.last().expect("initialised"),b)>1e-8 {
            self.poly.push(b);
        }
        let _=write!(self.path," L {:.3} {:.3}",b.x,b.y);
    }
    fn cubic(&mut self,b:Point,c:Point,d:Point,steps:usize) {
        let a=*self.poly.last().expect("initialised");
        for i in 1..=steps {
            let t=i as f64/steps as f64;
            let u=1.0-t;
            self.poly.push(pt(
                u*u*u*a.x+3.0*u*u*t*b.x+3.0*u*t*t*c.x+t*t*t*d.x,
                u*u*u*a.y+3.0*u*u*t*b.y+3.0*u*t*t*c.y+t*t*t*d.y,
            ));
        }
        let _=write!(self.path," C {:.3} {:.3} {:.3} {:.3} {:.3} {:.3}",
            b.x,b.y,c.x,c.y,d.x,d.y);
    }
    fn close(&mut self){
        self.path.push_str(" Z");
        // Final Bezier terminates at first vertex; polygon closure is implicit.
        self.poly.pop();
    }
}
fn panel(m:PantsMeasures,back:bool)->PantsPanel {
    let factor=if back {1.02}else{0.98};
    let hip_w=(m.hip_mm+m.hip_ease_mm)/4.0*factor;
    let waist_q=(m.waist_mm+m.waist_ease_mm)/4.0;
    let dart=if back {m.dart_intake_mm}else{0.0};
    let drafted_waist=waist_q*if back {1.02}else{0.98}+dart;
    let wh=drafted_waist*0.5;
    let hh=hip_w*0.5;
    let rise=m.outseam_mm-m.inseam_mm;
    let hy=m.waist_to_hip_mm;
    let ky=m.waist_to_knee_mm;
    let ey=m.outseam_mm;
    let ext=(m.hip_mm+m.hip_ease_mm)/20.0
        +if back {30.0}else{0.0}+m.crotch_extension_mm;
    let kw=m.knee_mm/2.0+if back {10.0}else{-10.0};
    let hem_w=m.hem_mm/2.0+if back {10.0}else{-10.0};
    let a=pt(-wh,if back {-26.0}else{0.0});
    let b=pt(wh,if back {5.0}else{8.0});
    let side_hip=pt(hh,hy);
    let center_hip=pt(-hh,hy);
    let side_rise=pt(hh+if back {15.0}else{12.0},rise);
    let inner_rise=pt(-hh-ext,rise);
    let side_knee=pt(kw/2.0,ky);
    let inner_knee=pt(-kw/2.0,ky);
    let side_hem=pt(hem_w/2.0,ey);
    let inner_hem=pt(-hem_w/2.0,ey);

    let mut c=Curve::start(a);
    c.line(b);
    let side_start=c.poly.len()-1;
    c.cubic(pt(b.x+9.0,60.0),pt(side_hip.x-4.0,hy-35.0),side_hip,18);
    c.cubic(pt(side_hip.x+3.0,hy+35.0),pt(side_rise.x+5.0,rise-40.0),side_rise,16);
    c.cubic(pt(side_rise.x-10.0,rise+105.0),pt(side_knee.x+15.0,ky-90.0),side_knee,24);
    c.line(side_hem);
    let side_end=c.poly.len()-1;
    c.line(inner_hem);
    let inseam_start=c.poly.len()-1;
    c.line(inner_knee);
    c.cubic(pt(inner_knee.x-6.0,ky-95.0),
            pt(inner_rise.x+if back {57.0}else{35.0},rise+105.0),inner_rise,24);
    let inseam_end=c.poly.len()-1;
    c.cubic(pt(inner_rise.x+if back {18.0}else{21.0},rise-4.0),
        pt(center_hip.x-if back {35.0}else{17.0},hy+53.0),center_hip,32);
    c.cubic(pt(center_hip.x+if back {0.0}else{3.0},hy-45.0),
        pt(a.x-if back {15.0}else{0.0},55.0),a,18);
    c.close();
    let waist_dart_x=(a.x+b.x)/2.0+25.0;
    let darts=if back {
        vec![pt(waist_dart_x-dart/2.0,-8.0),
             pt(waist_dart_x,110.0),
             pt(waist_dart_x+dart/2.0,-8.0)]
    }else{Vec::new()};
    let seams=Seams{
        side_mm:path_length(&c.poly,side_start,side_end),
        inseam_mm:path_length(&c.poly,inseam_start,inseam_end),
        rise_mm:path_length(&c.poly,inseam_end,c.poly.len()-1)+
            dist(*c.poly.last().expect("nonempty"),a),
        waist_mm:path_length(&c.poly,0,side_start)-dart,
    };
    PantsPanel{
        piece:Piece{
            name:if back {"Hinterhose"}else{"Vorderhose"},
            outline:c.poly,holes:Vec::new(),copies:2,on_fold:false,
            note:if back {"Back crotch curve and waist dart: fitting required"}
                 else {"Front crotch curve, knee and grainline"},
        },
        svg_path:c.path,
        id:if back {"HH"}else{"VH"},
        levels:Levels{hip:hy,rise,knee:ky,hem:ey},
        seams,
        marks:vec![
            ("WC",a),("WS",b),("HS",side_hip),("HC",center_hip),
            ("CR",inner_rise),("RS",side_rise),("KS",side_knee),
            ("KI",inner_knee),("AS",side_hem),("AI",inner_hem),
        ],
        darts
    }
}
pub fn draft(m:PantsMeasures)->Result<PantsDraft,String>{
    m.validate()?;
    let front=panel(m,false);
    let back=panel(m,true);
    let checks=Checks{
        side_diff_mm:(front.seams.side_mm-back.seams.side_mm).abs(),
        inseam_diff_mm:(front.seams.inseam_mm-back.seams.inseam_mm).abs()
    };
    let rise=m.outseam_mm-m.inseam_mm;
    let first=(m.hip_mm+m.hip_ease_mm)/20.0+m.crotch_extension_mm;
    let rules=vec![
        RuleRecord{id:"H01",name:"Hüftviertel",formula:"(Hüftumfang+Zugabe)/4",value_mm:(m.hip_mm+m.hip_ease_mm)/4.0,source:"textillab.pants.experimental.1"},
        RuleRecord{id:"H02",name:"Taillenviertel",formula:"(Taillenumfang+Zugabe)/4",value_mm:(m.waist_mm+m.waist_ease_mm)/4.0,source:"textillab.pants.experimental.1"},
        RuleRecord{id:"H03",name:"Leibhöhe",formula:"Seitenlänge-Schrittlänge",value_mm:rise,source:"textillab.pants.experimental.1"},
        RuleRecord{id:"H04",name:"Vordere Schrittausladung",formula:"(Hüftumfang+Zugabe)/20+Änderung",value_mm:first,source:"textillab.pants.experimental.1"},
        RuleRecord{id:"H05",name:"Hintere Schrittausladung",formula:"Vorne+30 mm",value_mm:first+30.0,source:"textillab.pants.experimental.1"},
        RuleRecord{id:"H06",name:"Knie vorne",formula:"Knieumfang/2-10 mm",value_mm:m.knee_mm/2.0-10.0,source:"textillab.pants.experimental.1"},
        RuleRecord{id:"H07",name:"Knie hinten",formula:"Knieumfang/2+10 mm",value_mm:m.knee_mm/2.0+10.0,source:"textillab.pants.experimental.1"},
        RuleRecord{id:"H08",name:"Seitennähte Δ",formula:"|VH-HH|",value_mm:checks.side_diff_mm,source:"measured"},
        RuleRecord{id:"H09",name:"Innenbeinnähte Δ",formula:"|VH-HH|",value_mm:checks.inseam_diff_mm,source:"measured"},
        RuleRecord{id:"H10",name:"Abnäher hinten",formula:"eigene Formungsreserve",value_mm:m.dart_intake_mm,source:"textillab.pants.experimental.1"}
    ];
    Ok(PantsDraft{panels:[front,back],checks,rules})
}
/// SVG technical plot with controlled source attribution and seam-length checks.
pub fn export(m:PantsMeasures,seam_mm:f64,density_g_m2:f64)->Result<GarmentExport,String>{
    if !seam_mm.is_finite() || !(0.0..=80.0).contains(&seam_mm){
        return Err("Seam allowance must be 0..80 mm".into());
    }
    if !density_g_m2.is_finite() || !(1.0..=5000.0).contains(&density_g_m2){
        return Err("Areal density must be 1..5000 g/m2".into());
    }
    let d=draft(m)?;
    let mut drawing=String::new();
    let mut x=70.0;
    let mut sewn=0.0;
    let mut cut=0.0;
    let mut max_height:f64=0.0;
    let mut total_width=70.0;
    // A compact bounds pass before drawing.
    let mut bounds=Vec::new();
    for part in &d.panels {
        let outside=garments::offset(&part.piece.outline,seam_mm)?;
        let (mut xmin,mut xmax,mut ymin,mut ymax)=(
            f64::INFINITY,f64::NEG_INFINITY,f64::INFINITY,f64::NEG_INFINITY);
        for point in part.piece.outline.iter().chain(outside.iter()){
            xmin=xmin.min(point.x);xmax=xmax.max(point.x);
            ymin=ymin.min(point.y);ymax=ymax.max(point.y);
        }
        max_height=max_height.max(ymax-ymin);
        total_width+=xmax-xmin+130.0;
        bounds.push((outside,xmin,xmax,ymin,ymax));
    }
    let width=total_width+70.0;
    let height=max_height+175.0;
    let _=writeln!(drawing,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {width:.3} {height:.3}\" width=\"{width:.3}mm\" height=\"{height:.3}mm\" data-pants-system=\"textillab.pants.experimental.1\" data-pieces=\"4\">");
    drawing.push_str("<rect width=\"100%\" height=\"100%\" fill=\"#14202e\"/>\n");
    drawing.push_str("<text x=\"35\" y=\"24\" fill=\"#d9e6ef\" font-size=\"16\">TEXTILLAB · HOSEN-GRUNDSCHNITT · VH / HH</text>\n");
    for (panel,(outside,xmin,xmax,ymin,_ymax)) in d.panels.iter().zip(bounds.iter()){
        let dx=x-xmin;let dy=65.0-ymin;
        let _=writeln!(drawing,"<g transform=\"translate({dx:.3} {dy:.3})\">");
        let _=writeln!(drawing,
            "<path d=\"{}\" fill=\"#264554\" fill-opacity=\".36\" stroke=\"#7cdae1\" stroke-width=\"1.7\" stroke-dasharray=\"4 3\"/>",panel.svg_path);
        if seam_mm>0.0 {
            let _=writeln!(drawing,
                "<polygon fill=\"none\" stroke=\"#f4a670\" stroke-width=\"1.15\" points=\"{}\"/>",
                garments::svg_points(outside));
        }
        for (label,y) in [
            ("Hü",panel.levels.hip),("Sch",panel.levels.rise),
            ("Kn",panel.levels.knee),("Sa",panel.levels.hem)
        ]{
            let _=writeln!(drawing,
                "<path d=\"M {:.3} {y:.3} H {:.3}\" stroke=\"#4b7892\" stroke-width=\".8\" stroke-dasharray=\"5 5\"/>",
                xmin-15.0,xmax+15.0);
            let _=writeln!(drawing,
                "<text x=\"{:.3}\" y=\"{:.3}\" fill=\"#a8bed0\" font-size=\"10\">{label}</text>",
                xmax+17.0,y-5.0);
        }
        let y0=panel.levels.rise+48.0;
        let y1=panel.levels.hem-40.0;
        let _=writeln!(drawing,
            "<path d=\"M 0 {y0:.3} V {y1:.3}\" stroke=\"#d4d37c\" stroke-width=\"1.3\" stroke-dasharray=\"10 5\"/>");
        let _=writeln!(drawing,
            "<text x=\"9\" y=\"{:.3}\" fill=\"#d4d37c\" font-size=\"11\">FDL</text>",(y0+y1)*.5);
        if !panel.darts.is_empty() {
            let _=writeln!(drawing,
                "<polyline fill=\"none\" stroke=\"#fa9fa3\" stroke-width=\"1.6\" points=\"{}\"/>",
                garments::svg_points(&panel.darts));
        }
        for (tag,p) in &panel.marks {
            let _=writeln!(drawing,
                "<circle cx=\"{:.3}\" cy=\"{:.3}\" r=\"2.5\" fill=\"#80d9e0\"/>\
                 <text x=\"{:.3}\" y=\"{:.3}\" fill=\"#bedce2\" font-size=\"9\">{tag}</text>",
                p.x,p.y,p.x+5.0,p.y-5.0);
        }
        drawing.push_str("</g>\n");
        let _=writeln!(drawing,
            "<text x=\"{:.3}\" y=\"59\" text-anchor=\"middle\" fill=\"#e0edf7\" font-size=\"13\">{} · {} · 2×</text>",
            x+(xmax-xmin)*.5,panel.id,panel.piece.name);
        sewn+=garments::area(&panel.piece.outline).abs()*2.0;
        cut+=garments::area(outside).abs()*2.0;
        x+=xmax-xmin+130.0;
    }
    let mass=cut*density_g_m2/1_000_000.0;
    let _=writeln!(drawing,
        "<text x=\"35\" y=\"{:.3}\" fill=\"#a4bacb\" font-size=\"10\">KONZEPT-GRUNDSCHNITT · Müller &amp; Sohn-Regeln noch nicht validiert · Seitennaht Δ {:.1} mm · Innenbein Δ {:.1} mm</text>",
        height-20.0,d.checks.side_diff_mm,d.checks.inseam_diff_mm);
    drawing.push_str("</svg>\n");
    drawing=drawing.replace("data-pieces=\"4\"",
        &format!("data-pieces=\"4\" data-sewn-area=\"{sewn:.3}\" data-cut-area=\"{cut:.3}\" data-mass=\"{mass:.3}\""));
    Ok(GarmentExport{
        svg:drawing,
        parts:d.panels.into_iter().map(|p|p.piece).collect(),
        pieces_to_cut:4,sewn_area_mm2:sewn,
        cut_area_mm2:cut,estimated_mass_g:mass
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn separate_front_back_and_technical_points() {
        let d=draft(PantsMeasures::default()).unwrap();
        assert_eq!(d.panels.len(),2);
        assert_eq!(d.rules.len(),10);
        assert_ne!(d.panels[0].piece.outline,d.panels[1].piece.outline);
        assert!(d.panels[0].piece.outline.len()>100);
        assert_eq!(d.panels[0].piece.copies,2);
        assert!(!d.panels[1].darts.is_empty());
        assert!(d.panels[1].seams.rise_mm>d.panels[0].seams.rise_mm);
        assert!(d.checks.side_diff_mm<6.0);
        assert!(d.checks.inseam_diff_mm<12.0);
    }
    #[test]
    fn changing_body_dimensions_recomputes_curves() {
        let mut m=PantsMeasures::default();
        let a=export(m,10.0,210.0).unwrap();
        m.set_cm("hip",116.0).unwrap();
        m.set_cm("waistToKnee",62.0).unwrap();
        let b=export(m,10.0,210.0).unwrap();
        assert_ne!(a.svg,b.svg);
        assert!(b.cut_area_mm2>a.cut_area_mm2);
    }
    #[test]
    fn svg_provenance_not_production_claims() {
        let a=export(PantsMeasures::default(),10.0,210.0).unwrap();
        assert!(a.svg.contains("data-pants-system=\"textillab.pants.experimental.1\""));
        assert!(a.svg.contains("FDL"));
        assert!(a.svg.contains("HH"));
        assert!(a.svg.contains("VH"));
        assert_eq!(a.pieces_to_cut,4);
        assert!(a.estimated_mass_g>0.0);
    }
    #[test]
    fn reject_incompatible_levels() {
        let mut m=PantsMeasures::default();
        m.set_cm("outseam",95.0).unwrap();
        assert!(draft(m).is_err());
    }
}

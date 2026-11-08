//! Parametric two-piece hood prototype with verified *sewing-line* geometry.
//!
//! Publicly described Müller-&-Sohn concepts inform variant names and sample
//! dimensions; the geometry below is original TextilLab, not a certified
//! Müller-&-Sohn women's, men's, or children's drafting system.
//! Geometric seam agreement is not physical fit or manufacturing approval.

use crate::garments::{self, GarmentExport, Piece};
use crate::Point;
use std::fmt::Write;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Profile { Women, Men, Child }
impl Profile {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "women" => Ok(Self::Women),
            "men" => Ok(Self::Men),
            "child" => Ok(Self::Child),
            _ => Err("Unknown hood profile: women / men / child".into()),
        }
    }
    fn name(self) -> &'static str {
        match self {Self::Women=>"Damen / DOB",Self::Men=>"Herren / HAKA",Self::Child=>"Kinder"}
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style { Close, Relaxed }
impl Style {
    pub fn parse(value:&str)->Result<Self,String>{
        match value {"close"=>Ok(Self::Close),"relaxed"=>Ok(Self::Relaxed),_=>Err("Unknown hood shape".into())}
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SewingMode { Match, Ease, Stretch }
impl SewingMode {
    pub fn parse(v:&str)->Result<Self,String>{
        match v {"match"=>Ok(Self::Match),"ease"=>Ok(Self::Ease),"stretch"=>Ok(Self::Stretch),
            _=>Err("Unknown sewing-line treatment".into())}
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Measurements {
    pub head_mm:f64,
    pub back_neck_half_mm:f64,
    pub front_neck_half_mm:f64,
    pub hood_height_mm:f64,
    pub hood_depth_mm:f64,
    pub age_years:f64,
}
impl Measurements {
    pub fn for_profile(group:Profile)->Self {
        match group {
            Profile::Women=>Self{head_mm:570.0,back_neck_half_mm:100.0,front_neck_half_mm:120.0,hood_height_mm:320.0,hood_depth_mm:270.0,age_years:0.0},
            Profile::Men=>Self{head_mm:590.0,back_neck_half_mm:105.0,front_neck_half_mm:125.0,hood_height_mm:340.0,hood_depth_mm:280.0,age_years:0.0},
            Profile::Child=>Self{head_mm:520.0,back_neck_half_mm:80.0,front_neck_half_mm:95.0,hood_height_mm:280.0,hood_depth_mm:240.0,age_years:8.0},
        }
    }
    pub fn set_cm(&mut self,key:&str,value:f64)->Result<(),String>{
        if !value.is_finite(){return Err(format!("Invalid hood measure: {key}"))}
        match key {
            "head"=>self.head_mm=value*10.0,
            "backNeck"=>self.back_neck_half_mm=value*10.0,
            "frontNeck"=>self.front_neck_half_mm=value*10.0,
            "hoodHeight"=>self.hood_height_mm=value*10.0,
            "hoodDepth"=>self.hood_depth_mm=value*10.0,
            "ageYears"=>self.age_years=value,
            _=>return Err(format!("Unknown hood measure: {key}")),
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Options {
    pub profile:Profile,
    pub style:Style,
    pub mode:SewingMode,
    pub planned_ease_mm:f64,
    pub material_stretch_pct:f64,
    pub material_stretch_verified:bool,
    pub drawstrings:bool,
    pub seam_allowance_mm:f64,
    pub density_g_m2:f64
}
impl Default for Options {
    fn default()->Self {
        Self{profile:Profile::Women,style:Style::Close,mode:SewingMode::Match,
            planned_ease_mm:4.0,material_stretch_pct:0.0,
            material_stretch_verified:false,drawstrings:false,
            seam_allowance_mm:10.0,density_g_m2:210.0}
    }
}
fn p(x:f64,y:f64)->Point{Point::new(x,y)}
fn len(pts:&[Point])->f64{
    pts.windows(2).map(|w|(w[1].x-w[0].x).hypot(w[1].y-w[0].y)).sum()
}
fn join(a:&[Point],b:&[Point])->Vec<Point>{
    let mut result=a.to_vec();
    result.extend_from_slice(&b[1..]);
    result
}
fn reversed(points:&[Point])->Vec<Point>{points.iter().rev().copied().collect()}
fn cubic(a:Point,b:Point,c:Point,d:Point)->Vec<Point>{
    let mut result=Vec::with_capacity(57);
    for i in 0..=56 {
        let t=i as f64/56.0;
        let u=1.0-t;
        result.push(p(
            u*u*u*a.x+3.0*u*u*t*b.x+3.0*u*t*t*c.x+t*t*t*d.x,
            u*u*u*a.y+3.0*u*u*t*b.y+3.0*u*t*t*c.y+t*t*t*d.y
        ));
    }
    result
}
fn neck_curve(start:Point,target:f64,rise:f64,dy:f64)->Result<Vec<Point>,String>{
    let make=|span:f64|cubic(start,
        p(start.x+.28*span,start.y-rise),
        p(start.x+.73*span,start.y-rise),
        p(start.x+span,start.y+dy));
    let (mut low,mut high)=(1.0,target);
    for _ in 0..56 {
        let mid=(low+high)*.5;
        if len(&make(mid))>target {high=mid;} else {low=mid;}
    }
    let pts=make((low+high)*.5);
    if (len(&pts)-target).abs()>.05 {return Err("Hood neckline curve cannot be matched".into())}
    Ok(pts)
}
#[derive(Clone, Debug)]
pub struct SewingLine {
    pub id:&'static str,
    pub hood_mm:f64,
    pub garment_mm:f64,
    pub delta_mm:f64,
    pub expected_delta_mm:f64,
    pub matched:bool
}
fn check_pair(id:&'static str,a:f64,b:f64,mode:SewingMode,
    ease:f64,stretch_pct:f64)->SewingLine{
    let delta=a-b;
    let expected=if mode==SewingMode::Ease{ease}else{0.0};
    let matched=match mode {
        SewingMode::Match=>(delta).abs()<=.25,
        SewingMode::Ease=>a>=b && (delta-ease).abs()<=.25,
        SewingMode::Stretch=>a<b && (b/a-1.0)*100.0<=stretch_pct+1e-8,
    };
    SewingLine{id,hood_mm:a,garment_mm:b,delta_mm:delta,
        expected_delta_mm:expected,matched}
}
fn validate(m:Measurements,o:Options)->Result<(f64,f64),String>{
    let (extra_h,extra_d)=if o.style==Style::Relaxed {(30.0,45.0)}else{(0.0,0.0)};
    let h=m.hood_height_mm+extra_h;
    let depth=m.hood_depth_mm+extra_d;
    let head_range=if o.profile==Profile::Child {400.0..=620.0}else{480.0..=720.0};
    if !m.head_mm.is_finite()||!head_range.contains(&m.head_mm) {
        return Err("Head circumference outside selected body profile".into());
    }
    for x in [m.back_neck_half_mm,m.front_neck_half_mm,h,depth]{
        if !x.is_finite()||x<=0.0{return Err("Invalid hood body measurements".into());}
    }
    if !(50.0..=200.0).contains(&m.back_neck_half_mm) ||
       !(50.0..=240.0).contains(&m.front_neck_half_mm) ||
       !(240.0..=480.0).contains(&h) ||
       !(190.0..=460.0).contains(&depth) ||
       h<m.head_mm*.44 || depth<m.head_mm*.36 ||
       m.back_neck_half_mm+m.front_neck_half_mm>depth*1.22 {
        return Err("Incompatible head, height, depth and neckline measures".into());
    }
    if o.profile==Profile::Child {
        if !m.age_years.is_finite() || !(1.0..=14.0).contains(&m.age_years) {
            return Err("Child age must be between 1 and 14 years".into());
        }
        if o.drawstrings {return Err("Neck drawstrings are forbidden for child hood drafts".into());}
    }
    if !o.seam_allowance_mm.is_finite() ||
       !(0.0..=30.0).contains(&o.seam_allowance_mm) {
        return Err("Hood seam allowance must be between 0 and 30 mm".into());
    }
    if !o.density_g_m2.is_finite() ||
       !(1.0..=2000.0).contains(&o.density_g_m2) {
        return Err("Invalid fabric density".into());
    }
    match o.mode {
        SewingMode::Match=>{},
        SewingMode::Ease=>{
            if !o.planned_ease_mm.is_finite() ||
                !(0.0..=30.0).contains(&o.planned_ease_mm) ||
                o.planned_ease_mm==0.0{
                return Err("Ease mode requires a positive planned amount, <=30mm".into());
            }
        },
        SewingMode::Stretch=>{
            if !o.material_stretch_verified ||
                !o.material_stretch_pct.is_finite() ||
                !(0.0..=12.0).contains(&o.material_stretch_pct) ||
                o.material_stretch_pct==0.0 {
                return Err("Stretch requires verified material data, <=12 percent".into());
            }
        }
    }
    Ok((h,depth))
}
#[derive(Clone, Debug)]
pub struct HoodDraft {
    pub outline:Vec<Point>,
    pub back_curve:Vec<Point>,
    pub front_curve:Vec<Point>,
    pub center_curve:Vec<Point>,
    pub seams:Vec<SewingLine>,
    pub height_mm:f64,
    pub depth_mm:f64,
    pub validated_for_production:bool
}
pub fn draft(m:Measurements,o:Options)->Result<HoodDraft,String>{
    let (h,depth)=validate(m,o)?;
    let added=if o.mode==SewingMode::Ease{o.planned_ease_mm}else{0.0};
    let factor=if o.mode==SewingMode::Stretch {
        1.0/(1.0+o.material_stretch_pct/100.0)
    }else{1.0};
    let back_target=(m.back_neck_half_mm+added*.5)*factor;
    let front_target=(m.front_neck_half_mm+added*.5)*factor;
    let a=p(0.0,h);
    let back=neck_curve(a,back_target,14.0,-5.0)?;
    let shoulder=*back.last().expect("curve end");
    let front=neck_curve(shoulder,front_target,12.0,-12.0)?;
    let face_point=*front.last().expect("curve end");
    let apex=p(depth*.54,0.0);
    let back_top=join(
        &cubic(a,p(-17.0,h*.61),p(-8.0,h*.10),p(depth*.25,8.0)),
        &cubic(p(depth*.25,8.0),p(depth*.34,-5.0),p(depth*.47,-2.0),apex)
    );
    let face=join(
        &cubic(apex,p(depth*.84,-2.0),p(depth*1.03,h*.14),p(depth*.98,h*.34)),
        &cubic(p(depth*.98,h*.34),p(depth*1.05,h*.59),p(depth*.92,h*.75),face_point)
    );
    let mut outline=join(&join(&join(&back_top,&face),&reversed(&front)),&reversed(&back));
    outline.pop();
    let center=len(&back_top);
    let seams=vec![
        check_pair("center-pair",center,center,SewingMode::Match,0.0,0.0),
        check_pair("back-neck",len(&back),m.back_neck_half_mm,o.mode,added*.5,o.material_stretch_pct),
        check_pair("front-neck",len(&front),m.front_neck_half_mm,o.mode,added*.5,o.material_stretch_pct)
    ];
    if !seams.iter().all(|s|s.matched){return Err("Hood sewing-line contracts failed".into());}
    Ok(HoodDraft{outline,back_curve:back,front_curve:front,center_curve:back_top,
        seams,height_mm:h,depth_mm:depth,validated_for_production:false})
}
fn plot_poly(poly:&[Point])->String{
    let mut s=String::new();
    for (i,p) in poly.iter().enumerate() {
        let _=write!(s,"{} {:.3} {:.3} ",if i==0 {"M"}else{"L"},p.x,p.y);
    }
    s
}
pub fn export(m:Measurements,o:Options)->Result<GarmentExport,String>{
    let shape=draft(m,o)?;
    let cut=garments::offset(&shape.outline,o.seam_allowance_mm)?;
    let sewn=2.0*garments::area(&shape.outline).abs();
    let cut_area=2.0*garments::area(&cut).abs();
    let mass=cut_area*o.density_g_m2/1_000_000.0;
    if !mass.is_finite()||sewn<=0.0||cut_area<=0.0{
        return Err("Invalid hood cutting area".into());
    }
    let mut minx=f64::INFINITY;let mut maxx=f64::NEG_INFINITY;
    let mut miny=f64::INFINITY;let mut maxy=f64::NEG_INFINITY;
    for q in shape.outline.iter().chain(cut.iter()){
        minx=minx.min(q.x);maxx=maxx.max(q.x);
        miny=miny.min(q.y);maxy=maxy.max(q.y);
    }
    let piece_w=maxx-minx;
    let width=piece_w*2.0+210.0;
    let height=maxy-miny+155.0;
    let mut svg=format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {width:.3} {height:.3}\" width=\"{width:.3}mm\" height=\"{height:.3}mm\" data-hood-system=\"textillab.hood.prototype.1\" data-pieces=\"2\" data-seams-ok=\"true\" data-production-approved=\"false\">\n"
    );
    svg.push_str("<rect width=\"100%\" height=\"100%\" fill=\"#14202e\"/>\n");
    let _=writeln!(svg,"<text x=\"30\" y=\"27\" fill=\"#e5f2f8\" font-size=\"16\">TEXTILLAB · KAPUZE · {}</text>",o.profile.name());
    for n in 0..2 {
        let dx=if n==0 {75.0-minx}else{width-75.0+minx};
        let dy=83.0-miny;
        let flip=if n==0 {""}else{" scale(-1 1)"};
        let _=writeln!(svg,"<g data-piece-id=\"KH\" transform=\"translate({dx:.3} {dy:.3}){flip}\">");
        let _=writeln!(svg,
            "<path d=\"{} Z\" fill=\"#25495a\" fill-opacity=\".72\" stroke=\"#7fe0e5\" stroke-width=\"1.55\" stroke-dasharray=\"4 3\"/>",
            plot_poly(&shape.outline));
        if o.seam_allowance_mm>0.0{
            let _=writeln!(svg,
                "<polygon points=\"{}\" fill=\"none\" stroke=\"#f1a678\" stroke-width=\"1.3\"/>",
                garments::svg_points(&cut));
        }
        for (id,curve,color) in [
            ("center",&shape.center_curve[..],"#f4bd70"),
            ("back-neck",&shape.back_curve[..],"#67c8d6"),
            ("front-neck",&shape.front_curve[..],"#c3d782")
        ]{
            let _=writeln!(svg,
                "<path data-seam=\"{id}\" d=\"{}\" fill=\"none\" stroke=\"{color}\" stroke-width=\"2.2\"/>",
                plot_poly(curve));
        }
        let _=writeln!(svg,
            "<path d=\"M {:.3} {:.3} V {:.3} H {:.3}\" fill=\"none\" stroke=\"#668b9f\" stroke-dasharray=\"4 6\"/>",
            0.0,shape.height_mm,0.0,shape.depth_mm);
        svg.push_str("</g>\n");
    }
    let _=writeln!(svg,
        "<text x=\"30\" y=\"{:.3}\" fill=\"#d4edf3\" font-size=\"12\">H {:.1} cm · T {:.1} cm · Rücken {:.1} cm · Vorn {:.1} cm · Nähte: exakt gemessen</text>",
        height-34.0,shape.height_mm/10.0,shape.depth_mm/10.0,
        shape.seams[1].hood_mm/10.0,shape.seams[2].hood_mm/10.0);
    let _=writeln!(svg,
        "<text x=\"30\" y=\"{:.3}\" fill=\"#b0d4b9\" font-size=\"11\">NÄHLINIEN RECHNERISCH GEPRÜFT · NICHT FÜR FERTIGUNG VALIDIERT · KEINE KINDERKORDELN</text>",
        height-14.0);
    svg.push_str("</svg>\n");
    Ok(GarmentExport{
        svg,parts:vec![Piece{
            name:"Kapuzenseite",outline:shape.outline,holes:Vec::new(),copies:2,on_fold:false,
            note:"Two mirrored hood panels; pattern fit not production-verified"
        }],
        pieces_to_cut:2,sewn_area_mm2:sewn,cut_area_mm2:cut,estimated_mass_g:mass
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn separate_profile_examples_and_sewing_line_lengths() {
        for group in [Profile::Women,Profile::Men,Profile::Child]{
            let m=Measurements::for_profile(group);
            let o=Options{profile:group,..Options::default()};
            let result=draft(m,o).unwrap();
            assert_eq!(result.seams.len(),3);
            assert!(result.seams.iter().all(|x|x.matched));
            assert!((result.seams[1].hood_mm-m.back_neck_half_mm).abs()<.05);
            assert!((result.seams[2].hood_mm-m.front_neck_half_mm).abs()<.05);
            assert!(!result.validated_for_production);
            let svg=export(m,o).unwrap();
            assert_eq!(svg.pieces_to_cut,2);
            assert!(svg.svg.contains("data-hood-system="));
            assert!(svg.cut_area_mm2>svg.sewn_area_mm2);
        }
    }
    #[test]
    fn deliberate_ease_and_verified_stretch_are_checked() {
        let m=Measurements::for_profile(Profile::Women);
        let easing=Options{mode:SewingMode::Ease,planned_ease_mm:6.0,..Options::default()};
        let result=draft(m,easing).unwrap();
        assert!((result.seams[1].delta_mm-3.0).abs()<.05);
        let stretch=Options{mode:SewingMode::Stretch,material_stretch_pct:4.0,
            material_stretch_verified:true,..Options::default()};
        assert!(draft(m,stretch).unwrap().seams.iter().all(|s|s.matched));
        assert!(draft(m,Options{material_stretch_verified:false,..stretch}).is_err());
    }
    #[test]
    fn child_age_and_neck_cord_restrictions_fail_closed() {
        let mut m=Measurements::for_profile(Profile::Child);
        let settings=Options{profile:Profile::Child,..Options::default()};
        m.age_years=0.0;
        assert!(draft(m,settings).is_err());
        m.age_years=8.0;
        assert!(draft(m,Options{drawstrings:true,..settings}).is_err());
    }
    #[test]
    fn invalid_geometry_rejected() {
        let m=Measurements{front_neck_half_mm:380.0,..Measurements::for_profile(Profile::Women)};
        assert!(draft(m,Options::default()).is_err());
        let o=Options{seam_allowance_mm:f64::NAN,..Options::default()};
        assert!(draft(Measurements::for_profile(Profile::Women),o).is_err());
    }
}

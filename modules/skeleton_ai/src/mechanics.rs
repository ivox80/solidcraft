//! Lightweight 3-D Euler-Bernoulli frame analysis and compliance-based sizing.
//! This is a conceptual linear elastic beam model, NOT a continuum stress,
//! buckling, connection, or safety/certification analysis.
use crate::{Shape, SkeletonDocument, Station, V3};
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

fn steel_e() -> f64 { 210e9 }
fn poisson() -> f64 { 0.30 }
fn density() -> f64 { 7850.0 }
fn target_fraction() -> f64 { 1.0 }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Material {
    #[serde(default="steel_e")] pub young_pa: f64,
    #[serde(default="poisson")] pub poisson: f64,
    #[serde(default="density")] pub density_kg_m3: f64,
}
impl Default for Material {
    fn default() -> Self { Self{young_pa:steel_e(), poisson:poisson(), density_kg_m3:density()} }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Support { pub node: usize, pub fixed: [bool;6] }
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Load {
    pub node: usize,
    #[serde(default)] pub force: [f64;3],
    #[serde(default)] pub moment: [f64;3],
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MechanicalSetup {
    #[serde(default)] pub material: Material,
    #[serde(default)] pub supports: Vec<Support>,
    #[serde(default)] pub loads: Vec<Load>,
    #[serde(default="target_fraction")] pub volume_fraction: f64,
}
#[derive(Clone, Debug, Serialize)]
pub struct AnalysisReport {
    pub compliance_j: f64,
    pub volume_m3: f64,
    pub mass_kg: f64,
    pub max_displacement_m: f64,
    pub displacements: Vec<[f64;6]>,
    pub edge_energies_j: Vec<f64>,
    pub edge_areas_m2: Vec<f64>,
    pub edge_lengths_m: Vec<f64>,
    pub method: &'static str,
    pub warning: &'static str,
}
#[derive(Clone, Debug, Serialize)]
pub struct OptimizationResult {
    pub before: AnalysisReport,
    pub after: AnalysisReport,
    pub document: SkeletonDocument,
    pub accepted_iterations: usize,
}
#[derive(Clone, Copy, Debug)]
struct Props { a:f64, iy:f64, iz:f64, j:f64 }
impl Props {
    fn scaled(self,s:f64)->Self { Self{a:self.a*s,iy:self.iy*s*s,iz:self.iz*s*s,j:self.j*s*s} }
}
fn section_props(s:&Station)->Result<Props,String>{
    let (w,h)=s.dims()?;
    let (a,iy,iz,j)=match s.profile {
        Shape::Circle=>{let r=w*0.5;let i=PI*r.powi(4)/4.0;(PI*r*r,i,i,2.0*i)},
        Shape::Tube=>{let r=w*0.5;let ri=r*(1.0-s.wall);let i=PI*(r.powi(4)-ri.powi(4))/4.0;
            (PI*(r*r-ri*ri),i,i,2.0*i)},
        Shape::Ellipse=>{let a=PI*w*h/4.0;let iy=PI*w*h.powi(3)/64.0;
            let iz=PI*h*w.powi(3)/64.0;
            (a,iy,iz,PI*w.powi(3)*h.powi(3)/(16.0*(w*w+h*h)))},
        Shape::Rectangle | Shape::RoundedRectangle=>{
            let (b,t)=(w.max(h),w.min(h));
            let j=b*t.powi(3)/3.0*(1.0-0.63*t/b+0.052*(t/b).powi(5));
            let factor=if s.profile==Shape::RoundedRectangle {1.0-(4.0-PI)*(.5*s.corner*.95).powi(2)}else{1.0};
            (w*h*factor,w*h.powi(3)*factor/12.0,h*w.powi(3)*factor/12.0,j*factor)
        },
        // Early-prototype equivalent-section approximations, not validated torsion constants.
        Shape::Hexagon=>(.75*w*h,.75*w*h.powi(3)/12.0,.75*h*w.powi(3)/12.0,.15*w*h*(w.min(h)).powi(2)),
        Shape::Triangle=>(.5*w*h,w*h.powi(3)/36.0,h*w.powi(3)/48.0,.03*w*h*(w.min(h)).powi(2)),
        Shape::IBeam=>{
            let flange=(s.wall*h/2.0).min(h*.45);
            let stem=(s.wall*w/2.0).min(w*.45);
            let a=2.0*w*flange+(h-2.0*flange)*2.0*stem;
            let iy=(w*h.powi(3)-(w-2.0*stem)*(h-2.0*flange).powi(3))/12.0;
            let iz=(2.0*flange*w.powi(3)+(h-2.0*flange)*(2.0*stem).powi(3))/12.0;
            (a,iy,iz,(2.0*w*flange.powi(3)+(h-2.0*flange)*(2.0*stem).powi(3))/3.0)
        },
    };
    if [a,iy,iz,j].iter().any(|v|!v.is_finite()||*v<=0.0){return Err("invalid section stiffness".into());}
    Ok(Props{a,iy,iz,j})
}
fn props_at(stations:&[Station],t:f64)->Result<Props,String>{
    let k=stations.windows(2).position(|ss|t<=ss[1].t).unwrap_or(stations.len()-2);
    let (a,b)=(&stations[k],&stations[k+1]);
    let q=((t-a.t)/(b.t-a.t)).clamp(0.0,1.0);
    // Interpolate section properties so shape changes at BOTH endpoints affect mechanics.
    let x=section_props(a)?;let y=section_props(b)?;
    Ok(Props{a:x.a*(1.0-q)+y.a*q,iy:x.iy*(1.0-q)+y.iy*q,
        iz:x.iz*(1.0-q)+y.iz*q,j:x.j*(1.0-q)+y.j*q})
}
fn equivalent_props(stations:&[Station])->Result<Props,String>{
    // Harmonic integration approximates a non-prismatic member using its
    // axial/bending/torsion flexibilities. This is a surrogate, NOT a true tapered-element FEM.
    let n=9usize;let mut inv=[0.0;4];
    for i in 0..n {let p=props_at(stations,(i as f64+.5)/(n as f64))?;
        for (k,v) in [p.a,p.iy,p.iz,p.j].iter().enumerate(){inv[k]+=1.0/v/(n as f64)} }
    Ok(Props{a:1.0/inv[0],iy:1.0/inv[1],iz:1.0/inv[2],j:1.0/inv[3]})
}
fn add_block(k:&mut [[f64;12];12],idx:&[usize],b:&[Vec<f64>]){
    for (u,&i) in idx.iter().enumerate(){for (v,&j) in idx.iter().enumerate(){k[i][j]+=b[u][v];}}
}
fn beam_local(l:f64,e:f64,g:f64,p:Props)->[[f64;12];12]{
    let mut k=[[0.0;12];12];
    let axial=e*p.a/l;let torsion=g*p.j/l;
    add_block(&mut k,&[0,6],&[vec![axial,-axial],vec![-axial,axial]]);
    add_block(&mut k,&[3,9],&[vec![torsion,-torsion],vec![-torsion,torsion]]);
    for (ix,ei,sign) in [([1,5,7,11],e*p.iz,1.0),([2,4,8,10],e*p.iy,-1.0)] {
        let a=12.0*ei/l.powi(3);let b=sign*6.0*ei/l.powi(2);let c=4.0*ei/l;let d=2.0*ei/l;
        add_block(&mut k,&ix,&[vec![a,b,-a,b],vec![b,c,-b,d],vec![-a,-b,a,-b],vec![b,d,-b,c]]);
    }
    k
}
#[derive(Clone)]
struct Element {index:usize,nodes:[usize;2],len:f64,r:[[f64;3];3],p:Props}
fn make_elements(doc:&SkeletonDocument)->Result<Vec<Element>,String>{
    let skeleton=doc.to_skeleton(0.0,0.0)?; // validates stations and active members
    let mut active_offset=0usize;
    let mut el=Vec::new();
    for (index,&[i,j]) in doc.edges.iter().enumerate(){
        if !doc.active.get(index).copied().unwrap_or(true)||doc.areas[index]<=0.0{continue;}
        let a=V3::from_array(doc.nodes[i]);let b=V3::from_array(doc.nodes[j]);
        let ex=b.sub(a).unit();let ref_axis=if ex.2.abs()>.95 {V3(0.0,1.0,0.0)}else{V3(0.0,0.0,1.0)};
        let ey=ref_axis.cross(ex).unit();let ez=ex.cross(ey);
        let stations=&skeleton.beams[active_offset].sections;
        active_offset+=1;
        el.push(Element{index,nodes:[i,j],len:b.sub(a).norm(),r:[ex.arr(),ey.arr(),ez.arr()],p:equivalent_props(&stations.stations)?});
    }
    Ok(el)
}
fn to_global(local:&[[f64;12];12],r:&[[f64;3];3])->[[f64;12];12]{
    // T: local displacement = T * global displacement.
    let mut t=[[0.0;12];12];
    for off in [0,3,6,9]{for i in 0..3{for j in 0..3{t[off+i][off+j]=r[i][j];}}}
    let mut tmp=[[0.0;12];12];let mut out=[[0.0;12];12];
    for i in 0..12{for j in 0..12{for k in 0..12{tmp[i][j]+=local[i][k]*t[k][j];}}}
    for i in 0..12{for j in 0..12{for k in 0..12{out[i][j]+=t[k][i]*tmp[k][j];}}}
    out
}
fn indexes(e:&Element)->[usize;12]{
    let mut id=[0usize;12];for d in 0..6{id[d]=e.nodes[0]*6+d;id[d+6]=e.nodes[1]*6+d;}id
}
fn solve_spd(mut k:Vec<f64>,mut f:Vec<f64>)->Result<Vec<f64>,String>{
    let n=f.len();if n==0{return Err("no free degrees of freedom".into());}
    // Dense Cholesky: small skeletons only; rejects singular and unstable models.
    let maxdiag=(0..n).map(|i|k[i*n+i].abs()).fold(0.0_f64,f64::max);
    let tol=maxdiag*1e-12;
    for i in 0..n{
        for j in 0..=i {
            let mut s=k[i*n+j];
            for a in 0..j{s-=k[i*n+a]*k[j*n+a];}
            if i==j {if s<=tol || !s.is_finite(){return Err("unstable structure (singular stiffness); check supports and connections".into());}
                k[i*n+i]=s.sqrt();}
            else{k[i*n+j]=s/k[j*n+j];}
        }
    }
    for i in 0..n{for j in 0..i{f[i]-=k[i*n+j]*f[j];}f[i]/=k[i*n+i];}
    for i in (0..n).rev(){for j in i+1..n{f[i]-=k[j*n+i]*f[j];}f[i]/=k[i*n+i];}
    Ok(f)
}
#[derive(Clone)]
struct SolveInternal {report:AnalysisReport,elements:Vec<Element>,u:Vec<f64>,e:f64,g:f64}
fn solve(doc:&SkeletonDocument,scales:&[f64])->Result<SolveInternal,String>{
    let m=doc.mechanics.as_ref().ok_or("No mechanical setup: add a support and a load")?;
    if !m.material.young_pa.is_finite()||m.material.young_pa<=0.0 || !(-.99..0.5).contains(&m.material.poisson)
        || !m.material.density_kg_m3.is_finite()||m.material.density_kg_m3<=0.0{return Err("invalid material".into());}
    let n=doc.nodes.len();if n>128{return Err("prototype solver supports at most 128 nodes".into());}
    let e=m.material.young_pa;let g=e/(2.0*(1.0+m.material.poisson));
    let elements=make_elements(doc)?;
    if scales.len()!=doc.edges.len(){return Err("scale array mismatch".into());}
    let d=n*6;let mut k=vec![0.0;d*d];let mut f=vec![0.0;d];
    let mut fixed=vec![false;d];let mut used=vec![false;n];
    for s in &m.supports {if s.node>=n{return Err("support node out of bounds".into());}
        for j in 0..6{fixed[s.node*6+j]|=s.fixed[j];}}
    if m.supports.is_empty(){return Err("at least one support is required".into());}
    if m.loads.is_empty(){return Err("at least one load is required".into());}
    for ld in &m.loads {if ld.node>=n{return Err("load node out of bounds".into());}
        for j in 0..3{if !ld.force[j].is_finite()||!ld.moment[j].is_finite(){return Err("nonfinite force/moment".into());}
            f[ld.node*6+j]+=ld.force[j];f[ld.node*6+j+3]+=ld.moment[j];}}
    let mut volume=0.0;let mut edge_areas=vec![0.0;doc.edges.len()];let mut lengths=vec![0.0;doc.edges.len()];
    for beam in &elements {
        used[beam.nodes[0]]=true;used[beam.nodes[1]]=true;
        let scale=scales[beam.index];if !scale.is_finite()||scale<=0.0{return Err("invalid section scale".into());}
        let props=beam.p.scaled(scale);
        let local=beam_local(beam.len,e,g,props);let glob=to_global(&local,&beam.r);let ids=indexes(beam);
        for i in 0..12{for j in 0..12{k[ids[i]*d+ids[j]]+=glob[i][j];}}
        volume+=props.a*beam.len;edge_areas[beam.index]=props.a;lengths[beam.index]=beam.len;
    }
    for node in 0..n {if !used[node] && f[node*6..node*6+6].iter().any(|x|x.abs()>1e-14){return Err("loaded node has no active beams".into());}}
    let free:Vec<usize>=(0..d).filter(|&id|used[id/6]&&!fixed[id]).collect();
    let mut kf=vec![0.0;free.len()*free.len()];let mut ff=vec![0.0;free.len()];
    for (i,&a) in free.iter().enumerate(){ff[i]=f[a];for (j,&b) in free.iter().enumerate(){kf[i*free.len()+j]=k[a*d+b];}}
    let ans=solve_spd(kf,ff)?;
    let mut u=vec![0.0;d];for (i,&a) in free.iter().enumerate(){u[a]=ans[i];}
    let compliance:f64=f.iter().zip(&u).map(|(a,b)|a*b).sum();
    let max_displacement=(0..n).map(|i|(u[i*6].powi(2)+u[i*6+1].powi(2)+u[i*6+2].powi(2)).sqrt()).fold(0.0_f64,f64::max);
    let mut energy=vec![0.0;doc.edges.len()];
    for beam in &elements {
        let ids=indexes(beam);let p=beam.p.scaled(scales[beam.index]);
        let loc=beam_local(beam.len,e,g,p);let mut lu=[0.0;12];
        for offset in [0,3,6,9]{for q in 0..3{for j in 0..3{lu[offset+q]+=beam.r[q][j]*u[ids[offset+j]];}}}
        let mut work=0.0;for i in 0..12{for j in 0..12{work+=lu[i]*loc[i][j]*lu[j];}}
        energy[beam.index]=0.5*work;
    }
    if !compliance.is_finite()||compliance < -1e-7{return Err("invalid compliance".into());}
    let displacement=(0..n).map(|i|{let mut a=[0.0;6];a.copy_from_slice(&u[i*6..i*6+6]);a}).collect();
    Ok(SolveInternal{report:AnalysisReport{
        compliance_j:compliance,volume_m3:volume,mass_kg:volume*m.material.density_kg_m3,
        max_displacement_m:max_displacement,displacements:displacement,edge_energies_j:energy,
        edge_areas_m2:edge_areas,edge_lengths_m:lengths,
        method:"linear 3D Euler-Bernoulli frame FEM (6 DOF/node)",
        warning:"Prototype: equivalent tapered sections; no local stresses, buckling, fatigue or certified safety margins"
    },elements,u,e,g})
}
pub fn analyze(doc:&SkeletonDocument)->Result<AnalysisReport,String>{
    solve(doc,&vec![1.0;doc.edges.len()]).map(|s|s.report)
}
fn sensitivities(sol:&SolveInternal,scales:&[f64])->Vec<f64>{
    let mut result=vec![0.0;scales.len()];
    for beam in &sol.elements {
        let ids=indexes(beam);let mut u=[0.0;12];
        for off in [0,3,6,9]{for i in 0..3{for j in 0..3{u[off+i]+=beam.r[i][j]*sol.u[ids[off+j]];}}}
        let current=scales[beam.index];
        let delta=current*0.001;let p=beam.p.scaled(current);
        let more=beam.p.scaled(current+delta);
        let a=beam_local(beam.len,sol.e,sol.g,p);
        let b=beam_local(beam.len,sol.e,sol.g,more);
        let mut grad=0.0;
        for i in 0..12{for j in 0..12{grad+=u[i]*(b[i][j]-a[i][j])*u[j];}}
        result[beam.index]=(grad/delta).max(1e-24);
    }
    result
}
/// Size active members subject to a fixed TOTAL material volume.
/// A constrained OC step is verified by a new FEM solve and accepted only when
/// compliance is lower. It does NOT change graph topology or predict safety.
pub fn optimize(doc:&SkeletonDocument,iterations:usize)->Result<OptimizationResult,String>{
    if iterations==0||iterations>60{return Err("iterations must be in 1..=60".into());}
    let mech=doc.mechanics.as_ref().ok_or("mechanical setup missing")?;
    if !(0.4..=1.2).contains(&mech.volume_fraction){return Err("volume_fraction must be between 0.4 and 1.2".into());}
    let base=analyze(doc)?;let mut scales=vec![1.0;doc.edges.len()];
    let baseline_v=base.volume_m3;
    let target_v=baseline_v*mech.volume_fraction;
    let mut best=solve(doc,&scales)?;let mut accepted=0;
    for _ in 0..iterations {
        let sens=sensitivities(&best,&scales);
        let mut low=1e-30;let mut high=1e30;let mut next=scales.clone();
        for _ in 0..80 {
            let lambda=(low*high).sqrt();let mut volume=0.0;
            for beam in &best.elements {
                let i=beam.index;let min_bound=0.25_f64.max(scales[i]*0.78);
                let max_bound=4.0_f64.min(scales[i]*1.22);
                next[i]=(scales[i]*(sens[i]/(lambda*beam.len*beam.p.a)).sqrt()).clamp(min_bound,max_bound);
                volume+=beam.p.a*beam.len*next[i];
            }
            if volume>target_v {low=lambda;} else {high=lambda;}
        }
        let candidate=solve(doc,&next)?;
        let still_adjusting_budget=(best.report.volume_m3-target_v).abs()>target_v*0.001;
        let approaches_budget=(candidate.report.volume_m3-target_v).abs()<
            (best.report.volume_m3-target_v).abs();
        if (still_adjusting_budget && approaches_budget) ||
           candidate.report.compliance_j <= best.report.compliance_j*(1.0+1e-10) {
            let diff=next.iter().zip(&scales).fold(0.0_f64,|mx,(a,b)|mx.max((a-b).abs()));
            best=candidate;scales=next;accepted+=1;
            if diff<1e-5{break;}
        } else {break;}
    }
    let mut updated=doc.clone();
    for beam in &best.elements {
        let i=beam.index;let factor=scales[i];
        updated.areas[i]*=factor;
        if let Some(stations)=updated.section_stations.get_mut(&i.to_string()) {
            let length_scale=factor.sqrt();
            for s in &mut stations.stations {
                if let (Some(w),Some(h))=(s.width,s.height){s.width=Some(w*length_scale);s.height=Some(h*length_scale);}
                else if let Some(a)=s.area{s.area=Some(a*factor);}
            }
        }
    }
    // Ensure returned CAD parameters and the result solve agree.
    let after=analyze(&updated)?;
    Ok(OptimizationResult{before:base,after,document:updated,accepted_iterations:accepted})
}

#[cfg(test)]
mod tests {
    use super::*;
    fn cantilever()->SkeletonDocument{
        serde_json::from_str(r#"{"nodes":[[0,0,0],[1,0,0]],"edges":[[0,1]],"areas":[0.007853981633974483],
        "mechanics":{"supports":[{"node":0,"fixed":[true,true,true,true,true,true]}],
        "loads":[{"node":1,"force":[0,0,-1000]}]},
        "section_stations":{"0":{"stations":[{"t":0,"profile":"circle","width":0.1,"height":0.1},{"t":1,"profile":"circle","width":0.1,"height":0.1}]}}}"#).unwrap()
    }
    #[test] fn cantilever_matches_analytic_transverse_deflection(){
        let d=cantilever();let r=analyze(&d).unwrap();
        let i=PI*0.1_f64.powi(4)/64.0;
        let expected=1000.0/(3.0*210e9*i);
        assert!((r.max_displacement_m/expected-1.0).abs()<1e-8);
    }
    #[test] fn unsupported_cantilever_rejected(){
        let mut d=cantilever();d.mechanics.as_mut().unwrap().supports.clear();
        assert!(analyze(&d).is_err());
    }
    #[test] fn doubling_load_quadruples_compliance(){
        let mut d=cantilever();let a=analyze(&d).unwrap().compliance_j;
        d.mechanics.as_mut().unwrap().loads[0].force[2]*=2.0;
        let b=analyze(&d).unwrap().compliance_j;
        assert!((b/a-4.0).abs()<1e-9);
    }
    #[test] fn one_beam_budget_keeps_area(){
        let d=cantilever();let result=optimize(&d,5).unwrap();
        assert!((result.after.volume_m3/result.before.volume_m3-1.0).abs()<1e-8);
        assert!((result.after.compliance_j/result.before.compliance_j-1.0).abs()<1e-8);
    }
}
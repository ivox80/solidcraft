//! Convex polyhedral modeling kernel for v0.3.
//! All faces are planar and convex; face vertices are counter-clockwise as seen from outside.
//! Polygonal cylinders approximate curved surfaces; this is NOT NURBS/B-Rep CAD.
use std::f64::consts::PI;
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

type V3 = [f64; 3];
const EPS: f64 = 1.0e-7;
const MAX_PARTS: usize = 512;

fn add(a: V3, b: V3) -> V3 { [a[0]+b[0],a[1]+b[1],a[2]+b[2]] }
fn sub(a: V3, b: V3) -> V3 { [a[0]-b[0],a[1]-b[1],a[2]-b[2]] }
fn mul(a: V3, s:f64) -> V3 { [a[0]*s,a[1]*s,a[2]*s] }
fn dot(a:V3,b:V3)->f64 { a[0]*b[0]+a[1]*b[1]+a[2]*b[2] }
fn cross(a:V3,b:V3)->V3 {
    [a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]]
}
fn norm(a:V3)->f64 { dot(a,a).sqrt() }
fn normalize(a:V3)->Option<V3> {
    let n=norm(a); if n<EPS {None} else {Some(mul(a,1.0/n))}
}
fn near(a:V3,b:V3)->bool { norm(sub(a,b))<EPS*20.0 }

#[derive(Clone,Copy,Debug)]
struct Plane { n:V3, d:f64 } // outward normal, feasible region n·x <= d
impl Plane {
    fn from_face(face:&[V3])->Option<Self> {
        if face.len()<3 {return None;}
        let origin=face[0];
        for i in 1..face.len()-1 {
            if let Some(n)=normalize(cross(sub(face[i],origin),sub(face[i+1],origin))) {
                return Some(Self{n,d:dot(n,origin)});
            }
        }
        None
    }
    fn inverse(self)->Self {Self{n:mul(self.n,-1.0),d:-self.d}}
}

#[derive(Clone,Debug,PartialEq)]
#[cfg_attr(feature="wasm",derive(serde::Serialize,serde::Deserialize))]
pub struct ConvexSolid {
    pub faces:Vec<Vec<V3>>
}
impl ConvexSolid {
    pub fn cuboid(min:V3,max:V3)->Result<Self,&'static str>{
        if min.iter().chain(max.iter()).any(|x|!x.is_finite())
            || (0..3).any(|i|min[i]>=max[i]) {return Err("invalid box dimensions");}
        let [x,y,z]=min;let [a,b,c]=max;
        Ok(Self{faces:vec![
            vec![[x,y,z],[x,b,z],[a,b,z],[a,y,z]], // -Z
            vec![[x,y,c],[a,y,c],[a,b,c],[x,b,c]], // +Z
            vec![[x,y,z],[x,y,c],[x,b,c],[x,b,z]], // -X
            vec![[a,y,z],[a,b,z],[a,b,c],[a,y,c]], // +X
            vec![[x,y,z],[a,y,z],[a,y,c],[x,y,c]], // -Y
            vec![[x,b,z],[x,b,c],[a,b,c],[a,b,z]], // +Y
        ]})
    }
    pub fn cylinder(cx:f64,cy:f64,r:f64,z0:f64,z1:f64,sides:usize)->Result<Self,&'static str>{
        if ![cx,cy,r,z0,z1].iter().all(|v|v.is_finite()) ||
           r<=EPS || z1<=z0 || !(6..=48).contains(&sides) {return Err("invalid cylinder parameters");}
        let rim:Vec<[f64;2]>=(0..sides).map(|i|{
            let a=2.0*PI*(i as f64)/(sides as f64);
            [cx+r*a.cos(),cy+r*a.sin()]
        }).collect();
        let top:Vec<V3>=rim.iter().map(|p|[p[0],p[1],z1]).collect();
        let mut bottom:Vec<V3>=rim.iter().map(|p|[p[0],p[1],z0]).collect();
        let mut faces=Vec::with_capacity(sides+2);
        bottom.reverse();faces.push(bottom);
        faces.push(top);
        for i in 0..sides {
            let j=(i+1)%sides;let p=rim[i];let q=rim[j];
            faces.push(vec![[p[0],p[1],z0],[q[0],q[1],z0],[q[0],q[1],z1],[p[0],p[1],z1]]);
        }
        Ok(Self{faces})
    }
    pub fn rotated_z(&self,degrees:f64)->Result<Self,&'static str>{
        if !degrees.is_finite(){return Err("invalid angle");}
        let (sn,cs)=degrees.to_radians().sin_cos();
        let center=self.centroid();
        let mut new=self.clone();
        for f in &mut new.faces {for p in f{
            let x=p[0]-center[0];let y=p[1]-center[1];
            p[0]=center[0]+cs*x-sn*y;
            p[1]=center[1]+sn*x+cs*y;
        }}
        Ok(new)
    }
    pub fn centroid(&self)->V3 {
        let mut pts=Vec::<V3>::new();
        for f in &self.faces {for &p in f{if !pts.iter().any(|q|near(*q,p)){pts.push(p);}}}
        let n=pts.len().max(1) as f64;
        pts.into_iter().fold([0.,0.,0.],add).map(|v|v/n)
    }
    fn planes(&self)->Result<Vec<Plane>,&'static str>{
        self.faces.iter().map(|f|Plane::from_face(f).ok_or("degenerate face")).collect()
    }
    pub fn validate(&self)->Result<(),&'static str>{
        if self.faces.len()<4 || self.faces.len()>128{return Err("invalid face count");}
        let planes=self.planes()?;
        for (f,p) in self.faces.iter().zip(&planes){
            if f.len()<3 || f.len()>128 {return Err("invalid face size");}
            for &v in f {
                if v.iter().any(|x|!x.is_finite()){return Err("non-finite geometry");}
                if (dot(p.n,v)-p.d).abs()>EPS*100.0{return Err("non-planar face");}
            }
        }
        for p in &planes {for f in &self.faces {for &v in f {
            if dot(p.n,v)>p.d+EPS*100.0 {return Err("non-convex or inward-facing solid");}
        }}}
        if self.volume()<EPS{return Err("zero volume");}
        Ok(())
    }
    pub fn volume(&self)->f64 {
        let mut volume=0.0;
        for f in &self.faces {
            for i in 1..f.len().saturating_sub(1) {
                volume+=dot(f[0],cross(f[i],f[i+1]))/6.0;
            }
        }
        volume.abs()
    }
    pub fn offset_face(&self,index:usize,delta:f64)->Result<Self,&'static str>{
        if !delta.is_finite(){return Err("invalid face offset");}
        let mut planes=self.planes()?;
        let p=planes.get_mut(index).ok_or("face index out of bounds")?;
        p.d+=delta;
        let result=Self::from_planes(&planes).ok_or("face offset would collapse solid")?;
        // Reject edits where the selected face disappears entirely.
        if !result.faces.iter().any(|face|{
            let pl=Plane::from_face(face).unwrap();
            dot(pl.n,p.n)>1.0-EPS*10.0 && (pl.d-p.d).abs()<EPS*100.0
        }) {return Err("selected face disappeared");}
        result.validate()?;
        Ok(result)
    }
    pub fn intersection(&self,other:&Self)->Option<Self>{
        let mut planes=self.planes().ok()?;
        planes.extend(other.planes().ok()?);
        Self::from_planes(&planes)
    }
    pub fn subtract(&self,cutter:&Self)->Vec<Self>{
        if self.intersection(cutter).is_none(){return vec![self.clone()];}
        let mut inside=Some(self.clone());
        let mut fragments=Vec::new();
        for plane in cutter.planes().expect("validated cutter"){
            let Some(piece)=inside.take() else {break;};
            let original_planes=piece.planes().expect("validated solid");
            let mut outside_planes=original_planes.clone();
            outside_planes.push(plane.inverse());
            if let Some(outside)=Self::from_planes(&outside_planes){fragments.push(outside);}
            let mut inside_planes=original_planes;
            inside_planes.push(plane);
            inside=Self::from_planes(&inside_planes);
        }
        fragments
    }
    /// Construct bounded convex solid from outward half-spaces, enumerating triple-plane vertices.
    fn from_planes(input:&[Plane])->Option<Self>{
        if input.len()>130 {return None;}
        let mut planes:Vec<Plane>=Vec::new();
        for &p in input{
            if let Some(existing)=planes.iter_mut().find(|q|dot(q.n,p.n)>1.0-1e-10){
                if p.d<existing.d {existing.d=p.d;}
            }else{planes.push(p);}
        }
        let mut points:Vec<V3>=Vec::new();
        for i in 0..planes.len(){
            for j in i+1..planes.len(){
                for k in j+1..planes.len(){
                    let a=planes[i];let b=planes[j];let c=planes[k];
                    let bc=cross(b.n,c.n);
                    let det=dot(a.n,bc);
                    if det.abs()<1e-10{continue;}
                    let candidate=mul(add(add(mul(bc,a.d),mul(cross(c.n,a.n),b.d)),
                        mul(cross(a.n,b.n),c.d)),1.0/det);
                    if !candidate.iter().all(|x|x.is_finite()){continue;}
                    if planes.iter().all(|q|dot(q.n,candidate)<=q.d+EPS*20.0) &&
                        !points.iter().any(|p|near(*p,candidate)) {
                        points.push(candidate);
                    }
                }
            }
        }
        if points.len()<4{return None;}
        let mut faces=Vec::new();
        for p in &planes {
            let mut verts:Vec<V3>=points.iter().copied()
                .filter(|v|(dot(p.n,*v)-p.d).abs()<EPS*50.0).collect();
            if verts.len()<3{continue;}
            let origin=mul(verts.iter().copied().fold([0.,0.,0.],add),1.0/verts.len() as f64);
            let reference=if p.n[2].abs()<0.8 {[0.,0.,1.]} else {[0.,1.,0.]};
            let u=normalize(cross(reference,p.n))?;
            let v=cross(p.n,u);
            verts.sort_by(|a,b|{
                let aa=sub(*a,origin);let bb=sub(*b,origin);
                dot(aa,v).atan2(dot(aa,u)).total_cmp(&dot(bb,v).atan2(dot(bb,u)))
            });
            // Collinear and near-zero faces cannot produce a valid solid.
            let area=verts[1..].windows(2).fold(0.,|acc,w|{
                acc+dot(p.n,cross(sub(w[0],verts[0]),sub(w[1],verts[0])))*0.5
            });
            if area>EPS {faces.push(verts);}
        }
        let solid=Self{faces};
        if solid.faces.len()>=4 && solid.volume()>EPS {Some(solid)} else {None}
    }
}
/// Subtract every convex cutter from every convex target; pieces remain convex.
pub fn subtract_convex_many(targets:&[ConvexSolid],cutters:&[ConvexSolid])
    ->Result<Vec<Vec<ConvexSolid>>,&'static str>{
    for s in targets.iter().chain(cutters.iter()){s.validate()?;}
    if targets.len()>128 || cutters.len()>128{return Err("too many solids");}
    let mut results=Vec::new();
    for target in targets {
        let mut fragments=vec![target.clone()];
        for cutter in cutters {
            let mut next=Vec::new();
            for part in fragments {
                next.extend(part.subtract(cutter));
                if next.len()>MAX_PARTS{return Err("fragment limit exceeded");}
            }
            fragments=next;
        }
        results.push(fragments);
    }
    Ok(results)
}

#[cfg(feature="wasm")]
fn from_js(value:JsValue)->Result<ConvexSolid,JsValue>{
    let s:ConvexSolid=serde_wasm_bindgen::from_value(value).map_err(|e|JsValue::from_str(&e.to_string()))?;
    s.validate().map_err(JsValue::from_str)?;
    Ok(s)
}
#[cfg(feature="wasm")]
fn to_js<T:serde::Serialize>(value:&T)->Result<JsValue,JsValue>{
    serde_wasm_bindgen::to_value(value).map_err(|e|JsValue::from_str(&e.to_string()))
}
#[cfg(feature="wasm")]
#[wasm_bindgen]
pub fn poly_box_wasm(min:JsValue,max:JsValue)->Result<JsValue,JsValue>{
    let a:V3=serde_wasm_bindgen::from_value(min).map_err(|e|JsValue::from_str(&e.to_string()))?;
    let b:V3=serde_wasm_bindgen::from_value(max).map_err(|e|JsValue::from_str(&e.to_string()))?;
    to_js(&ConvexSolid::cuboid(a,b).map_err(JsValue::from_str)?)
}
#[cfg(feature="wasm")]
#[wasm_bindgen]
pub fn poly_cylinder_wasm(cx:f64,cy:f64,r:f64,z0:f64,z1:f64,sides:usize)->Result<JsValue,JsValue>{
    to_js(&ConvexSolid::cylinder(cx,cy,r,z0,z1,sides).map_err(JsValue::from_str)?)
}
#[cfg(feature="wasm")]
#[wasm_bindgen]
pub fn poly_rotate_z_wasm(solid:JsValue,degrees:f64)->Result<JsValue,JsValue>{
    to_js(&from_js(solid)?.rotated_z(degrees).map_err(JsValue::from_str)?)
}
#[cfg(feature="wasm")]
#[wasm_bindgen]
pub fn poly_offset_face_wasm(solid:JsValue,face:usize,delta:f64)->Result<JsValue,JsValue>{
    to_js(&from_js(solid)?.offset_face(face,delta).map_err(JsValue::from_str)?)
}
#[cfg(feature="wasm")]
#[wasm_bindgen]
pub fn poly_subtract_many_wasm(targets:JsValue,cutters:JsValue)->Result<JsValue,JsValue>{
    let t:Vec<ConvexSolid>=serde_wasm_bindgen::from_value(targets)
        .map_err(|e|JsValue::from_str(&e.to_string()))?;
    let c:Vec<ConvexSolid>=serde_wasm_bindgen::from_value(cutters)
        .map_err(|e|JsValue::from_str(&e.to_string()))?;
    to_js(&subtract_convex_many(&t,&c).map_err(JsValue::from_str)?)
}
#[cfg(feature="wasm")]
#[wasm_bindgen]
pub fn poly_volume_wasm(solid:JsValue)->Result<f64,JsValue>{
    Ok(from_js(solid)?.volume())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn cube()->ConvexSolid{ConvexSolid::cuboid([0.,0.,0.],[10.,10.,10.]).unwrap()}
    fn approx(a:f64,b:f64){assert!((a-b).abs()<1e-3,"{a} != {b}");}
    #[test]fn cube_faces_are_outward(){cube().validate().unwrap();approx(cube().volume(),1000.);}
    #[test]fn rotation_preserves_volume(){
        let r=cube().rotated_z(35.).unwrap();r.validate().unwrap();
        approx(r.volume(),1000.);
        assert!((r.faces[2][1][0]-r.faces[2][0][0]).abs()>1.);
    }
    #[test]fn cylinder_has_expected_polygonal_volume(){
        let s=ConvexSolid::cylinder(0.,0.,4.,0.,10.,24).unwrap();s.validate().unwrap();
        approx(s.volume(),24. * 0.5 * 16. * (2.*PI/24.).sin() * 10.);
    }
    #[test]fn face_offset_grows_adjacent_faces(){
        let c=cube();
        let big=c.offset_face(3,2.).unwrap();big.validate().unwrap();
        approx(big.volume(),1200.);
        assert_eq!(c.faces[3][0][0],10.);
        assert!(c.offset_face(3,-11.).is_err());
    }
    #[test]fn rotated_box_boolean_conserves_volume(){
        let target=cube();
        let cutter=ConvexSolid::cuboid([2.,2.,-5.],[8.,8.,15.]).unwrap()
            .rotated_z(30.).unwrap();
        let intersection=target.intersection(&cutter).unwrap().volume();
        let fragments=target.subtract(&cutter);
        assert!(!fragments.is_empty());
        approx(fragments.iter().map(ConvexSolid::volume).sum::<f64>()+intersection,1000.);
    }
    #[test]fn cylinder_cut_volume_matches_polygonal_tool(){
        let target=cube();
        let cutter=ConvexSolid::cylinder(5.,5.,2.,-1.,11.,12).unwrap();
        let removed=target.intersection(&cutter).unwrap().volume();
        let parts=target.subtract(&cutter);
        approx(removed+parts.iter().map(ConvexSolid::volume).sum::<f64>(),1000.);
    }
    #[test]fn outside_cutter_is_noop(){
        let target=cube();
        let cutter=ConvexSolid::cuboid([20.,20.,20.],[30.,30.,30.]).unwrap();
        let fragments=target.subtract(&cutter);
        assert_eq!(fragments.len(),1);
        approx(fragments[0].volume(),1000.);
    }
    #[test]fn multiple_targets_cutters(){
        let targets=[cube(),cube().rotated_z(20.).unwrap()];
        let cutters=[ConvexSolid::cylinder(5.,5.,2.,-1.,11.,8).unwrap()];
        let result=subtract_convex_many(&targets,&cutters).unwrap();
        assert_eq!(result.len(),2);
        for pieces in &result{assert!(pieces.iter().map(ConvexSolid::volume).sum::<f64>()<1000.);}
    }
}

//! SolidCraft Skeleton AI: native Rust geometry core.
//!
//! Uses the V0.4 `section_stations` document representation. `t=0` and `t=1`
//! are independent endpoints; any number of intermediate stations is allowed.
//! Generates a watertight-*candidate* triangle mesh from a smooth-union SDF.
//! Not a B-Rep solid or a FEM solver; use mechanical validation separately.
use serde::{Deserialize, Serialize};
pub mod inference;
pub mod mechanics;
use std::collections::HashMap;
use std::f64::consts::PI;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct V3(pub f64, pub f64, pub f64);
impl V3 {
    pub fn from_array(a: [f64; 3]) -> Self { Self(a[0], a[1], a[2]) }
    pub fn arr(self) -> [f64; 3] { [self.0, self.1, self.2] }
    pub fn add(self, b: Self) -> Self { Self(self.0+b.0,self.1+b.1,self.2+b.2) }
    pub fn sub(self, b: Self) -> Self { Self(self.0-b.0,self.1-b.1,self.2-b.2) }
    pub fn mul(self, s: f64) -> Self { Self(self.0*s,self.1*s,self.2*s) }
    pub fn dot(self, b: Self) -> f64 { self.0*b.0+self.1*b.1+self.2*b.2 }
    pub fn cross(self, b: Self) -> Self { Self(self.1*b.2-self.2*b.1,self.2*b.0-self.0*b.2,self.0*b.1-self.1*b.0) }
    pub fn norm(self) -> f64 { self.dot(self).sqrt() }
    pub fn unit(self) -> Self { self.mul(1.0/self.norm()) }
    pub fn is_finite(self) -> bool { self.0.is_finite()&&self.1.is_finite()&&self.2.is_finite() }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Shape {
    #[default] Circle, Ellipse, Rectangle, RoundedRectangle,
    Hexagon, Triangle, Tube, IBeam,
}
fn default_aspect() -> f64 { 1.0 }
fn default_wall() -> f64 { 0.22 }
fn default_corner() -> f64 { 0.3 }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Station {
    pub t: f64,
    #[serde(default)] pub profile: Shape,
    #[serde(default)] pub width: Option<f64>,
    #[serde(default)] pub height: Option<f64>,
    #[serde(default)] pub area: Option<f64>,
    #[serde(default = "default_aspect")] pub aspect: f64,
    #[serde(default = "default_wall")] pub wall: f64,
    #[serde(default = "default_corner")] pub corner: f64,
    #[serde(default)] pub twist_deg: f64,
}

impl Station {
    pub fn dims(&self) -> Result<(f64, f64), String> {
        if !(0.0..=1.0).contains(&self.t) || !self.t.is_finite() ||
            !self.twist_deg.is_finite() || !(0.2..=5.0).contains(&self.aspect) ||
            !(0.06..=0.48).contains(&self.wall) || !(0.0..=1.0).contains(&self.corner) {
            return Err("invalid station t/aspect/wall/corner/twist".into());
        }
        let (w,h) = match (self.width,self.height) {
            (Some(w),Some(h)) => (w,h),
            (None,None) => {
                let a=self.area.ok_or("station needs width/height or area")?;
                if !a.is_finite() || a<=0.0 { return Err("area must be positive".into()); }
                let ar=self.aspect;
                let (bw,bh,base_area)=base_shape_dimensions_area(self.profile,ar,self.wall,self.corner);
                let s=(a/base_area).sqrt();
                (bw*s,bh*s)
            }
            _ => return Err("width and height must be given together".into()),
        };
        if !w.is_finite() || !h.is_finite() || w<=0.0 || h<=0.0 {
            return Err("width and height must be positive finite values".into());
        }
        if matches!(self.profile,Shape::Circle|Shape::Tube) && (w-h).abs()>1e-6*w.max(h) {
            return Err("circle/tube require equal width and height".into());
        }
        Ok((w,h))
    }
}

fn shape_polygon(shape: Shape, half_w: f64, half_h: f64, wall: f64) -> Vec<(f64,f64)> {
    match shape {
        Shape::Rectangle|Shape::RoundedRectangle => vec![(-half_w,-half_h),(half_w,-half_h),(half_w,half_h),(-half_w,half_h)],
        Shape::Hexagon => (0..6).map(|k| {let a=k as f64*PI/3.0;(half_w*a.cos(),(2.0/3.0_f64.sqrt())*half_h*a.sin())}).collect(),
        Shape::Triangle => vec![(0.0,half_h*4.0/3.0),(-half_w,-half_h*2.0/3.0),(half_w,-half_h*2.0/3.0)],
        Shape::IBeam => {
            let t=wall;
            let thick=half_h*t; let stem=half_w*t;
            vec![(-half_w,-half_h),(half_w,-half_h),(half_w,-half_h+thick),
                (stem,-half_h+thick),(stem,half_h-thick),(half_w,half_h-thick),
                (half_w,half_h),(-half_w,half_h),(-half_w,half_h-thick),
                (-stem,half_h-thick),(-stem,-half_h+thick),(-half_w,-half_h+thick)]
        }
        _ => vec![],
    }
}

fn polygon_area(poly: &[(f64,f64)]) -> f64 {
    let mut sum=0.0;
    for i in 0..poly.len() {
        let a=poly[i]; let b=poly[(i+1)%poly.len()];
        sum+=a.0*b.1-b.0*a.1;
    }
    sum.abs()*0.5
}

fn base_shape_dimensions_area(shape:Shape,aspect:f64,wall:f64,corner:f64)->(f64,f64,f64) {
    let w=2.0*aspect; let h=2.0;
    let area=match shape {
        Shape::Circle => PI,
        Shape::Tube => PI*(1.0-(1.0-wall).powi(2)),
        Shape::Ellipse => PI*aspect,
        Shape::RoundedRectangle => {
            let r=aspect.min(1.0)*corner*0.95;
            4.0*aspect-(4.0-PI)*r*r
        }
        Shape::Hexagon => 3.0*3.0_f64.sqrt()*aspect/2.0,
        Shape::Triangle => 3.0*3.0_f64.sqrt()*aspect/4.0,
        _=>polygon_area(&shape_polygon(shape,aspect,1.0,wall)),
    };
    match shape {
        Shape::Circle|Shape::Tube => (2.0,2.0,area),
        Shape::Hexagon => (2.0*aspect,3.0_f64.sqrt(),area),
        Shape::Triangle => (3.0_f64.sqrt()*aspect,1.5,area),
        _ => (w,h,area),
    }
}

fn polygon_sd(x: f64,y:f64,p:&[(f64,f64)]) -> f64 {
    let mut closest=f64::INFINITY; let mut inside=false;
    for k in 0..p.len() {
        let a=p[k]; let b=p[(k+1)%p.len()];
        let d=(b.0-a.0,b.1-a.1);
        let ds=d.0*d.0+d.1*d.1;
        if ds<1e-25 {continue;}
        let t=(((x-a.0)*d.0+(y-a.1)*d.1)/ds).clamp(0.0,1.0);
        closest=closest.min(((x-a.0)-t*d.0).powi(2)+((y-a.1)-t*d.1).powi(2));
        if (a.1>y)!=(b.1>y) && x<(b.0-a.0)*(y-a.1)/(b.1-a.1)+a.0 {inside=!inside;}
    }
    closest.sqrt()*if inside {-1.0}else{1.0}
}

fn section_sd(st: &Station, x:f64,y:f64) -> f64 {
    let (w,h)=st.dims().expect("validated station");
    let a=w*0.5;let b=h*0.5;
    match st.profile {
        Shape::Circle => x.hypot(y)-a,
        Shape::Ellipse => ((x/a).hypot(y/b)-1.0)*a.min(b),
        Shape::Tube => (x.hypot(y)-a).max((a*(1.0-st.wall))-x.hypot(y)),
        Shape::Rectangle => rect_sd(x,y,a,b),
        Shape::RoundedRectangle => {
            let r=a.min(b)*st.corner*0.95;
            rect_sd(x,y,a-r,b-r)-r
        }
        _ => polygon_sd(x,y,&shape_polygon(st.profile,a,b,st.wall)),
    }
}
fn rect_sd(x:f64,y:f64,a:f64,b:f64)->f64 {
    let dx=x.abs()-a;let dy=y.abs()-b;
    dx.max(0.0).hypot(dy.max(0.0))+dx.max(dy).min(0.0)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EdgeStations { pub stations: Vec<Station> }
impl EdgeStations {
    pub fn validate(&self)->Result<(),String> {
        if self.stations.len()<2 {return Err("minimum two stations required".into());}
        if (self.stations[0].t).abs()>1e-9 || (self.stations.last().unwrap().t-1.0).abs()>1e-9 {
            return Err("stations must contain endpoints t=0 and t=1".into());
        }
        for s in &self.stations {s.dims()?;}
        if self.stations.windows(2).any(|w|w[1].t-w[0].t<1e-5) {
            return Err("station positions must increase".into());
        }
        Ok(())
    }
    /// Section SDF with both endpoints independently configurable.
    pub fn sdf(&self,y:f64,z:f64,t:f64)->f64 {
        let t=t.clamp(0.0,1.0);
        let idx=self.stations.windows(2).position(|w|t<=w[1].t).unwrap_or(self.stations.len()-2);
        let a=&self.stations[idx];let b=&self.stations[idx+1];
        let f=((t-a.t)/(b.t-a.t)).clamp(0.0,1.0);
        let twist=((1.0-f)*a.twist_deg+f*b.twist_deg).to_radians();
        let (si,co)=twist.sin_cos();
        let x=y*co+z*si;let zz=-y*si+z*co;
        let da=section_sd(a,x,zz);
        if a.profile==b.profile && (a.width==b.width) && (a.height==b.height) && a.area==b.area && a.aspect==b.aspect && a.wall==b.wall && a.corner==b.corner {return da;}
        (1.0-f)*da+f*section_sd(b,x,zz)
    }
    pub fn max_radius(&self)->f64 {
        self.stations.iter().map(|s|{let(w,h)=s.dims().expect("validated");(w*w+h*h).sqrt()*0.5}).fold(0.0,f64::max)
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum ActiveValue { Bool(bool), Int(i64) }
fn deserialize_active<'de, D>(deserializer: D) -> Result<Vec<bool>, D::Error>
where D: serde::Deserializer<'de> {
    let values = Vec::<ActiveValue>::deserialize(deserializer)?;
    values.into_iter().map(|v| match v {
        ActiveValue::Bool(x) => Ok(x),
        ActiveValue::Int(0) => Ok(false),
        ActiveValue::Int(1) => Ok(true),
        _ => Err(serde::de::Error::custom("active entries must be booleans or 0/1")),
    }).collect()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkeletonDocument {
    pub nodes: Vec<[f64;3]>,
    pub edges: Vec<[usize;2]>,
    pub areas: Vec<f64>,
    #[serde(default, deserialize_with = "deserialize_active")] pub active: Vec<bool>,
    #[serde(default)] pub section_stations: HashMap<String,EdgeStations>,
    #[serde(default)] pub cross_section: Option<DefaultSection>,
    #[serde(default)] pub mechanics: Option<mechanics::MechanicalSetup>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DefaultSection {
    #[serde(default)] pub profile: Shape,
    #[serde(default = "default_aspect")] pub aspect:f64,
    #[serde(default = "default_wall")] pub wall:f64,
    #[serde(default = "default_corner")] pub corner:f64,
}
impl Default for DefaultSection {
    fn default()->Self {Self{profile:Shape::Circle,aspect:1.0,wall:0.22,corner:0.3}}
}
#[derive(Clone, Debug)]
pub struct Beam {
    pub start:V3,pub end:V3,pub e0:V3,pub e1:V3,pub e2:V3,
    pub length:f64,pub sections:EdgeStations,
}
#[derive(Clone, Debug)]
pub struct Skeleton {
    pub beams:Vec<Beam>,pub joints:Vec<(V3,f64)>,pub smoothing:f64,
    pub min:V3,pub max:V3,
}
impl SkeletonDocument {
    pub fn to_skeleton(&self,blend:f64,junction:f64)->Result<Skeleton,String>{
        if !(0.0..=0.7).contains(&blend)||!(0.0..=1.25).contains(&junction){return Err("invalid blend/junction".into());}
        if self.edges.len()!=self.areas.len(){return Err("areas/edges mismatch".into());}
        if !self.active.is_empty()&&self.active.len()!=self.edges.len(){return Err("active/edges mismatch".into());}
        if self.nodes.iter().any(|p|!V3::from_array(*p).is_finite()){return Err("nonfinite node".into());}
        let mut beams=Vec::new();
        let mut degree=vec![0_usize;self.nodes.len()];
        let mut radii:Vec<Vec<f64>>=vec![vec![];self.nodes.len()];
        let spec=self.cross_section.clone().unwrap_or_default();
        let mut bbmin=V3(f64::INFINITY,f64::INFINITY,f64::INFINITY);
        let mut bbmax=V3(f64::NEG_INFINITY,f64::NEG_INFINITY,f64::NEG_INFINITY);
        for (k,edge) in self.edges.iter().enumerate() {
            let enabled=self.active.get(k).copied().unwrap_or(true);
            if !enabled||self.areas[k]<=0.0{continue;}
            if edge[0]>=self.nodes.len()||edge[1]>=self.nodes.len()||edge[0]==edge[1]{return Err(format!("invalid edge {k}"));}
            let start=V3::from_array(self.nodes[edge[0]]);let end=V3::from_array(self.nodes[edge[1]]);
            let diff=end.sub(start);let length=diff.norm();
            if length<1e-10{return Err(format!("edge {k} zero length"));}
            let e0=diff.unit();
            let reference=if e0.dot(V3(0.0,0.0,1.0)).abs()>0.95 {V3(0.0,1.0,0.0)} else {V3(0.0,0.0,1.0)};
            let e1=reference.cross(e0).unit();let e2=e0.cross(e1);
            let sections=if let Some(s)=self.section_stations.get(&k.to_string()){s.clone()}else{
                let st=Station{t:0.0,profile:spec.profile,width:None,height:None,
                    area:Some(self.areas[k]),aspect:spec.aspect,wall:spec.wall,corner:spec.corner,twist_deg:0.0};
                let mut endst=st.clone();endst.t=1.0;
                EdgeStations{stations:vec![st,endst]}
            };
            sections.validate()?;
            let r=sections.max_radius();
            degree[edge[0]]+=1;degree[edge[1]]+=1;
            radii[edge[0]].push((sections.stations[0].dims()?.0.powi(2)+sections.stations[0].dims()?.1.powi(2)).sqrt()*0.5);
            radii[edge[1]].push((sections.stations.last().unwrap().dims()?.0.powi(2)+sections.stations.last().unwrap().dims()?.1.powi(2)).sqrt()*0.5);
            for p in [start,end]{
                bbmin=V3(bbmin.0.min(p.0),bbmin.1.min(p.1),bbmin.2.min(p.2));
                bbmax=V3(bbmax.0.max(p.0),bbmax.1.max(p.1),bbmax.2.max(p.2));
            }
            beams.push(Beam{start,end,e0,e1,e2,length,sections});
            if !r.is_finite(){return Err("nonfinite radius".into());}
        }
        if beams.is_empty(){return Err("no active beams".into());}
        let mut radii_max=0.0_f64;let mut rr=Vec::new();
        for beam in &beams{let r=beam.sections.max_radius();radii_max=radii_max.max(r);rr.push(r);}
        rr.sort_by(|a,b|a.total_cmp(b));
        let smoothing=blend*rr[rr.len()/2];
        let joints=(0..self.nodes.len()).filter(|&i|degree[i]>=2 && !radii[i].is_empty() && junction>0.0)
            .map(|i|(V3::from_array(self.nodes[i]),radii[i].iter().copied().fold(f64::INFINITY,f64::min)*junction)).collect();
        let pad=2.4*radii_max+0.01;
        Ok(Skeleton{beams,joints,smoothing,min:bbmin.sub(V3(pad,pad,pad)),max:bbmax.add(V3(pad,pad,pad))})
    }
}
fn smooth_min(a:f64,b:f64,k:f64)->f64 {
    if k<=0.0{return a.min(b);}
    let h=(0.5+0.5*(b-a)/k).clamp(0.0,1.0);
    b*(1.0-h)+a*h-k*h*(1.0-h)
}
impl Skeleton {
    pub fn sdf(&self,p:V3)->f64 {
        let mut total=f64::INFINITY;
        for beam in &self.beams {
            let v=p.sub(beam.start);let axial=v.dot(beam.e0);
            let y=v.dot(beam.e1);let z=v.dot(beam.e2);
            let section=beam.sections.sdf(y,z,axial/beam.length);
            let cap=(-axial).max(axial-beam.length);
            let d=section.max(0.0).hypot(cap.max(0.0))+section.max(cap).min(0.0);
            total=if total.is_infinite(){d}else{smooth_min(total,d,self.smoothing)};
        }
        for &(q,r) in &self.joints {total=smooth_min(total,p.sub(q).norm()-r,self.smoothing);}
        total
    }
}

#[derive(Clone, Debug, Default)]
pub struct Mesh {pub vertices:Vec<[f64;3]>,pub faces:Vec<[u32;3]>}
impl Mesh {
    pub fn edge_incidence_watertight(&self)->bool {
        if self.faces.is_empty(){return false;}
        let mut counts:HashMap<(u32,u32),usize>=HashMap::new();
        for face in &self.faces {
            for (a,b) in [(face[0],face[1]),(face[1],face[2]),(face[2],face[0])] {
                let k=(a.min(b),a.max(b));*counts.entry(k).or_default()+=1;
            }
        }
        counts.values().all(|&n|n==2)
    }
    pub fn to_obj(&self)->String {
        let mut txt=String::from("# SolidCraft Skeleton AI - Rust SDF mesh (not B-Rep)\n");
        for p in &self.vertices {txt+=&format!("v {:.9} {:.9} {:.9}\n",p[0],p[1],p[2]);}
        for f in &self.faces {txt+=&format!("f {} {} {}\n",f[0]+1,f[1]+1,f[2]+1);}
        txt
    }
    pub fn to_binary_stl(&self)->Vec<u8> {
        let mut buf=vec![0_u8;80];
        let header=b"SolidCraft Skeleton AI v0.5 Rust (triangle mesh)";
        buf[..header.len()].copy_from_slice(header);
        buf.extend_from_slice(&(self.faces.len() as u32).to_le_bytes());
        for f in &self.faces {
            let p=V3::from_array(self.vertices[f[0] as usize]);
            let q=V3::from_array(self.vertices[f[1] as usize]);
            let r=V3::from_array(self.vertices[f[2] as usize]);
            let n=q.sub(p).cross(r.sub(p));
            let n=if n.norm()>1e-15 {n.unit()}else{V3::default()};
            for v in [n,p,q,r]{for x in v.arr(){buf.extend_from_slice(&(x as f32).to_le_bytes());}}
            buf.extend_from_slice(&0_u16.to_le_bytes());
        }
        buf
    }
}

/// Extract a welded manifold *candidate* from the SDF via six tetrahedra per voxel.
/// Dense grid bounded by max_cells; caller must check the returned surface quality.
pub fn generate_mesh(s:&Skeleton,resolution:usize,max_cells:usize)->Result<Mesh,String> {
    if !(16..=300).contains(&resolution){return Err("resolution must be 16..300".into());}
    let size=s.max.sub(s.min);let voxel=size.0.max(size.1).max(size.2)/resolution as f64;
    let dims=[(size.0/voxel).ceil() as usize+1,(size.1/voxel).ceil() as usize+1,(size.2/voxel).ceil() as usize+1];
    let cells=dims[0].checked_mul(dims[1]).and_then(|x|x.checked_mul(dims[2])).ok_or("grid overflow")?;
    if cells>max_cells{return Err(format!("grid too large: {cells} nodes"));}
    let idx=|x:usize,y:usize,z:usize| -> usize{(x*dims[1]+y)*dims[2]+z};
    let point=|x:usize,y:usize,z:usize|->V3{s.min.add(V3(x as f64*voxel,y as f64*voxel,z as f64*voxel))};
    let mut field=vec![0.0_f32;cells];
    for x in 0..dims[0]{for y in 0..dims[1]{for z in 0..dims[2]{field[idx(x,y,z)]=s.sdf(point(x,y,z)) as f32;}}}
    if !field.iter().any(|v|*v<0.0)||!field.iter().any(|v|*v>0.0){return Err("field has no zero isosurface".into());}
    let mut mesh=Mesh::default();
    let mut vertices:HashMap<(usize,usize),u32>=HashMap::new();
    let local=[[0,0,0],[1,0,0],[1,1,0],[0,1,0],[0,0,1],[1,0,1],[1,1,1],[0,1,1]];
    let tets:[[usize;4];6]=[[0,5,1,6],[0,1,2,6],[0,2,3,6],[0,3,7,6],[0,7,4,6],[0,4,5,6]];
    for x in 0..dims[0]-1 {for y in 0..dims[1]-1 {for z in 0..dims[2]-1 {
        let mut gid=[0_usize;8];let mut xyz=[V3::default();8];let mut val=[0.0_f64;8];
        for i in 0..8 {
            let xx=x+local[i][0];let yy=y+local[i][1];let zz=z+local[i][2];
            gid[i]=idx(xx,yy,zz);xyz[i]=point(xx,yy,zz);val[i]=field[gid[i]] as f64;
        }
        for tet in &tets {
            let inside:Vec<usize>=tet.iter().copied().filter(|&i|val[i]<0.0).collect();
            let outside:Vec<usize>=tet.iter().copied().filter(|&i|val[i]>=0.0).collect();
            if inside.is_empty()||outside.is_empty(){continue;}
            let ic=inside.iter().fold(V3::default(),|a,&i|a.add(xyz[i])).mul(1.0/inside.len() as f64);
            let oc=outside.iter().fold(V3::default(),|a,&i|a.add(xyz[i])).mul(1.0/outside.len() as f64);
            let normal_direction=oc.sub(ic);
            let mut interpolate=|a:usize,b:usize,mesh:&mut Mesh|->u32 {
                let key=(gid[a].min(gid[b]),gid[a].max(gid[b]));
                if let Some(&v)=vertices.get(&key){return v;}
                let w=(val[a]/(val[a]-val[b])).clamp(0.0,1.0);
                let v=xyz[a].add(xyz[b].sub(xyz[a]).mul(w));
                let id=mesh.vertices.len() as u32;
                mesh.vertices.push(v.arr());vertices.insert(key,id);id
            };
            let triangle=|a:u32,b:u32,c:u32,mesh:&mut Mesh| {
                let pa=V3::from_array(mesh.vertices[a as usize]);
                let pb=V3::from_array(mesh.vertices[b as usize]);
                let pc=V3::from_array(mesh.vertices[c as usize]);
                if pb.sub(pa).cross(pc.sub(pa)).dot(normal_direction)>0.0{mesh.faces.push([a,b,c]);}
                else{mesh.faces.push([a,c,b]);}
            };
            if inside.len()==1 || outside.len()==1 {
                let (one,three)=if inside.len()==1 {(inside[0],&outside)}else{(outside[0],&inside)};
                let a=interpolate(one,three[0],&mut mesh);
                let b=interpolate(one,three[1],&mut mesh);
                let c=interpolate(one,three[2],&mut mesh);
                triangle(a,b,c,&mut mesh);
            } else {
                let a=interpolate(inside[0],outside[0],&mut mesh);
                let b=interpolate(inside[0],outside[1],&mut mesh);
                let c=interpolate(inside[1],outside[0],&mut mesh);
                let d=interpolate(inside[1],outside[1],&mut mesh);
                triangle(a,b,c,&mut mesh);
                triangle(b,d,c,&mut mesh);
            }
        }
    }}}
    if mesh.faces.is_empty(){return Err("mesh contains no triangles".into());}
    Ok(mesh)
}

/// Helper for UI: change only a specific station; the other endpoint is untouched.
pub fn set_station(doc:&mut SkeletonDocument,edge_index:usize,station_index:usize,station:Station)->Result<(),String> {
    if edge_index>=doc.edges.len(){return Err("edge index out of bounds".into());}
    let key=edge_index.to_string();
    if !doc.section_stations.contains_key(&key) {
        // Materialize both endpoints from the effective default section.
        if !doc.active.get(edge_index).copied().unwrap_or(true) || doc.areas[edge_index]<=0.0 {
            return Err("inactive edge".into());
        }
        let spec=doc.cross_section.clone().unwrap_or_default();
        let st=Station {t:0.0,profile:spec.profile,width:None,height:None,
            area:Some(doc.areas[edge_index]),aspect:spec.aspect,wall:spec.wall,
            corner:spec.corner,twist_deg:0.0};
        let mut end=st.clone();end.t=1.0;
        doc.section_stations.insert(key.clone(),EdgeStations {stations:vec![st,end]});
    }
    let prof=doc.section_stations.get_mut(&key).unwrap();
    if station_index>=prof.stations.len(){return Err("station index out of bounds".into());}
    let old=prof.stations[station_index].clone();
    prof.stations[station_index]=station;
    if let Err(e)=prof.validate(){prof.stations[station_index]=old;return Err(e);}
    Ok(())
}

/// Add an interior station to an explicitly indexed connection, preserving both endpoints.
pub fn insert_station(doc:&mut SkeletonDocument,edge_index:usize,station:Station)->Result<(),String> {
    if edge_index>=doc.edges.len(){return Err("edge index out of bounds".into());}
    let key=edge_index.to_string();
    if !doc.section_stations.contains_key(&key){
        if !doc.active.get(edge_index).copied().unwrap_or(true)||doc.areas[edge_index]<=0.0{
            return Err("inactive edge".into());
        }
        let spec=doc.cross_section.clone().unwrap_or_default();
        let first=Station{t:0.0,profile:spec.profile,width:None,height:None,
            area:Some(doc.areas[edge_index]),aspect:spec.aspect,wall:spec.wall,corner:spec.corner,twist_deg:0.0};
        let mut last=first.clone();last.t=1.0;
        doc.section_stations.insert(key.clone(),EdgeStations{stations:vec![first,last]});
    }
    let p=doc.section_stations.get_mut(&key).unwrap();
    if station.t<=0.0||station.t>=1.0 {return Err("can only insert interior stations".into());}
    station.dims()?;
    let pos=p.stations.partition_point(|s|s.t<station.t);
    p.stations.insert(pos,station);
    if let Err(e)=p.validate(){p.stations.remove(pos);return Err(e);}
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn station(t:f64,p:Shape,w:f64,h:f64)->Station{
        Station{t,profile:p,width:Some(w),height:Some(h),area:None,aspect:1.0,wall:0.22,corner:0.3,twist_deg:0.0}
    }
    #[test] fn both_ends_are_independent(){
        let p=EdgeStations{stations:vec![station(0.0,Shape::Rectangle,0.2,0.1),station(0.5,Shape::Ellipse,0.1,0.08),station(1.0,Shape::Circle,0.06,0.06)]};
        p.validate().unwrap();
        assert!(p.sdf(0.07,0.0,0.0)<0.0); // inside wide start
        assert!(p.sdf(0.07,0.0,1.0)>0.0); // outside narrow end
    }
    #[test] fn setting_end_leaves_start_untouched(){
        let mut doc=SkeletonDocument{nodes:vec![[0.0,0.0,0.0],[1.0,0.0,0.0]],edges:vec![[0,1]],areas:vec![0.01],active:vec![],section_stations:HashMap::new(),cross_section:None,mechanics:None};
        let start=doc.to_skeleton(0.0,0.0).unwrap().beams[0].sections.stations[0].dims().unwrap();
        set_station(&mut doc,0,1,station(1.0,Shape::Ellipse,0.2,0.1)).unwrap();
        assert_eq!(start,doc.section_stations["0"].stations[0].dims().unwrap());
    }
    #[test] fn all_eight_profiles_have_solid_or_hollow_sdf(){
        for p in [Shape::Circle,Shape::Ellipse,Shape::Rectangle,Shape::RoundedRectangle,Shape::Hexagon,Shape::Triangle,Shape::Tube,Shape::IBeam] {
            let st=station(0.0,p,0.1,0.1);
            st.dims().unwrap();
            assert!(section_sd(&st,1.0,1.0)>0.0);
            if p==Shape::Tube{assert!(section_sd(&st,0.0,0.0)>0.0);}else{assert!(section_sd(&st,0.0,0.0)<0.0);}
        }
    }
    #[test] fn setting_start_leaves_end_untouched(){
        let mut doc=SkeletonDocument{nodes:vec![[0.0,0.0,0.0],[1.0,0.0,0.0]],edges:vec![[0,1]],areas:vec![0.01],active:vec![],section_stations:HashMap::new(),cross_section:None,mechanics:None};
        let end=doc.to_skeleton(0.0,0.0).unwrap().beams[0].sections.stations[1].dims().unwrap();
        set_station(&mut doc,0,0,station(0.0,Shape::RoundedRectangle,0.18,0.08)).unwrap();
        assert_eq!(end,doc.section_stations["0"].stations[1].dims().unwrap());
    }
    #[test] fn v04_example_json_deserializes(){
        let doc:SkeletonDocument=serde_json::from_str(include_str!("../examples/v04_custom_stations.json")).unwrap();
        let s=doc.to_skeleton(0.12,0.35).unwrap();
        assert!(!s.beams.is_empty());
        let doc2:SkeletonDocument=serde_json::from_str(include_str!("../examples/two_sided_profiles.json")).unwrap();
        assert_eq!(doc2.to_skeleton(0.0,0.0).unwrap().beams[0].sections.stations.len(),3);
    }
    #[test] fn rejects_missing_endpoint(){
        let p=EdgeStations{stations:vec![station(0.0,Shape::Circle,0.1,0.1),station(0.9,Shape::Circle,0.1,0.1)]};
        assert!(p.validate().is_err());
    }
    #[test] fn simple_mesh_generation(){
        let doc=SkeletonDocument{nodes:vec![[0.0,0.0,0.0],[1.0,0.0,0.0]],edges:vec![[0,1]],areas:vec![0.012],active:vec![],section_stations:HashMap::new(),cross_section:None,mechanics:None};
        let s=doc.to_skeleton(0.1,0.0).unwrap();
        let m=generate_mesh(&s,36,300_000).unwrap();
        assert!(!m.faces.is_empty());
        assert!(m.edge_incidence_watertight());
    }
}
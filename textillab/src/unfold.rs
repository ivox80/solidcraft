//! Exact edge-length-preserving unfold for developable triangular mesh patches.
//!
//! Each connected patch is laid out using shared triangle edges. Marking an edge
//! as a seam splits the traversal; vertices on opposite sides can acquire
//! different 2D positions. This is not an elastic flattening optimiser.

use crate::Point;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt::Write;

#[derive(Clone, Copy, Debug)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}
impl Vec3 {
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
    fn distance(self, b: Self) -> f64 {
        ((self.x - b.x).powi(2) + (self.y - b.y).powi(2) + (self.z - b.z).powi(2)).sqrt()
    }
}

#[derive(Clone, Debug)]
pub struct TriangleMesh {
    pub vertices: Vec<Vec3>,
    pub faces: Vec<[usize; 3]>,
    /// Undirected edges by vertex index. These edges are not traversed.
    pub seams: BTreeSet<(usize, usize)>,
}

fn edge(a: usize, b: usize) -> (usize, usize) {
    if a < b { (a, b) } else { (b, a) }
}
fn norm(v: Point) -> f64 { (v.x * v.x + v.y * v.y).sqrt() }
fn delta(a: Point, b: Point) -> Point { Point::new(b.x - a.x, b.y - a.y) }
fn cross(a: Point, b: Point) -> f64 { a.x * b.y - a.y * b.x }
fn same(a: Point, b: Point, tol: f64) -> bool { norm(delta(a, b)) <= tol }

#[derive(Debug, Clone)]
pub struct FlatPatch {
    pub face_ids: Vec<usize>,
    /// Each triangle keeps its own 2D coordinates to preserve seam duplicates.
    pub triangles: Vec<[Point; 3]>,
}
#[derive(Debug, Clone)]
pub struct UnfoldResult { pub patches: Vec<FlatPatch> }

impl TriangleMesh {
    pub fn validate(&self) -> Result<(), String> {
        if self.faces.is_empty() { return Err("Mesh has no faces".into()); }
        if !self.vertices.iter().all(|v| v.x.is_finite() && v.y.is_finite() && v.z.is_finite()) {
            return Err("Non-finite mesh coordinate".into());
        }
        let mut incidence: BTreeMap<(usize, usize), Vec<usize>> = BTreeMap::new();
        for (i, f) in self.faces.iter().enumerate() {
            if f.iter().any(|&v| v >= self.vertices.len()) ||
               f[0] == f[1] || f[1] == f[2] || f[0] == f[2] {
                return Err(format!("Invalid triangle {i}"));
            }
            let a = self.vertices[f[0]];
            let b = self.vertices[f[1]];
            let c = self.vertices[f[2]];
            let ab = (b.x-a.x, b.y-a.y, b.z-a.z);
            let ac = (c.x-a.x, c.y-a.y, c.z-a.z);
            let cr = (ab.1*ac.2-ab.2*ac.1, ab.2*ac.0-ab.0*ac.2, ab.0*ac.1-ab.1*ac.0);
            if (cr.0*cr.0+cr.1*cr.1+cr.2*cr.2).sqrt() < 1e-10 {
                return Err(format!("Degenerate triangle {i}"));
            }
            for j in 0..3 { incidence.entry(edge(f[j], f[(j+1)%3])).or_default().push(i); }
        }
        if incidence.values().any(|i| i.len() > 2) { return Err("Non-manifold edge".into()); }
        for &(a,b) in &self.seams {
            if a >= self.vertices.len() || b >= self.vertices.len() || !incidence.contains_key(&edge(a,b)) {
                return Err("Seam edge does not exist".into());
            }
        }
        Ok(())
    }

    /// Unfolds patches if shared vertex coordinates can be preserved.
    /// Non-developable cycles may fail instead of silently distorting material.
    pub fn unfold(&self) -> Result<UnfoldResult, String> {
        self.validate()?;
        let mut incidence: BTreeMap<(usize,usize), Vec<usize>> = BTreeMap::new();
        for (fi,f) in self.faces.iter().enumerate() {
            for j in 0..3 { incidence.entry(edge(f[j],f[(j+1)%3])).or_default().push(fi); }
        }
        let mut global_visited = vec![false; self.faces.len()];
        let mut result = Vec::new();
        for root in 0..self.faces.len() {
            if global_visited[root] { continue; }
            let mut coords = BTreeMap::<usize, Point>::new();
            let mut ids = Vec::new();
            let mut queue = VecDeque::from([root]);
            let initial = self.faces[root];
            let pa = self.vertices[initial[0]];
            let pb = self.vertices[initial[1]];
            let pc = self.vertices[initial[2]];
            let ab = pa.distance(pb);
            let ac = pa.distance(pc);
            let bc = pb.distance(pc);
            let x = (ac*ac + ab*ab - bc*bc)/(2.0*ab);
            let y = (ac*ac - x*x).max(0.0).sqrt();
            coords.insert(initial[0], Point::new(0.0,0.0));
            coords.insert(initial[1], Point::new(ab,0.0));
            coords.insert(initial[2], Point::new(x,y));
            global_visited[root] = true;
            while let Some(fi) = queue.pop_front() {
                ids.push(fi);
                let f = self.faces[fi];
                for j in 0..3 {
                    let a = f[j]; let b = f[(j+1)%3];
                    let shared = edge(a,b);
                    if self.seams.contains(&shared) { continue; }
                    let neighbours = &incidence[&shared];
                    if neighbours.len() != 2 { continue; }
                    let next = if neighbours[0] == fi { neighbours[1] } else { neighbours[0] };
                    let other_face = self.faces[next];
                    let c = *other_face.iter().find(|&&v| v != a && v != b)
                        .ok_or("Bad adjacent face")?;
                    let qa = coords[&a]; let qb = coords[&b];
                    let d = delta(qa,qb); let len = norm(d);
                    let da = self.vertices[a].distance(self.vertices[c]);
                    let db = self.vertices[b].distance(self.vertices[c]);
                    let projection = (da*da + len*len - db*db)/(2.0*len);
                    let height_sq = da*da - projection*projection;
                    if height_sq < -1e-6 { return Err("Triangle lengths inconsistent".into()); }
                    let h = height_sq.max(0.0).sqrt();
                    let perp = Point::new(-d.y/len, d.x/len);
                    let mid = Point::new(qa.x + d.x*projection/len, qa.y + d.y*projection/len);
                    // Neighbour must lie on the other side of the shared edge.
                    let opposite = f[(j+2)%3];
                    let sign = if cross(d, delta(qa, coords[&opposite])) > 0.0 { -1.0 } else { 1.0 };
                    let candidate = Point::new(mid.x+sign*h*perp.x,mid.y+sign*h*perp.y);
                    let tolerance = 1e-5_f64.max(da*1e-7);
                    if let Some(prev) = coords.get(&c) {
                        if !same(*prev,candidate,tolerance) {
                            return Err(format!("Patch cannot unfold isometrically around vertex {c}; add cut seams or use strain optimisation"));
                        }
                    } else {
                        coords.insert(c,candidate);
                    }
                    if !global_visited[next] {
                        global_visited[next] = true;
                        queue.push_back(next);
                    }
                }
            }
            let triangles = ids.iter().map(|&fi| {
                let f = self.faces[fi];
                [coords[&f[0]],coords[&f[1]],coords[&f[2]]]
            }).collect();
            result.push(FlatPatch {face_ids: ids, triangles});
        }
        Ok(UnfoldResult { patches: result })
    }
}

impl UnfoldResult {
    /// SVG pages arranged horizontally; each triangle is outlined for inspection.
    pub fn svg(&self) -> String {
        let mut out = String::from("<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 2000 900\">\n");
        let mut offset = 10.0;
        for (i, patch) in self.patches.iter().enumerate() {
            let minx = patch.triangles.iter().flat_map(|t| t.iter().map(|p| p.x)).fold(f64::INFINITY,f64::min);
            let miny = patch.triangles.iter().flat_map(|t| t.iter().map(|p| p.y)).fold(f64::INFINITY,f64::min);
            let maxx = patch.triangles.iter().flat_map(|t| t.iter().map(|p| p.x)).fold(f64::NEG_INFINITY,f64::max);
            let width = maxx-minx;
            let _ = writeln!(out,"<g transform=\"translate({:.4},{:.4})\">",offset-minx,30.0-miny);
            for tri in &patch.triangles {
                let _ = writeln!(out,"<polygon fill=\"#80b9df\" fill-opacity=\"0.25\" stroke=\"#16537e\" stroke-width=\"0.4\" points=\"{:.5},{:.5} {:.5},{:.5} {:.5},{:.5}\"/>",
                    tri[0].x,tri[0].y,tri[1].x,tri[1].y,tri[2].x,tri[2].y);
            }
            let _ = writeln!(out,"<text x=\"{:.3}\" y=\"{:.3}\" font-size=\"5\">Patch {}</text>",minx,miny-5.0,i+1);
            out.push_str("</g>\n");
            offset += width+25.0;
        }
        out.push_str("</svg>\n");
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn quad() -> TriangleMesh {
        TriangleMesh {
            vertices: vec![Vec3::new(0.0,0.0,0.0),Vec3::new(100.0,0.0,0.0),
                Vec3::new(100.0,60.0,0.0),Vec3::new(0.0,60.0,0.0)],
            faces: vec![[0,1,2],[0,2,3]],
            seams: BTreeSet::new(),
        }
    }
    #[test]
    fn planar_quad_unfolds_as_one_patch() {
        let u=quad().unfold().unwrap();
        assert_eq!(u.patches.len(),1);
        assert_eq!(u.patches[0].triangles.len(),2);
        assert!(u.svg().contains("<svg"));
    }
    #[test]
    fn seam_splits_connected_triangles() {
        let mut mesh=quad();
        mesh.seams.insert((0,2));
        let u=mesh.unfold().unwrap();
        assert_eq!(u.patches.len(),2);
        assert!(u.patches.iter().all(|p|p.triangles.len()==1));
    }
    #[test]
    fn bent_hinge_keeps_triangle_edge_lengths() {
        let mut mesh=quad();
        mesh.vertices[3].z=25.0;
        let u=mesh.unfold().unwrap();
        assert_eq!(u.patches.len(),1);
        for (fi,t) in u.patches[0].face_ids.iter().zip(&u.patches[0].triangles) {
            let f=mesh.faces[*fi];
            for j in 0..3 {
                let len=norm(delta(t[j],t[(j+1)%3]));
                assert!((len-mesh.vertices[f[j]].distance(mesh.vertices[f[(j+1)%3]])).abs()<1e-7);
            }
        }
    }
    #[test]
    fn nonmanifold_edge_rejected() {
        let mut mesh=quad();
        mesh.faces.push([0,2,1]);
        assert!(mesh.validate().is_err());
    }
}

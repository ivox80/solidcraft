//! Surface-patch topology inspection for the future B-Rep shell stage.
//! This is sampled G0 edge matching, NOT exact NURBS curve intersection,
//! trimming, healing, manifold solid validation, or a full B-Rep structure.
use crate::nurbs::NurbsSurface;
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

const BOUNDARIES: [&str; 4] = ["u_min", "u_max", "v_min", "v_max"];
const SAMPLES: usize = 9;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "wasm", derive(serde::Serialize, serde::Deserialize))]
pub struct Seam {
    pub face_a: usize,
    pub edge_a: String,
    pub face_b: usize,
    pub edge_b: String,
    pub reversed: bool,
    pub max_gap: f64,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "wasm", derive(serde::Serialize, serde::Deserialize))]
pub struct BoundaryEdge {
    pub face: usize,
    pub edge: String,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "wasm", derive(serde::Serialize, serde::Deserialize))]
pub struct PatchTopology {
    pub face_count: usize,
    pub seam_count: usize,
    pub open_boundary_count: usize,
    pub nonmanifold_edge_groups: usize,
    pub connected_components: usize,
    pub seams: Vec<Seam>,
    pub open_boundaries: Vec<BoundaryEdge>,
}

#[derive(Clone)]
struct Edge {
    face: usize,
    side: usize,
    samples: Vec<[f64; 3]>,
}

fn gap(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    ((a[0]-b[0]).powi(2)+(a[1]-b[1]).powi(2)+(a[2]-b[2]).powi(2)).sqrt()
}

/// Return (is_reversed, maximum sampled distance) if two boundaries coincide.
fn match_edges(a: &Edge, b: &Edge, tolerance: f64) -> Option<(bool, f64)> {
    let compare = |reverse: bool| -> Option<f64> {
        let mut worst: f64 = 0.;
        for i in 0..SAMPLES {
            let j = if reverse { SAMPLES-1-i } else { i };
            worst = worst.max(gap(&a.samples[i], &b.samples[j]));
            if worst > tolerance { return None; }
        }
        Some(worst)
    };
    match (compare(false), compare(true)) {
        (Some(same), Some(reversed)) if reversed < same => Some((true, reversed)),
        (Some(same), _) => Some((false, same)),
        (None, Some(reversed)) => Some((true, reversed)),
        _ => None,
    }
}
fn boundary_point(surface: &NurbsSurface, side: usize, t: f64) -> Result<[f64; 3], &'static str> {
    let (ud, vd) = surface.domain();
    let u = ud[0] + (ud[1]-ud[0])*t;
    let v = vd[0] + (vd[1]-vd[0])*t;
    match side {
        0 => surface.evaluate(ud[0], v),
        1 => surface.evaluate(ud[1], v),
        2 => surface.evaluate(u, vd[0]),
        3 => surface.evaluate(u, vd[1]),
        _ => Err("unknown surface boundary"),
    }
}

/// Match all four boundaries of every untrimmed NURBS patch using sampled
/// distance. A paired seam is evidence of G0 adjacency, not a solid.
pub fn analyze_patches(surfaces: &[NurbsSurface], tolerance: f64)
    -> Result<PatchTopology, &'static str>
{
    if surfaces.is_empty() || surfaces.len() > 48 { return Err("invalid patch count"); }
    if !tolerance.is_finite() || !(1e-8..=1e-2).contains(&tolerance) {
        return Err("edge tolerance must be between 1e-8 and 1e-2");
    }
    let mut edges = Vec::with_capacity(surfaces.len()*4);
    for (face, surface) in surfaces.iter().enumerate() {
        surface.validate()?;
        for side in 0..4 {
            let mut samples=Vec::with_capacity(SAMPLES);
            for i in 0..SAMPLES {
                samples.push(boundary_point(surface,side,i as f64/(SAMPLES-1) as f64)?);
            }
            edges.push(Edge{face,side,samples});
        }
    }
    let mut assigned=vec![false;edges.len()];
    let mut seams=Vec::new();
    let mut open=Vec::new();
    let mut nonmanifold=0;
    let mut adjacency=vec![Vec::<usize>::new();surfaces.len()];
    for i in 0..edges.len() {
        if assigned[i] {continue;}
        assigned[i]=true;
        let mut group=vec![i];
        for (j, other) in edges.iter().enumerate().skip(i+1) {
            if !assigned[j] && match_edges(&edges[i],other,tolerance).is_some(){
                assigned[j]=true;
                group.push(j);
            }
        }
        if group.len()==1 {
            let e=&edges[i];
            open.push(BoundaryEdge{face:e.face,edge:BOUNDARIES[e.side].into()});
        } else {
            for &other in group.iter().skip(1) {
                let a=&edges[i];
                let b=&edges[other];
                // Mark connections even for ambiguous non-manifold groups.
                adjacency[a.face].push(b.face);
                adjacency[b.face].push(a.face);
                if group.len()==2 {
                    let (reversed,max_gap)=match_edges(a,b,tolerance).unwrap();
                    seams.push(Seam{
                        face_a:a.face,edge_a:BOUNDARIES[a.side].into(),
                        face_b:b.face,edge_b:BOUNDARIES[b.side].into(),
                        reversed,max_gap,
                    });
                }
            }
            if group.len()>2 {nonmanifold+=1;}
        }
    }
    let mut seen=vec![false;surfaces.len()];
    let mut components=0;
    for root in 0..surfaces.len(){
        if seen[root] {continue;}
        components+=1;
        let mut stack=vec![root];seen[root]=true;
        while let Some(face)=stack.pop(){
            for &other in &adjacency[face] {
                if !seen[other] {seen[other]=true;stack.push(other);}
            }
        }
    }
    Ok(PatchTopology{
        face_count:surfaces.len(),
        seam_count:seams.len(),
        open_boundary_count:open.len(),
        nonmanifold_edge_groups:nonmanifold,
        connected_components:components,
        seams,
        open_boundaries:open,
    })
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn nurbs_patch_topology_wasm(input: JsValue, tolerance: f64) -> Result<JsValue, JsValue> {
    let surfaces: Vec<NurbsSurface> = serde_wasm_bindgen::from_value(input)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let topology = analyze_patches(&surfaces, tolerance).map_err(JsValue::from_str)?;
    serde_wasm_bindgen::to_value(&topology)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn standalone_panel_has_four_open_boundaries() {
        let report=analyze_patches(&[NurbsSurface::curved_panel()],1e-6).unwrap();
        assert_eq!(report.seam_count,0);
        assert_eq!(report.open_boundary_count,4);
        assert_eq!(report.connected_components,1);
    }
    #[test]
    fn cylinder_quarters_have_four_seams_but_no_caps() {
        let patches=NurbsSurface::cylinder_quarters(35.,-45.,45.).unwrap();
        let report=analyze_patches(&patches,1e-6).unwrap();
        assert_eq!(report.face_count,4);
        assert_eq!(report.seam_count,4);
        assert_eq!(report.open_boundary_count,8);
        assert_eq!(report.nonmanifold_edge_groups,0);
        assert_eq!(report.connected_components,1);
        assert!(report.seams.iter().all(|s|s.max_gap<1e-8));
    }
    #[test]
    fn broken_seam_is_detected_after_editing() {
        let mut patches=NurbsSurface::cylinder_quarters(35.,-45.,45.).unwrap();
        patches[1]=patches[1].edited_control(0,0,[36.,0.,-45.],1.).unwrap();
        let report=analyze_patches(&patches,1e-6).unwrap();
        assert_eq!(report.seam_count,3);
        assert_eq!(report.open_boundary_count,10);
    }
    #[test]
    fn invalid_tolerance_is_rejected() {
        let patches=[NurbsSurface::curved_panel()];
        assert!(analyze_patches(&patches,0.).is_err());
        assert!(analyze_patches(&patches,f64::NAN).is_err());
    }
}

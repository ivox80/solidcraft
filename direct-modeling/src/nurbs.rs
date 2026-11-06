//! Tensor-product rational B-spline (NURBS) surface evaluation.
//!
//! v0.4 is a surface-only geometry foundation. Meshes here are *display tessellations*;
//! they are not CAD solids, and cannot yet participate in the v0.3 Boolean kernel.
use std::f64::consts::{FRAC_PI_2, FRAC_1_SQRT_2};
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

type Vec3 = [f64; 3];
const EPS: f64 = 1.0e-10;

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "wasm", derive(serde::Serialize, serde::Deserialize))]
pub struct NurbsSurface {
    pub degree_u: usize,
    pub degree_v: usize,
    pub knots_u: Vec<f64>,
    pub knots_v: Vec<f64>,
    /// The first index is U, the second index is V.
    pub control_points: Vec<Vec<Vec3>>,
    pub weights: Vec<Vec<f64>>,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "wasm", derive(serde::Serialize, serde::Deserialize))]
pub struct SurfaceMesh {
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub indices: Vec<u32>,
}

fn add(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn scale(a: Vec3, f: f64) -> Vec3 {
    [a[0] * f, a[1] * f, a[2] * f]
}
fn dot(a: Vec3, b: Vec3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn magnitude(a: Vec3) -> f64 {
    dot(a, a).sqrt()
}

fn check_knots(knots: &[f64], count: usize, degree: usize) -> Result<(), &'static str> {
    if !(1..=5).contains(&degree) || count < degree + 1 || count > 20 {
        return Err("degree/control-point count out of supported range");
    }
    if knots.len() != count + degree + 1 || knots.iter().any(|v| !v.is_finite()) {
        return Err("invalid knot vector length or values");
    }
    if knots.windows(2).any(|w| w[0] > w[1]) {
        return Err("knot vector must be nondecreasing");
    }
    if knots[count] - knots[degree] <= EPS {
        return Err("knot domain has zero length");
    }
    // Clamping makes the control handles intuitive, especially at patch boundaries.
    if (0..=degree).any(|i| knots[i] != knots[0]
        || knots[count + i] != knots[count]) {
        return Err("only clamped NURBS knot vectors are supported");
    }
    Ok(())
}

fn find_span(knots: &[f64], count: usize, degree: usize, t: f64) -> usize {
    if t >= knots[count] {
        return count - 1;
    }
    let mut low = degree;
    let mut high = count;
    let mut middle = (low + high) / 2;
    while t < knots[middle] || t >= knots[middle + 1] {
        if t < knots[middle] {
            high = middle;
        } else {
            low = middle;
        }
        middle = (low + high) / 2;
    }
    middle
}

/// Nonzero B-spline basis values (The NURBS Book, algorithm A2.2).
fn basis(span: usize, t: f64, degree: usize, knots: &[f64]) -> Vec<f64> {
    let mut values = vec![0.0; degree + 1];
    let mut left = vec![0.0; degree + 1];
    let mut right = vec![0.0; degree + 1];
    values[0] = 1.0;
    for j in 1..=degree {
        left[j] = t - knots[span + 1 - j];
        right[j] = knots[span + j] - t;
        let mut saved = 0.0;
        for r in 0..j {
            let denominator = right[r + 1] + left[j - r];
            let term = if denominator.abs() < EPS {
                0.0
            } else {
                values[r] / denominator
            };
            values[r] = saved + right[r + 1] * term;
            saved = left[j - r] * term;
        }
        values[j] = saved;
    }
    values
}

impl NurbsSurface {
    pub fn validate(&self) -> Result<(), &'static str> {
        let nu = self.control_points.len();
        let nv = self.control_points.first().map_or(0, Vec::len);
        check_knots(&self.knots_u, nu, self.degree_u)?;
        check_knots(&self.knots_v, nv, self.degree_v)?;
        if self.weights.len() != nu
            || self.weights.iter().any(|row| row.len() != nv)
            || self.control_points.iter().any(|row| row.len() != nv)
        {
            return Err("control-point and weight grid dimensions differ");
        }
        if self.control_points.iter().flatten().flatten().any(|v| !v.is_finite()) {
            return Err("control points must be finite");
        }
        if self.weights.iter().flatten().any(|&w| !w.is_finite() || w <= EPS) {
            return Err("weights must be positive and finite");
        }
        Ok(())
    }

    pub fn domain(&self) -> ([f64; 2], [f64; 2]) {
        (
            [self.knots_u[self.degree_u], self.knots_u[self.control_points.len()]],
            [self.knots_v[self.degree_v], self.knots_v[self.control_points[0].len()]],
        )
    }

    /// Evaluate a rational tensor-product B-spline point (not a mesh approximation).
    pub fn evaluate(&self, u: f64, v: f64) -> Result<Vec3, &'static str> {
        if !u.is_finite() || !v.is_finite() {
            return Err("parameters must be finite");
        }
        let (ud, vd) = self.domain();
        if u < ud[0] - EPS || u > ud[1] + EPS
            || v < vd[0] - EPS || v > vd[1] + EPS
        {
            return Err("parameters outside surface domain");
        }
        let u = u.clamp(ud[0], ud[1]);
        let v = v.clamp(vd[0], vd[1]);
        let us = find_span(&self.knots_u, self.control_points.len(), self.degree_u, u);
        let vs = find_span(&self.knots_v, self.control_points[0].len(), self.degree_v, v);
        let ub = basis(us, u, self.degree_u, &self.knots_u);
        let vb = basis(vs, v, self.degree_v, &self.knots_v);
        let mut numerator = [0., 0., 0.];
        let mut denominator = 0.0;
        for (i, &ubasis) in ub.iter().enumerate() {
            for (j, &vbasis) in vb.iter().enumerate() {
                let ci = us - self.degree_u + i;
                let cj = vs - self.degree_v + j;
                let factor = ubasis * vbasis * self.weights[ci][cj];
                numerator = add(numerator, scale(self.control_points[ci][cj], factor));
                denominator += factor;
            }
        }
        if denominator <= EPS || !denominator.is_finite() {
            return Err("invalid rational denominator");
        }
        Ok(scale(numerator, 1.0 / denominator))
    }

    /// Finite-difference normal for viewport shading; the surface itself is exact.
    pub fn normal(&self, u: f64, v: f64) -> Result<Vec3, &'static str> {
        let (ud, vd) = self.domain();
        let hu = (ud[1] - ud[0]) * 1e-4;
        let hv = (vd[1] - vd[0]) * 1e-4;
        let pu0 = self.evaluate((u - hu).max(ud[0]), v)?;
        let pu1 = self.evaluate((u + hu).min(ud[1]), v)?;
        let pv0 = self.evaluate(u, (v - hv).max(vd[0]))?;
        let pv1 = self.evaluate(u, (v + hv).min(vd[1]))?;
        let tangent_u = sub(pu1, pu0);
        let tangent_v = sub(pv1, pv0);
        let n = cross(tangent_u, tangent_v);
        let mag = magnitude(n);
        if !mag.is_finite() || mag <= EPS {
            return Err("degenerate surface normal");
        }
        Ok(scale(n, 1.0 / mag))
    }

    /// Uniform tessellation for display only. Rendering quality is adjustable
    /// without changing the underlying rational surface.
    pub fn tessellate(&self, steps_u: usize, steps_v: usize) -> Result<SurfaceMesh, &'static str> {
        self.validate()?;
        if !(2..=128).contains(&steps_u) || !(2..=128).contains(&steps_v) {
            return Err("tessellation segments must be between 2 and 128");
        }
        let (ud, vd) = self.domain();
        let mut positions = Vec::with_capacity((steps_u + 1) * (steps_v + 1));
        let mut normals = Vec::with_capacity((steps_u + 1) * (steps_v + 1));
        let mut indices = Vec::with_capacity(steps_u * steps_v * 6);
        for i in 0..=steps_u {
            let u = ud[0] + (ud[1] - ud[0]) * (i as f64 / steps_u as f64);
            for j in 0..=steps_v {
                let v = vd[0] + (vd[1] - vd[0]) * (j as f64 / steps_v as f64);
                positions.push(self.evaluate(u, v)?);
                normals.push(self.normal(u, v)?);
            }
        }
        let row = steps_v + 1;
        for i in 0..steps_u {
            for j in 0..steps_v {
                let a = (i * row + j) as u32;
                let b = ((i + 1) * row + j) as u32;
                let c = b + 1;
                let d = a + 1;
                indices.extend([a, b, c, a, c, d]);
            }
        }
        Ok(SurfaceMesh { positions, normals, indices })
    }

    pub fn edited_control(&self, i: usize, j: usize, point: Vec3, weight: f64)
        -> Result<Self, &'static str>
    {
        let mut result = self.clone();
        if i >= result.control_points.len() || j >= result.control_points[0].len() {
            return Err("control-point index out of range");
        }
        result.control_points[i][j] = point;
        result.weights[i][j] = weight;
        result.validate()?;
        Ok(result)
    }

    pub fn curved_panel() -> Self {
        let x = [-40.0, -15.0, 15.0, 40.0];
        let y = [-30.0, -10.0, 10.0, 30.0];
        let mut control_points = vec![vec![[0.; 3]; 4]; 4];
        for (i, &cx) in x.iter().enumerate() {
            for (j, &cy) in y.iter().enumerate() {
                let z = if i == 0 || i == 3 || j == 0 || j == 3 { 0.0 } else { 35.0 };
                control_points[i][j] = [cx, cy, z];
            }
        }
        Self {
            degree_u: 3,
            degree_v: 3,
            knots_u: vec![0., 0., 0., 0., 1., 1., 1., 1.],
            knots_v: vec![0., 0., 0., 0., 1., 1., 1., 1.],
            control_points,
            weights: vec![vec![1.0; 4]; 4],
        }
    }

    /// Four exact rational quadratic quarter-cylinder patches (surface only,
    /// not a watertight closed solid with caps). Z is the cylinder axis.
    pub fn cylinder_quarters(radius: f64, z0: f64, z1: f64)
        -> Result<Vec<Self>, &'static str>
    {
        if ![radius, z0, z1].iter().all(|x| x.is_finite()) ||
            radius <= EPS || z1 <= z0 {
            return Err("invalid cylinder parameters");
        }
        let mut result = Vec::with_capacity(4);
        for quadrant in 0..4 {
            let start = quadrant as f64 * FRAC_PI_2;
            let mid = start + FRAC_PI_2 * 0.5;
            let end = start + FRAC_PI_2;
            let angles = [start, mid, end];
            let weights = [1.0, FRAC_1_SQRT_2, 1.0];
            let mut cp = Vec::with_capacity(3);
            for (i, &angle) in angles.iter().enumerate() {
                let r = if i == 1 { radius / FRAC_1_SQRT_2 } else { radius };
                cp.push(vec![
                    [r * angle.cos(), r * angle.sin(), z0],
                    [r * angle.cos(), r * angle.sin(), z1],
                ]);
            }
            result.push(Self {
                degree_u: 2,
                degree_v: 1,
                knots_u: vec![0., 0., 0., 1., 1., 1.],
                knots_v: vec![0., 0., 1., 1.],
                control_points: cp,
                weights: weights.iter().map(|&w| vec![w, w]).collect(),
            });
        }
        Ok(result)
    }
}

#[cfg(feature = "wasm")]
fn from_js(value: JsValue) -> Result<NurbsSurface, JsValue> {
    let s: NurbsSurface = serde_wasm_bindgen::from_value(value)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    s.validate().map_err(JsValue::from_str)?;
    Ok(s)
}
#[cfg(feature = "wasm")]
fn to_js<T: serde::Serialize>(value: &T) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(value).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn nurbs_panel_wasm() -> Result<JsValue, JsValue> {
    to_js(&NurbsSurface::curved_panel())
}
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn nurbs_cylinder_wasm(radius: f64, z0: f64, z1: f64) -> Result<JsValue, JsValue> {
    to_js(&NurbsSurface::cylinder_quarters(radius, z0, z1).map_err(JsValue::from_str)?)
}
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn nurbs_evaluate_wasm(value: JsValue, u: f64, v: f64) -> Result<JsValue, JsValue> {
    to_js(&from_js(value)?.evaluate(u, v).map_err(JsValue::from_str)?)
}
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn nurbs_tessellate_wasm(value: JsValue, u_steps: usize, v_steps: usize)
    -> Result<JsValue, JsValue>
{
    to_js(&from_js(value)?.tessellate(u_steps, v_steps).map_err(JsValue::from_str)?)
}
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn nurbs_edit_control_wasm(value: JsValue, iu: usize, iv: usize, point: JsValue, weight: f64)
    -> Result<JsValue, JsValue>
{
    let p: Vec3 = serde_wasm_bindgen::from_value(point)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    to_js(&from_js(value)?.edited_control(iu, iv, p, weight).map_err(JsValue::from_str)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn approximately(a: f64, b: f64) {
        assert!((a - b).abs() < 1.0e-6, "{a} is not approximately {b}");
    }
    #[test]
    fn panel_corners_match_control_points() {
        let s = NurbsSurface::curved_panel();
        s.validate().unwrap();
        assert_eq!(s.evaluate(0., 0.).unwrap(), [-40., -30., 0.]);
        assert_eq!(s.evaluate(1., 1.).unwrap(), [40., 30., 0.]);
        assert!(s.evaluate(0.5, 0.5).unwrap()[2] > 8.0);
    }
    #[test]
    fn edited_control_changes_surface_without_mutating_original() {
        let s = NurbsSurface::curved_panel();
        let old_z = s.evaluate(0.5, 0.5).unwrap()[2];
        let changed = s.edited_control(1, 1, [-15., -10., 75.], 1.).unwrap();
        assert!(changed.evaluate(0.5, 0.5).unwrap()[2] > old_z);
        assert_eq!(s.control_points[1][1][2], 35.);
        assert!(s.edited_control(1, 1, [0., 0., 0.], -1.).is_err());
    }
    #[test]
    fn rational_quadratic_exact_cylinder_radial_distance() {
        let quarters = NurbsSurface::cylinder_quarters(10., -5., 5.).unwrap();
        for patch in &quarters {
            patch.validate().unwrap();
            for i in 0..=10 {
                let u = i as f64 / 10.0;
                let p = patch.evaluate(u, 0.4).unwrap();
                approximately((p[0] * p[0] + p[1] * p[1]).sqrt(), 10.);
                approximately(p[2], -1.);
            }
        }
        for q in 0..4 {
            let a = quarters[q].evaluate(1., 0.37).unwrap();
            let b = quarters[(q + 1) % 4].evaluate(0., 0.37).unwrap();
            for i in 0..3 { approximately(a[i], b[i]); }
        }
    }
    #[test]
    fn tessellation_indices_and_normals_are_consistent() {
        let mesh = NurbsSurface::curved_panel().tessellate(18, 12).unwrap();
        assert_eq!(mesh.positions.len(), 19 * 13);
        assert_eq!(mesh.normals.len(), 19 * 13);
        assert_eq!(mesh.indices.len(), 18 * 12 * 6);
        assert!(mesh.indices.iter().all(|&i| (i as usize) < mesh.positions.len()));
        for n in mesh.normals {
            approximately(magnitude(n), 1.);
        }
    }
    #[test]
    fn invalid_knots_and_nonfinite_points_are_rejected() {
        let mut s = NurbsSurface::curved_panel();
        s.knots_u[4] = -1.0;
        assert!(s.validate().is_err());
        let mut s = NurbsSurface::curved_panel();
        s.control_points[0][0][0] = f64::NAN;
        assert!(s.validate().is_err());
        let mut s = NurbsSurface::curved_panel();
        s.weights[0][0] = 0.;
        assert!(s.validate().is_err());
    }
    #[test]
    fn out_of_domain_or_invalid_tessellation_is_rejected() {
        let s = NurbsSurface::curved_panel();
        assert!(s.evaluate(-0.1, 0.5).is_err());
        assert!(s.tessellate(1, 20).is_err());
        assert!(s.tessellate(20, 129).is_err());
    }
}

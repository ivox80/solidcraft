//! Experimental SolidCraft direct modeling kernel.
//! Includes axis-aligned boxes (v0.2) and convex polygonal solids (v0.3). Not NURBS.

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "wasm", derive(serde::Serialize, serde::Deserialize))]
pub struct Box3 {
    pub min: [f64; 3],
    pub max: [f64; 3],
}

impl Box3 {
    pub fn new(min: [f64; 3], max: [f64; 3]) -> Result<Self, &'static str> {
        if min.iter().chain(max.iter()).any(|x| !x.is_finite()) {
            return Err("coordinates must be finite");
        }
        if (0..3).any(|i| min[i] >= max[i]) {
            return Err("min must be strictly less than max");
        }
        Ok(Self { min, max })
    }

    pub fn volume(&self) -> f64 {
        (0..3).map(|i| self.max[i] - self.min[i]).product()
    }

    pub fn intersection(&self, other: &Self) -> Option<Self> {
        let min = std::array::from_fn(|i| self.min[i].max(other.min[i]));
        let max = std::array::from_fn(|i| self.max[i].min(other.max[i]));
        Self::new(min, max).ok()
    }

    /// Move one of the six planar faces; axis=0,1,2; positive=true for max face.
    pub fn move_face(&mut self, axis: usize, positive: bool, delta: f64) -> Result<(), &'static str> {
        if axis >= 3 || !delta.is_finite() {
            return Err("invalid axis or delta");
        }
        let mut next = *self;
        if positive {
            next.max[axis] += delta;
        } else {
            next.min[axis] += delta;
        }
        *self = Self::new(next.min, next.max)?;
        Ok(())
    }

    /// Return disjoint boxes representing self minus cutter (maximum six).
    pub fn subtract(&self, cutter: &Self) -> Vec<Self> {
        let Some(i) = self.intersection(cutter) else { return vec![*self]; };
        let mut out = Vec::new();
        let mut add = |min, max| {
            if let Ok(b) = Self::new(min, max) { out.push(b); }
        };
        // Split into left/right, front/back, lower/upper slabs without overlap.
        add(self.min, [i.min[0], self.max[1], self.max[2]]);
        add([i.max[0], self.min[1], self.min[2]], self.max);
        let x0 = i.min[0]; let x1 = i.max[0];
        add([x0, self.min[1], self.min[2]], [x1, i.min[1], self.max[2]]);
        add([x0, i.max[1], self.min[2]], [x1, self.max[1], self.max[2]]);
        let y0 = i.min[1]; let y1 = i.max[1];
        add([x0, y0, self.min[2]], [x1, y1, i.min[2]]);
        add([x0, y0, i.max[2]], [x1, y1, self.max[2]]);
        out
    }
}

/// Apply multiple cutters to multiple targets. Outputs are disjoint per target.
pub fn subtract_many(targets: &[Box3], cutters: &[Box3]) -> Vec<Vec<Box3>> {
    targets.iter().map(|target| {
        cutters.iter().fold(vec![*target], |parts, cutter| {
            parts.iter().flat_map(|b| b.subtract(cutter)).collect()
        })
    }).collect()
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn subtract_many_wasm(targets: JsValue, cutters: JsValue) -> Result<JsValue, JsValue> {
    let targets: Vec<Box3> = serde_wasm_bindgen::from_value(targets)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let cutters: Vec<Box3> = serde_wasm_bindgen::from_value(cutters)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    for b in targets.iter().chain(cutters.iter()) {
        Box3::new(b.min, b.max).map_err(JsValue::from_str)?;
    }
    serde_wasm_bindgen::to_value(&subtract_many(&targets, &cutters))
        .map_err(|e| JsValue::from_str(&e.to_string()))
}


/// Move a planar face of one box in the browser.
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn move_face_wasm(value: JsValue, axis: usize, positive: bool, delta: f64) -> Result<JsValue, JsValue> {
    let mut b: Box3 = serde_wasm_bindgen::from_value(value)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    b = Box3::new(b.min, b.max).map_err(JsValue::from_str)?;
    b.move_face(axis, positive, delta).map_err(JsValue::from_str)?;
    serde_wasm_bindgen::to_value(&b).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn b(min: [f64; 3], max: [f64; 3]) -> Box3 { Box3::new(min, max).unwrap() }
    fn total(parts: &[Box3]) -> f64 { parts.iter().map(Box3::volume).sum() }

    #[test]
    fn face_edit_is_transactional() {
        let mut x = b([0., 0., 0.], [1., 1., 1.]);
        assert!(x.move_face(0, true, -2.).is_err());
        assert_eq!(x.max[0], 1.);
        x.move_face(0, true, 1.).unwrap();
        assert_eq!(x.volume(), 2.);
    }
    #[test]
    fn subtraction_conserves_volume() {
        let target = b([0., 0., 0.], [10., 10., 10.]);
        let cutter = b([2., 2., 2.], [8., 8., 8.]);
        let parts = target.subtract(&cutter);
        assert_eq!(parts.len(), 6);
        assert!((total(&parts) - 784.).abs() < 1e-9);
        for (a, p) in parts.iter().enumerate() {
            for q in parts.iter().skip(a+1) {
                assert!(p.intersection(q).is_none());
            }
        }
    }
    #[test]
    fn no_overlap_keeps_target() {
        let target = b([0., 0., 0.], [1., 1., 1.]);
        assert_eq!(target.subtract(&b([2., 2., 2.], [3., 3., 3.])), vec![target]);
    }
    #[test]
    fn fully_covered_is_empty() {
        assert!(b([0., 0., 0.], [1., 1., 1.]).subtract(&b([-1., -1., -1.], [2., 2., 2.])).is_empty());
    }
    #[test]
    fn multiple_cutters_are_applied() {
        let t = b([0., 0., 0.], [10., 10., 10.]);
        let c1 = b([0., 0., 0.], [2., 10., 10.]);
        let c2 = b([8., 0., 0.], [10., 10., 10.]);
        assert!((total(&subtract_many(&[t], &[c1, c2])[0]) - 600.).abs() < 1e-9);
    }
    #[test]
    fn rejects_invalid_coordinates() {
        assert!(Box3::new([0., 0., 0.], [f64::NAN, 1., 1.]).is_err());
        assert!(Box3::new([0., 0., 0.], [0., 1., 1.]).is_err());
    }
}

pub mod poly;

pub mod nurbs;
pub mod nurbs_shell;

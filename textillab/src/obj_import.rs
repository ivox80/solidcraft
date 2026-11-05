//! Wavefront OBJ mesh importer for textile panel unfolding.
//!
//! Uses the geometric position indices in faces; texture/normal indices are ignored.
//! OBJ files do not specify physical units. Coordinates are passed through unchanged.

use crate::unfold::{TriangleMesh, Vec3};
use std::collections::{BTreeMap, BTreeSet};

fn face_index(token: &str, count: usize, line_number: usize) -> Result<usize, String> {
    let raw = token.split('/').next().unwrap_or("");
    let n: i64 = raw
        .parse()
        .map_err(|_| format!("Line {line_number}: invalid OBJ vertex index '{token}'"))?;
    if n == 0 {
        return Err(format!("Line {line_number}: OBJ index 0 is invalid"));
    }
    let index = if n > 0 { n - 1 } else { count as i64 + n };
    if index < 0 || index >= count as i64 {
        return Err(format!("Line {line_number}: OBJ index out of bounds: {n}"));
    }
    Ok(index as usize)
}

pub fn parse_obj(source: &str) -> Result<TriangleMesh, String> {
    let mut vertices = Vec::new();
    let mut faces = Vec::new();
    for (line_idx, original) in source.lines().enumerate() {
        let line_number = line_idx + 1;
        let cleaned = original.split('#').next().unwrap_or("").trim();
        let mut words = cleaned.split_whitespace();
        match words.next() {
            Some("v") => {
                let mut position = [0.0_f64; 3];
                for value in &mut position {
                    let s = words.next().ok_or_else(|| {
                        format!("Line {line_number}: OBJ vertex needs 3 coordinates")
                    })?;
                    *value = s.parse().map_err(|_| {
                        format!("Line {line_number}: invalid vertex coordinate '{s}'")
                    })?;
                    if !value.is_finite() {
                        return Err(format!("Line {line_number}: non-finite coordinate"));
                    }
                }
                vertices.push(Vec3::new(position[0], position[1], position[2]));
            }
            Some("f") => {
                let indices: Vec<usize> = words
                    .map(|word| face_index(word, vertices.len(), line_number))
                    .collect::<Result<_, _>>()?;
                if indices.len() < 3 {
                    return Err(format!("Line {line_number}: face requires 3+ vertices"));
                }
                // Fan triangulation is valid for convex polygons. Restrict this
                // importer to triangles and quads to avoid misreading concave n-gons.
                if indices.len() > 4 {
                    return Err(format!(
                        "Line {line_number}: only triangles and quads supported"
                    ));
                }
                faces.push([indices[0], indices[1], indices[2]]);
                if indices.len() == 4 {
                    faces.push([indices[0], indices[2], indices[3]]);
                }
            }
            _ => {}
        }
    }
    let mesh = TriangleMesh {
        vertices,
        faces,
        seams: BTreeSet::new(),
    };
    mesh.validate()?;
    Ok(mesh)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EdgeInfo {
    pub a: usize,
    pub b: usize,
    pub adjacent_faces: Vec<usize>,
}

pub fn mesh_edges(mesh: &TriangleMesh) -> Vec<EdgeInfo> {
    let mut edges: BTreeMap<(usize, usize), Vec<usize>> = BTreeMap::new();
    for (face_id, face) in mesh.faces.iter().enumerate() {
        for i in 0..3 {
            let a = face[i].min(face[(i + 1) % 3]);
            let b = face[i].max(face[(i + 1) % 3]);
            edges.entry((a, b)).or_default().push(face_id);
        }
    }
    edges
        .into_iter()
        .map(|((a, b), adjacent_faces)| EdgeInfo {
            a,
            b,
            adjacent_faces,
        })
        .collect()
}

/// User-facing vertex numbers use OBJ's 1-based indexing.
pub fn parse_seam(specification: &str, mesh: &TriangleMesh) -> Result<(usize, usize), String> {
    let (a, b) = specification
        .trim()
        .split_once(':')
        .ok_or("Use the seam format 'vertexA:vertexB', e.g. 1:3")?;
    let parse = |s: &str| -> Result<usize, String> {
        let index: usize = s
            .trim()
            .parse()
            .map_err(|_| format!("Invalid seam vertex '{s}'"))?;
        if index == 0 || index > mesh.vertices.len() {
            return Err(format!("Seam vertex {index} is out of range"));
        }
        Ok(index - 1)
    };
    let a = parse(a)?;
    let b = parse(b)?;
    if a == b {
        return Err("Seam endpoints must be different".into());
    }
    let edge = (a.min(b), a.max(b));
    if !mesh_edges(mesh)
        .iter()
        .any(|e| (e.a, e.b) == edge)
    {
        return Err(format!("Seam {}:{} is not a mesh edge", a + 1, b + 1));
    }
    Ok(edge)
}

#[cfg(test)]
mod tests {
    use super::*;

    const QUAD: &str = "# 100 x 60 mm\nv 0 0 0\nv 100 0 0\nv 100 60 0\nv 0 60 0\nf 1 2 3 4\n";

    #[test]
    fn quad_import_and_flatten() {
        let mesh = parse_obj(QUAD).unwrap();
        assert_eq!(mesh.faces.len(), 2);
        assert_eq!(mesh.unfold().unwrap().patches.len(), 1);
        assert_eq!(mesh_edges(&mesh).len(), 5);
    }

    #[test]
    fn support_negative_and_slash_indices() {
        let mesh = parse_obj("v 0 0 0\nv 1 0 0\nv 0 1 0\nf -3/1/1 -2/2/1 -1/3/1").unwrap();
        assert_eq!(mesh.faces[0], [0, 1, 2]);
    }

    #[test]
    fn seam_selection_splits_faces() {
        let mut mesh = parse_obj(QUAD).unwrap();
        assert_eq!(parse_seam("1:3", &mesh).unwrap(), (0, 2));
        mesh.seams.insert((0, 2));
        assert_eq!(mesh.unfold().unwrap().patches.len(), 2);
    }

    #[test]
    fn reject_invalid_mesh_and_seam() {
        assert!(parse_obj("v 0 0 0\nf 1 2 3").is_err());
        assert!(parse_obj("v 0 0 0\nv 1 0 0\nv 0 1 0\nf 0 2 3").is_err());
        let mesh = parse_obj(QUAD).unwrap();
        assert!(parse_seam("1:99", &mesh).is_err());
        assert!(parse_seam("2:4", &mesh).is_err());
    }
}

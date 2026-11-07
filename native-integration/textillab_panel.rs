//! Native egui TextilLab panel for a custom SolidCraft source build.
//! Standalone OBJ source and optional sidecar JSON until document feature integration lands.

use egui::{Align2, Color32, Id, Pos2, Rect, Sense, Stroke, Vec2};
use serde_json::json;
use std::collections::BTreeSet;
use textillab::obj_import::{mesh_edges, parse_obj};
use textillab::studio_patterns::{make_cut_preview, patch_boundary};
use textillab::unfold::{TriangleMesh, Vec3};

pub struct PanelState {
    pub open: bool,
    path: String,
    output: String,
    seam_mm: f64,
    density_g_m2: f64,
    mesh: Option<TriangleMesh>,
    last_svg: String,
    status: String,
    pieces: usize,
    area_mm2: f64,
    mass_g: f64,
}

impl Default for PanelState {
    fn default() -> Self {
        Self {
            open: false,
            path: String::new(),
            output: "textillab-cut.svg".into(),
            seam_mm: 10.0,
            density_g_m2: 210.0,
            mesh: None,
            last_svg: String::new(),
            status: "OBJ laden, dann Schnittkanten im 3D-Netz anklicken".into(),
            pieces: 0,
            area_mm2: 0.0,
            mass_g: 0.0,
        }
    }
}

impl PanelState {
    pub fn show(&mut self, ctx: &egui::Context) {
        egui::Area::new(Id::new("textillab.native.launch"))
            .anchor(Align2::RIGHT_TOP, Vec2::new(-14.0, 159.0))
            .show(ctx, |ui| {
                if ui.button("TextilLab").clicked() {
                    self.open = !self.open;
                }
            });

        if !self.open {
            return;
        }
        let mut open = self.open;
        egui::Window::new("TextilLab · Native CAD Preview")
            .id(Id::new("textillab.native.window"))
            .open(&mut open)
            .default_width(810.0)
            .default_height(580.0)
            .resizable(true)
            .show(ctx, |ui| self.window(ui));
        self.open = open;
    }

    fn load(&mut self) {
        let source = match std::fs::read_to_string(&self.path) {
            Ok(source) => source,
            Err(error) => {
                self.status = format!("OBJ konnte nicht gelesen werden: {error}");
                return;
            }
        };
        match parse_obj(&source) {
            Ok(mesh) => {
                if mesh.faces.len() > 10_000 {
                    self.status = "Maximal 10.000 Dreiecke in der Vorschau".into();
                    return;
                }
                self.mesh = Some(mesh);
                self.refresh();
            }
            Err(error) => self.status = format!("Ungültiges OBJ: {error}"),
        }
    }

    fn refresh(&mut self) {
        let Some(mesh) = &self.mesh else { return };
        match make_cut_preview(mesh, self.seam_mm, self.density_g_m2) {
            Ok(result) => {
                self.pieces = result.pieces;
                self.area_mm2 = result.cut_area_mm2;
                self.mass_g = result.estimated_mass_g;
                self.last_svg = result.svg;
                self.status = "Schnittmuster neu berechnet".into();
            }
            Err(error) => {
                self.pieces = 0;
                self.last_svg.clear();
                self.status = format!("Abwicklung nicht möglich: {error}");
            }
        }
    }

    fn export(&mut self) {
        if self.last_svg.is_empty() {
            self.status = "Kein gültiges Schnittmuster zum Exportieren".into();
            return;
        }
        if let Err(error) = std::fs::write(&self.output, &self.last_svg) {
            self.status = format!("SVG konnte nicht gespeichert werden: {error}");
            return;
        }
        // Sidecar project: editable parameters, but not yet a native .sc timeline feature.
        let sidecar = format!("{}.textillab.json", self.output);
        let seams: Vec<[usize; 2]> = self
            .mesh
            .as_ref()
            .map(|mesh| {
                mesh.seams.iter().map(|&(a,b)| [a + 1, b + 1]).collect()
            })
            .unwrap_or_default();
        let project = json!({
            "schema": "textillab-sidecar/1",
            "source_obj": self.path,
            "seam_allowance_mm": self.seam_mm,
            "areal_density_g_m2": self.density_g_m2,
            "seams_obj_indices": seams
        });
        if let Err(error) = std::fs::write(&sidecar, project.to_string()) {
            self.status = format!("SVG gespeichert, Projektdatei fehlgeschlagen: {error}");
        } else {
            self.status = format!("SVG und Parameter-Projekt gespeichert: {sidecar}");
        }
    }

    fn window(&mut self, ui: &mut egui::Ui) {
        ui.label("3D-Kanten wählen, Schnittmuster berechnen und als SVG exportieren");
        ui.horizontal(|ui| {
            ui.label("OBJ:");
            ui.add(egui::TextEdit::singleline(&mut self.path).desired_width(460.0));
            if ui.button("Laden").clicked() {
                self.load();
            }
        });
        ui.horizontal(|ui| {
            ui.label("Nahtzugabe (mm)");
            let allowance = ui.add(egui::DragValue::new(&mut self.seam_mm).range(0.0..=200.0).speed(0.5));
            ui.separator();
            ui.label("Flächengewicht (g/m²)");
            let density = ui.add(egui::DragValue::new(&mut self.density_g_m2).range(1.0..=5000.0).speed(5.0));
            if allowance.changed() || density.changed() {
                self.refresh();
            }
        });
        ui.horizontal(|ui| {
            ui.label("Export:");
            ui.add(egui::TextEdit::singleline(&mut self.output).desired_width(300.0));
            if ui.add_enabled(!self.last_svg.is_empty(), egui::Button::new("SVG + Projekt speichern")).clicked() {
                self.export();
            }
            if ui.button("Schnittkanten löschen").clicked() {
                if let Some(mesh) = &mut self.mesh {
                    mesh.seams.clear();
                }
                self.refresh();
            }
        });
        ui.separator();
        ui.columns(2, |columns| {
            columns[0].heading("3D · Kanten wählen");
            columns[1].heading("2D · Zuschnittvorschau");
            let clicked = if let Some(mesh) = self.mesh.as_ref() {
                draw_mesh(&mut columns[0], mesh)
            } else {
                columns[0].label("OBJ laden, um eine 3D-Fläche anzuzeigen.");
                None
            };
            if let Some((a, b)) = clicked {
                if let Some(mesh) = &mut self.mesh {
                    if !mesh.seams.insert((a, b)) {
                        mesh.seams.remove(&(a, b));
                    }
                }
                self.refresh();
            }
            if let Some(mesh) = self.mesh.as_ref() {
                draw_cut_preview(&mut columns[1], mesh);
            } else {
                columns[1].label("Noch kein Schnittmuster.");
            }
        });
        ui.separator();
        if let Some(mesh) = &self.mesh {
            ui.label(format!("{} Dreiecke · {} Schnittkanten · {} Schnittteile · {:.4} m² Zuschnittfläche · {:.1} g",
                mesh.faces.len(), mesh.seams.len(), self.pieces, self.area_mm2 / 1_000_000.0, self.mass_g));
        }
        ui.label(&self.status);
        ui.small("Eigenständige OBJ-Textilfunktion im nativen SolidCraft-Fenster. Noch keine .sc-Timeline-Bindung.");
    }
}

fn project(v: Vec3) -> (f64, f64) {
    let yaw: f64 = 0.55;
    let pitch: f64 = 0.48;
    let x = v.x * yaw.cos() - v.y * yaw.sin();
    let y = v.x * yaw.sin() + v.y * yaw.cos();
    (x, y * pitch.cos() - v.z * pitch.sin())
}

fn scaled_points(points: &[(f64, f64)], rect: Rect) -> Vec<Pos2> {
    if points.is_empty() {
        return Vec::new();
    }
    let min_x = points.iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
    let max_x = points.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max);
    let min_y = points.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
    let max_y = points.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
    let width = (max_x - min_x).max(1e-6);
    let height = (max_y - min_y).max(1e-6);
    let scale = ((rect.width() - 28.0).max(10.0) as f64 / width)
        .min((rect.height() - 28.0).max(10.0) as f64 / height);
    points.iter().map(|(x,y)| {
        Pos2::new(
            rect.center().x + ((x - (min_x+max_x)*0.5)*scale) as f32,
            rect.center().y - ((y - (min_y+max_y)*0.5)*scale) as f32
        )
    }).collect()
}

fn segment_distance(p: Pos2, a: Pos2, b: Pos2) -> f32 {
    let direction = b - a;
    let length2 = direction.length_sq().max(1e-12);
    let t = ((p - a).dot(direction) / length2).clamp(0.0, 1.0);
    (p - a - direction*t).length()
}

/// Click near an interior triangle edge to toggle a cut seam.
fn draw_mesh(ui: &mut egui::Ui, mesh: &TriangleMesh) -> Option<(usize, usize)> {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(358.0, 310.0), Sense::click());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 4.0, Color32::from_rgb(23, 34, 46));
    let coords: Vec<_> = scaled_points(&mesh.vertices.iter().map(|v| project(*v)).collect::<Vec<_>>(), rect);
    let edges = mesh_edges(mesh);
    let mut chosen = None;
    let mouse = response.interact_pointer_pos();
    for edge in edges {
        let Some(&a) = coords.get(edge.a) else { continue };
        let Some(&b) = coords.get(edge.b) else { continue };
        let internal = edge.adjacent_faces.len() == 2;
        let seam = mesh.seams.contains(&(edge.a, edge.b));
        let hover = mouse.is_some_and(|p| segment_distance(p, a, b) < 8.0);
        let color = if seam {
            Color32::from_rgb(248, 131, 112)
        } else if hover && internal {
            Color32::from_rgb(250, 215, 129)
        } else if internal {
            Color32::from_rgb(119, 203, 218)
        } else {
            Color32::from_rgb(88, 127, 151)
        };
        painter.line_segment([a,b], Stroke::new(if seam {3.0} else {1.4}, color));
        if internal && hover && response.clicked() {
            chosen = Some((edge.a, edge.b));
        }
    }
    chosen
}

fn draw_cut_preview(ui: &mut egui::Ui, mesh: &TriangleMesh) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(358.0, 310.0), Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 4.0, Color32::from_rgb(19, 32, 42));
    let Ok(unfolded) = mesh.unfold() else {
        return;
    };
    let mut boundaries = Vec::new();
    for patch in &unfolded.patches {
        if let Ok(boundary) = patch_boundary(mesh, patch) {
            boundaries.push(boundary);
        }
    }
    let all: Vec<(f64,f64)> = boundaries.iter().flat_map(|b| b.iter().map(|p| (p.x, p.y))).collect();
    let projected = scaled_points(&all, rect);
    let mut start = 0;
    for polygon in boundaries {
        let end = start + polygon.len();
        if let Some(positions) = projected.get(start..end) {
            for i in 0..positions.len() {
                if let (Some(&a),Some(&b)) = (positions.get(i),positions.get((i+1)%positions.len())) {
                    painter.line_segment([a,b], Stroke::new(2.0, Color32::from_rgb(107, 211, 209)));
                }
            }
        }
        start=end;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn edge_hit_is_nonzero_away_from_segment() {
        let p=Pos2::new(50.0, 50.0);
        let a=Pos2::new(0.0, 0.0);
        let b=Pos2::new(100.0, 0.0);
        assert!((segment_distance(p, a, b) - 50.0).abs() < 1e-4);
    }
}

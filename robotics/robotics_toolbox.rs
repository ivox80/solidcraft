//! Native SolidCraft robotics toolbox (egui).
//! All edits flow through SolveApp::run, so native undo, joints and studies stay intact.
use crate::SolveApp;
use egui::{self, DragValue, RichText};
use serde_json::{json, Value};

const STATE_ID: &str = "sc_robotics_toolbox_03";
const KINDS: [&str; 5] = ["revolute", "slider", "cylindrical", "rigid", "ball"];

#[derive(Clone)]
struct State {
    open: bool,
    tab: usize,
    a: String,
    b: String,
    name: String,
    joint_kind: usize,
    point: [f64; 3],
    axis: [f64; 3],
    joint: String,
    drive: f64,
    from: f64,
    to: f64,
    steps: u32,
    study: String,
    playback_step: f64,
    effector: String,
    tip: [f64; 3],
    target: [f64; 3],
    joints: Vec<(String, String)>,
    occurrences: Vec<String>,
    studies: Vec<String>,
    output: String,
}
impl Default for State {
    fn default() -> Self {
        Self {
            open: true, tab: 0, a: String::new(), b: String::new(),
            name: "Robot Joint".into(), joint_kind: 0,
            point: [0.0; 3], axis: [0.0, 0.0, 1.0], joint: String::new(),
            drive: 0.0, from: 0.0, to: 90.0, steps: 100,
            study: "Robot Motion".into(), playback_step: 0.0,
            effector: String::new(), tip: [0.0; 3],
            target: [100.0, 0.0, 0.0], joints: Vec::new(),
            occurrences: Vec::new(), studies: Vec::new(), output: String::new(),
        }
    }
}
fn vector(ui: &mut egui::Ui, label: &str, v: &mut [f64; 3]) {
    ui.horizontal(|ui| {
        ui.label(label);
        for i in 0..3 {
            ui.add(DragValue::new(&mut v[i]).speed(0.5).prefix(["X ", "Y ", "Z "][i]));
        }
    });
}
fn trim(s: &str, max: usize) -> String { s.chars().take(max).collect() }
fn note(s: &mut State, result: Result<Value, String>) -> bool {
    match result {
        Ok(v) => { s.output = format!("OK: {}", trim(&v.to_string(), 800)); true }
        Err(e) => { s.output = format!("Fehler: {}", trim(&e, 800)); false }
    }
}
fn selected_occurrences(selection: &Value) -> Vec<String> {
    let mut ids = Vec::new();
    if let Some(arr) = selection.as_array() {
        for v in arr {
            if v.get("type").and_then(Value::as_str) != Some("occurrence") { continue; }
            let value = v.get("path").and_then(Value::as_array).and_then(|p| p.last())
                .or_else(|| v.get("occurrence")).or_else(|| v.get("id"));
            let found = value.and_then(|id| id.as_str().map(str::to_string)
                .or_else(|| id.as_u64().map(|n| n.to_string())));
            if let Some(id) = found { if !ids.contains(&id) { ids.push(id); } }
        }
    }
    ids
}
fn refresh(app: &mut SolveApp, s: &mut State) {
    match app.run("component.list", json!({"compact": true})) {
        Ok(v) => {
            let mut names = Vec::new();
            if let Some(components) = v.get("components").and_then(Value::as_array) {
                for c in components {
                    if let Some(occs) = c.get("occurrences").and_then(Value::as_array) {
                        for o in occs {
                            if let Some(name) = o.get("name").and_then(Value::as_str) {
                                names.push(name.to_string());
                            }
                        }
                    }
                }
            }
            names.sort(); names.dedup(); s.occurrences = names;
        }
        Err(e) => s.output = e,
    }
    match app.run("joint.list", json!({"quiet": true})) {
        Ok(v) => {
            s.joints = v.get("joints").and_then(Value::as_array).into_iter().flatten()
                .filter_map(|j| Some((j.get("name")?.as_str()?.to_string(),
                    j.get("type")?.as_str()?.to_string()))).collect();
        }
        Err(e) => s.output = e,
    }
    match app.run("motion.list", json!({})) {
        Ok(v) => {
            s.studies = v.get("studies").and_then(Value::as_array).into_iter().flatten()
                .filter_map(|j| j.get("name").and_then(Value::as_str).map(str::to_string)).collect();
        }
        Err(e) => s.output = e,
    }
}
fn occurrence_combo(ui: &mut egui::Ui, id: &'static str, label: &str, value: &mut String, options: &[String]) {
    ui.horizontal(|ui| {
        ui.label(label);
        let shown = if value.is_empty() { "Auswählen".to_owned() } else { value.clone() };
        egui::ComboBox::from_id_salt(id).selected_text(shown).show_ui(ui, |ui| {
            for name in options { ui.selectable_value(value, name.clone(), name); }
        });
        ui.text_edit_singleline(value).on_hover_text("Name/ID auch manuell eingeben");
    });
}
fn joint_combo(ui: &mut egui::Ui, s: &mut State) {
    ui.horizontal(|ui| {
        ui.label("Gelenk");
        let chosen = if s.joint.is_empty() { "Auswählen".to_owned() } else { s.joint.clone() };
        egui::ComboBox::from_id_salt("robot_joint_combo").selected_text(chosen).show_ui(ui, |ui| {
            for (name, kind) in &s.joints {
                ui.selectable_value(&mut s.joint, name.clone(), format!("{name} ({kind})"));
            }
        });
    });
}
fn create_joint(app: &mut SolveApp, s: &mut State) {
    if s.a.is_empty() || s.b.is_empty() || s.a == s.b {
        s.output = "Zwei verschiedene Baugruppen-Instanzen auswählen.".into(); return;
    }
    if !s.point.iter().chain(s.axis.iter()).all(|n| n.is_finite())
        || s.axis.iter().map(|v| v * v).sum::<f64>() < 1e-12 {
        s.output = "Punkt/Achse ungültig.".into(); return;
    }
    let side = |id: &str| json!({"occurrence": id, "point": s.point, "z": s.axis});
    let args = json!({"type": KINDS[s.joint_kind], "name": s.name,
        "a": side(&s.a), "b": side(&s.b), "flip": true});
    if note(s, app.run("joint.create", args)) { refresh(app, s); }
}
fn motion_params(s: &State) -> Value {
    json!({"name": s.study, "steps": s.steps,
        "keys": [{"joint": s.joint, "index": 0,
            "points": [[0, s.from], [s.steps, s.to]]}]})
}
fn tabs(ui: &mut egui::Ui, s: &mut State) {
    ui.horizontal(|ui| {
        for (i, name) in ["Gelenke", "Antrieb", "Zielpunkt", "Timeline"].iter().enumerate() {
            ui.selectable_value(&mut s.tab, i, *name);
        }
    });
    ui.separator();
}
fn body(app: &mut SolveApp, ui: &mut egui::Ui, s: &mut State) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("CAD-Baugruppen · Native Rust").strong());
        if ui.button("↻ Neu laden").clicked() { refresh(app, s); }
    });
    tabs(ui, s);
    match s.tab {
        0 => {
            if ui.button("2 ausgewählte Instanzen übernehmen").clicked() {
                match app.run("document.inspect", json!({"measure": false, "sketches": false})) {
                    Ok(v) => {
                        let ids = selected_occurrences(&v["selection"]);
                        if ids.len() == 2 {
                            s.a = ids[0].clone(); s.b = ids[1].clone();
                            s.output = "Auswahl übernommen.".into();
                        } else {
                            s.output = "Zwei Instanzen im Browser auswählen; andernfalls Namen manuell eingeben.".into();
                        }
                    }
                    Err(e) => s.output = e,
                }
            }
            occurrence_combo(ui, "robot_a", "Basis A", &mut s.a, &s.occurrences);
            occurrence_combo(ui, "robot_b", "Teil B", &mut s.b, &s.occurrences);
            ui.horizontal(|ui| { ui.label("Gelenkname"); ui.text_edit_singleline(&mut s.name); });
            egui::ComboBox::from_id_salt("robot_kind")
                .selected_text(KINDS[s.joint_kind])
                .show_ui(ui, |ui| {
                    for (i, kind) in KINDS.iter().enumerate() {
                        ui.selectable_value(&mut s.joint_kind, i, *kind);
                    }
                });
            vector(ui, "Gelenkpunkt (mm)", &mut s.point);
            vector(ui, "Achse", &mut s.axis);
            if ui.button("Gelenk erstellen").clicked() { create_joint(app, s); }
        }
        1 => {
            joint_combo(ui, s);
            ui.horizontal(|ui| {
                ui.label("Position (° oder mm)");
                ui.add(DragValue::new(&mut s.drive).speed(1.0));
            });
            if ui.button("Gelenk bewegen").clicked() {
                let r = app.run("joint.drive", json!({"joint": s.joint, "value": s.drive}));
                note(s, r);
            }
            ui.label("Drehgelenk: Grad. Schiebegelenk: Millimeter.");
        }
        2 => {
            occurrence_combo(ui, "robot_tool", "Werkzeugteil", &mut s.effector, &s.occurrences);
            vector(ui, "Werkzeugpunkt (Welt, mm)", &mut s.tip);
            vector(ui, "Zielpunkt (Welt, mm)", &mut s.target);
            if ui.button("Ziel anfahren (IK)").clicked() {
                let r = app.run("joint.reach",
                    json!({"occurrence": s.effector, "at": s.tip, "point": s.target}));
                note(s, r);
            }
            ui.label("SolidCrafts eigene inverse Kinematik und Gelenkgrenzen.");
        }
        _ => {
            joint_combo(ui, s);
            ui.horizontal(|ui| { ui.label("Studie"); ui.text_edit_singleline(&mut s.study); });
            ui.horizontal(|ui| {
                ui.label("Von"); ui.add(DragValue::new(&mut s.from));
                ui.label("Bis"); ui.add(DragValue::new(&mut s.to));
            });
            ui.horizontal(|ui| {
                ui.label("Schritte"); ui.add(DragValue::new(&mut s.steps).range(1..=100_000));
            });
            if ui.button("Bewegungsstudie speichern").clicked() {
                if s.joint.is_empty() { s.output = "Gelenk auswählen.".into(); }
                else {
                    let args = motion_params(s);
                    if note(s, app.run("motion.study", args)) { refresh(app, s); }
                }
            }
            egui::ComboBox::from_id_salt("robot_studies")
                .selected_text(s.study.clone()).show_ui(ui, |ui| {
                    for name in &s.studies { ui.selectable_value(&mut s.study, name.clone(), name); }
                });
            ui.horizontal(|ui| {
                ui.add(egui::Slider::new(&mut s.playback_step, 0.0..=f64::from(s.steps)).text("Frame"));
                if ui.button("▶ Frame").clicked() {
                    let r = app.run("motion.play", json!({"study": s.study, "step": s.playback_step}));
                    note(s, r);
                }
            });
            if ui.button("Kollisionen im Bewegungsverlauf prüfen").clicked() {
                let r = app.run("motion.interference", json!({"study": s.study, "samples": 30}));
                note(s, r);
            }
        }
    }
    ui.separator();
    ui.label(RichText::new(&s.output).small());
}
/// Called once per frame from SolidCraft's cad_ui.
pub fn show(app: &mut SolveApp, ctx: &egui::Context) {
    let key = egui::Id::new(STATE_ID);
    let mut state = ctx.data_mut(|d| d.get_temp::<State>(key)).unwrap_or_default();
    if ctx.input(|i| i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(egui::Key::R)) {
        state.open = !state.open;
    }
    if !state.open {
        egui::Area::new(egui::Id::new("sc_robotics_launch"))
            .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-14.0, 96.0))
            .show(ctx, |ui| {
                if ui.button("⚙ Machine & Robotics").clicked() { state.open = true; }
            });
    } else {
        let mut open = state.open;
        egui::Window::new("MACHINE & ROBOTICS")
            .id(egui::Id::new("sc_robotics_panel"))
            .open(&mut open)
            .default_pos(egui::pos2(280.0, 135.0))
            .default_width(480.0)
            .resizable(true)
            .show(ctx, |ui| { body(app, ui, &mut state); });
        state.open = open;
    }
    ctx.data_mut(|d| d.insert_temp(key, state));
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_unambiguous_occurrences_are_selected() {
        let v = json!([{"type":"body","name":"A"},
            {"type":"occurrence","path":[2,4]},{"type":"occurrence","path":[2,5]}]);
        assert_eq!(selected_occurrences(&v), vec!["4", "5"]);
    }
    #[test]
    fn motion_study_builds_a_valid_key() {
        let mut s = State::default(); s.joint = "Shoulder".into();
        assert_eq!(motion_params(&s)["keys"][0]["points"][1], json!([100, 90.0]));
    }
}

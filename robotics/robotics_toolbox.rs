//! Native SolidCraft robotics toolbox (egui).
//! All edits flow through SolveApp::run, so native undo, joints and studies stay intact.
use crate::SolveApp;
use egui::{self, DragValue, RichText};
use serde_json::{json, Value};
use solidcraft_engine::Sel;

const STATE_ID: &str = "sc_robotics_toolbox_03";
const KINDS: [&str; 5] = ["revolute", "slider", "cylindrical", "rigid", "ball"];

#[derive(Clone)]
struct State {
    open: bool,
    tab: usize,
    session_uid: Option<u64>,
    a: String,
    b: String,
    name: String,
    joint_kind: usize,
    point: [f64; 3],
    axis: [f64; 3],
    joint: String,
    drive: f64,
    drive_min: f64,
    drive_max: f64,
    live_drive: bool,
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
            open: true, tab: 0, session_uid: None,
            a: String::new(), b: String::new(),
            name: "Robot Joint".into(), joint_kind: 0,
            point: [0.0; 3], axis: [0.0, 0.0, 1.0], joint: String::new(),
            drive: 0.0, drive_min: -180.0, drive_max: 180.0,
            live_drive: false, from: 0.0, to: 90.0, steps: 100,
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
/// Use the real CAD selection; 3D face, edge, body and vertex picks retain
/// their exact occurrence, including repeated components in different placements.
fn picked_occurrences(app: &SolveApp) -> Vec<String> {
    let mut ids = selected_occurrences(
        &serde_json::to_value(&app.session.selection).unwrap_or(Value::Null)
    );
    for item in &app.session.selection {
        let id = match item {
            Sel::Occurrence { path } => path.last().copied(),
            Sel::Body { name } => crate::dialogs_assembly::occurrence_of_body(&app.session, name),
            Sel::Face { body, .. } | Sel::Edge { body, .. } | Sel::Vertex { body, .. } =>
                crate::dialogs_assembly::occurrence_of_body(&app.session, body),
            _ => None,
        };
        if let Some(id) = id.filter(|id| *id != 0) {
            let name = id.to_string();
            if !ids.contains(&name) { ids.push(name); }
        }
    }
    ids
}
fn array3(v: &Value) -> Option<[f64; 3]> {
    let a = v.as_array()?;
    if a.len() != 3 { return None; }
    let xyz = [a[0].as_f64()?, a[1].as_f64()?, a[2].as_f64()?];
    xyz.iter().all(|x| x.is_finite()).then_some(xyz)
}
/// Only use a line as an axis if its length agrees with the end-to-end chord.
/// Curved CAD edges must be picked using SolidCraft's native Joint dialog.
fn line_axis(edge: &Value) -> Option<([f64; 3], [f64; 3])> {
    let a = array3(edge.get("start")?)?;
    let b = array3(edge.get("end")?)?;
    let length = edge.get("length")?.as_f64()?;
    if !length.is_finite() || length < 1e-8 { return None; }
    let delta = [b[0]-a[0], b[1]-a[1], b[2]-a[2]];
    let norm = delta.iter().map(|x| x*x).sum::<f64>().sqrt();
    if norm < length * 0.995 || norm < 1e-8 { return None; }
    let middle = [(a[0]+b[0])/2.0, (a[1]+b[1])/2.0, (a[2]+b[2])/2.0];
    Some((middle, [delta[0]/norm, delta[1]/norm, delta[2]/norm]))
}
fn pick_geometry(app: &mut SolveApp, s: &mut State) {
    let Some(selection) = app.session.selection.last().cloned() else {
        s.output = "Zuerst eine Fläche oder gerade Kante im 3D-Modell auswählen.".into();
        return;
    };
    match selection {
        Sel::Face { body, index, point } => {
            match app.run("model.faces", json!({"body": body})) {
                Ok(result) => {
                    let face = result.get("faces").and_then(Value::as_array)
                        .and_then(|faces| faces.iter().find(|f| f["index"].as_u64() == Some(index as u64)));
                    let Some(face) = face else { s.output = "Fläche nicht gefunden.".into(); return; };
                    if let Some(cyl) = face.get("cylinder").filter(|v| v.is_object()) {
                        if let (Some(p), Some(axis)) = (
                            cyl.get("origin").and_then(array3), cyl.get("axis").and_then(array3)
                        ) {
                            s.point = p;
                            s.axis = axis;
                            s.output = "Zylinderachse und Achsenpunkt übernommen.".into();
                            return;
                        }
                    }
                    if let Some(axis) = face.get("plane_normal").and_then(array3) {
                        s.point = [point.x, point.y, point.z];
                        s.axis = axis;
                        s.output = "Flächenpunkt und Flächennormale übernommen.".into();
                    } else {
                        s.output = "Keine eindeutige Normale. Grafischen SolidCraft-Gelenkdialog verwenden.".into();
                    }
                }
                Err(e) => s.output = e,
            }
        }
        Sel::Edge { body, index, .. } => {
            match app.run("model.edges", json!({"body": body})) {
                Ok(result) => {
                    let edge = result.get("edges").and_then(Value::as_array)
                        .and_then(|edges| edges.iter().find(|e| e["index"].as_u64() == Some(index as u64)));
                    if let Some((point, axis)) = edge.and_then(line_axis) {
                        s.point = point; s.axis = axis;
                        s.output = "Gerade Kante als Gelenkachse übernommen.".into();
                    } else {
                        s.output = "Kante ist gekrümmt oder mehrdeutig. Grafischen Gelenkdialog verwenden.".into();
                    }
                }
                Err(e) => s.output = e,
            }
        }
        Sel::Vertex { point, .. } => {
            s.point = [point.x, point.y, point.z];
            s.output = "Eckpunkt übernommen; Achsrichtung manuell festlegen.".into();
        }
        _ => s.output = "Eine Fläche, gerade Kante oder einen Eckpunkt auswählen.".into(),
    }
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
/// Activate SolidCraft's own on-canvas joint rings and slide arrows.
//// The native viewport handles preserve joint limits, loop constraints,
//// collision stops and a single undo checkpoint across a drag.
fn activate_3d_handles(app: &mut SolveApp, s: &mut State, prefer_joint: bool) {
    if !app.preferences.assembly_move_handles {
        s.output = "3D-Gelenkgriffe sind in SolidCrafts Einstellungen deaktiviert (Assembly Move Handles).".into();
        return;
    }
    if app.session.active_sketch.is_some() {
        s.output = "Bitte zuerst die aktive Skizze beenden.".into();
        return;
    }

    let mut candidates: Vec<u64> = Vec::new();
    if prefer_joint {
        if let Some(joint) = app.session.doc.assembly.joints.iter()
            .find(|j| j.name == s.joint && !j.suppressed)
        {
            for id in [joint.b.occurrence, joint.a.occurrence] {
                if id != 0 && !candidates.contains(&id) { candidates.push(id); }
            }
        }
    }
    // A viewport/body/browser selection wins when explicitly requested.
    for id in picked_occurrences(app).iter().filter_map(|name| name.parse::<u64>().ok()) {
        if !candidates.contains(&id) { candidates.push(id); }
    }
    if !prefer_joint {
        if let Some(joint) = app.session.doc.assembly.joints.iter()
            .find(|j| j.name == s.joint && !j.suppressed)
        {
            for id in [joint.b.occurrence, joint.a.occurrence] {
                if id != 0 && !candidates.contains(&id) { candidates.push(id); }
            }
        }
    }

    let previous = serde_json::to_value(&app.session.selection).unwrap_or(Value::Null);
    for id in candidates {
        // The handles deliberately follow the current selection each frame.
        // Select the occurrence first, otherwise SolidCraft's native sync closes them.
        let Some(occ) = app.session.doc.occurrences.iter().find(|o| o.id == id) else { continue; };
        let mut path = app.session.doc.first_path(occ.parent);
        path.push(id);
        if app.run("select.set", json!({"items": [{"type": "occurrence", "path": path}]})).is_err() {
            continue;
        }
        if crate::joint_handles::begin(app, id) {
            s.output = "3D-Manipulator aktiviert. Drehringe/Pfeile direkt am Modell ziehen; Esc beendet den Modus.".into();
            s.open = false;
            return;
        }
    }
    if let Some(items) = previous.as_array() {
        let _ = app.run("select.set", json!({"items": items}));
    }
    s.output = "Kein frei bewegliches Gelenk an diesem Bauteil. Gelenktyp, Fixierung oder Zwangsbedingungen prüfen.".into();
}

fn edit_selected_joint(app: &mut SolveApp, s: &mut State) {
    let Some(id) = app.session.doc.assembly.joints.iter()
        .find(|j| j.name == s.joint).map(|j| j.id)
    else {
        s.output = "Bitte zuerst ein Gelenk auswählen.".into();
        return;
    };
    match crate::dialogs_assembly::edit_joint(app, id) {
        Some(dialog) => {
            crate::joint_handles::close(app);
            app.tool = None;
            app.dialog = Some(dialog);
            s.open = false;
            s.output = "Grafischen Gelenkeditor geöffnet.".into();
        }
        None => s.output = "SolidCraft konnte das Gelenk nicht öffnen.".into(),
    }
}

fn find_selected_joint(app: &mut SolveApp, s: &mut State) {
    let Some(id) = app.session.doc.assembly.joints.iter()
        .find(|j| j.name == s.joint).map(|j| j.id)
    else {
        s.output = "Bitte zuerst ein Gelenk auswählen.".into();
        return;
    };
    crate::dialogs_assembly::find_joint(app, id);
    s.output = "Ansicht auf die verbundenen Bauteile zentriert.".into();
}

fn motion_params(s: &State) -> Value {
    let mut args = json!({"name": s.study, "steps": s.steps,
        "keys": [{"joint": s.joint, "index": 0,
            "points": [[0, s.from], [s.steps, s.to]]}]});
    // Editing an existing motion study must not create a second one.
    if s.studies.contains(&s.study) { args["study"] = json!(s.study); }
    args
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
                let ids = picked_occurrences(app);
                if ids.len() == 2 {
                    s.a = ids[0].clone(); s.b = ids[1].clone();
                    s.output = "Zwei exakte CAD-Instanzen übernommen.".into();
                } else {
                    s.output = "Zwei unterschiedliche Baugruppen oder Körper im Browser/3D wählen. Alternativ Instanzen manuell eingeben.".into();
                }
            }
            ui.horizontal_wrapped(|ui| {
                if ui.button("Achse/Punkt von CAD-Fläche oder Kante").clicked() {
                    pick_geometry(app, s);
                }
                if ui.button("Gelenk grafisch im 3D erstellen").clicked() {
                    s.open = false;
                    app.start("joint.create");
                }
            });
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
            ui.horizontal_wrapped(|ui| {
                if ui.button("3D-Griffe für gewählte Baugruppe").clicked() {
                    activate_3d_handles(app, s, false);
                }
            });
        }
        1 => {
            joint_combo(ui, s);
            let kind = s.joints.iter().find(|(name, _)| name == &s.joint)
                .map(|(_, kind)| kind.as_str()).unwrap_or("revolute");
            let unit = if kind == "slider" { "mm" } else { "° / mm" };
            ui.horizontal(|ui| {
                ui.label(format!("Position ({unit})"));
                ui.add(DragValue::new(&mut s.drive).speed(1.0));
                if ui.button("Istwert").clicked() {
                    if let Ok(reply) = app.run("joint.list", json!({"quiet": true})) {
                        let found = reply.get("joints").and_then(Value::as_array)
                            .and_then(|arr| arr.iter().find(|j| j["name"].as_str() == Some(s.joint.as_str())));
                        if let Some(value) = found.and_then(|j| j["values"][0]["value"].as_f64()) {
                            s.drive = value;
                            s.output = "Istwert des Gelenks übernommen.".into();
                        }
                    }
                }
            });
            ui.horizontal(|ui| {
                ui.label("Regelbereich");
                ui.add(DragValue::new(&mut s.drive_min).speed(1.0));
                ui.label("bis");
                ui.add(DragValue::new(&mut s.drive_max).speed(1.0));
            });
            ui.checkbox(&mut s.live_drive, "Live-Vorschau beim Bewegen");
            if s.drive_min < s.drive_max {
                let changed = ui.add(egui::Slider::new(&mut s.drive, s.drive_min..=s.drive_max).text(unit)).changed();
                if changed && s.live_drive && !s.joint.is_empty() {
                    let r = app.run("joint.drive", json!({"joint": s.joint, "value": s.drive}));
                    note(s, r);
                }
            } else {
                ui.label("Minimum muss kleiner als Maximum sein.");
            }
            ui.horizontal_wrapped(|ui| {
                if ui.button("Gelenk bewegen").clicked() {
                    let r = app.run("joint.drive", json!({"joint": s.joint, "value": s.drive}));
                    note(s, r);
                }
                if ui.button("Original SolidCraft-Antriebsdialog").clicked() {
                    s.open = false;
                    app.start("joint.drive");
                }
            });
            ui.horizontal_wrapped(|ui| {
                if ui.button("3D-Griffe am Gelenk anzeigen").clicked() {
                    activate_3d_handles(app, s, true);
                }
                if ui.button("Gelenk im Modell suchen").clicked() {
                    find_selected_joint(app, s);
                }
                if ui.button("Gelenkachse grafisch bearbeiten").clicked() {
                    edit_selected_joint(app, s);
                }
            });
            ui.label("Die 3D-Griffe von SolidCraft beachten Gelenkgrenzen und gruppieren einen Mauszug zu einem Undo-Schritt.");
            ui.label("Live-Vorschau per Slider kann dagegen mehrere Undo-Schritte anlegen.");
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
                match app.run("motion.interference", json!({"study": s.study, "samples": 30})) {
                    Ok(report) => {
                        let truncated = report["truncated"] == true;
                        let collisions = report["collisions"].as_array().map_or(0, Vec::len);
                        s.output = if truncated {
                            format!("Unvollständige Prüfung: {collisions} Kollisionspaare erkannt; Rest nicht geprüft.")
                        } else if report["clear"] == true {
                            "Prüfung vollständig: keine Kollision erkannt.".into()
                        } else {
                            format!("Prüfung vollständig: {collisions} Kollisionspaare. Details: {}", trim(&report.to_string(), 650))
                        };
                    }
                    Err(e) => s.output = e,
                }
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
    if state.session_uid != Some(app.session.uid) {
        state.session_uid = Some(app.session.uid);
        refresh(app, &mut state);
    }
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
        state.open = state.open && open;
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
    fn straight_edge_has_expected_axis() {
        let edge = json!({"start":[0,0,0], "end":[0,10,0], "length":10.0});
        assert_eq!(line_axis(&edge), Some(([0.0,5.0,0.0],[0.0,1.0,0.0])));
    }
    #[test]
    fn a_curved_edge_is_not_treated_as_a_linear_axis() {
        let edge = json!({"start":[0,0,0], "end":[1,0,0], "length":3.14159});
        assert!(line_axis(&edge).is_none());
    }
    #[test]
    fn existing_study_is_updated_not_duplicated() {
        let mut state = State::default();
        state.study = "Robot Motion".into();
        state.studies.push("Robot Motion".into());
        assert_eq!(motion_params(&state)["study"], "Robot Motion");
    }
    #[test]
    fn native_3d_grips_open_for_a_robot_joint() {
        use crate::scenario::Harness;
        for kind in ["revolute", "slider"] {
            let mut h = Harness::new();
            h.app.home.open = false;
            for (name, corner, size) in [
                ("Base", [-5, -5, 0], [10, 10, 5]),
                ("Arm", [-3, -3, 5], [63, 6, 5]),
            ] {
                h.app.run("component.activate", json!({"component": "root"})).unwrap();
                h.app.run("component.create", json!({"name": name, "grounded": false})).unwrap();
                h.app.run("solid.box", json!({
                    "length": size[0], "width": size[1],
                    "height": size[2], "corner": corner
                })).unwrap();
            }
            h.app.run("component.activate", json!({"component": "root"})).unwrap();
            h.app.run("occurrence.ground", json!({
                "occurrence": "Base:1", "grounded": true
            })).unwrap();
            let origin = |name: &str| json!({
                "occurrence": name, "point": [0, 0, 5], "z": [0, 0, 1]
            });
            let r = h.app.run("joint.create", json!({
                "type": kind, "name": "Tool Joint",
                "a": origin("Base"), "b": origin("Arm"),
                "flip": true, "angle": 0
            })).unwrap();
            assert!(r.get("joint").is_some());
            let mut state = State::default();
            state.joint = "Tool Joint".into();
            activate_3d_handles(&mut h.app, &mut state, true);
            assert!(!state.open, "3D mode should close the toolbox window");
            assert!(
                h.app.viewport.joint_handles.is_some(),
                "native handles did not start for {kind}: {}", state.output
            );
            let arm = h.app.session.doc.find_component("Arm")
                .and_then(|c| h.app.session.doc.occurrence_of(c))
                .map(|o| o.id).unwrap();
            assert_eq!(crate::joint_handles::selected_occurrence(&h.app), Some(arm));
            h.frames(2);
            assert!(crate::joint_handles::visible(&h.app));
            crate::joint_handles::close(&mut h.app);
        }
    }

    #[test]
    fn motion_study_builds_a_valid_key() {
        let mut s = State::default(); s.joint = "Shoulder".into();
        assert_eq!(motion_params(&s)["keys"][0]["points"][1], json!([100, 90.0]));
    }
}

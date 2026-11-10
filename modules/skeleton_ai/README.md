# SolidCraft Skeleton AI V0.7 – Rust-Editor mit FEM und Optimierung

**Neu:** 3D-Balken-FEM (6 Freiheitsgrade pro Knoten), Lasten/Momente, Lagerungen, FEM-Verformungsanzeige sowie FEM-geprüfte Querschnittsoptimierung bei Materialbudget. Details siehe [FEM_OPTIMIERUNG_DE.md](FEM_OPTIMIERUNG_DE.md).

Start: `cargo run --release --bin skeleton-editor -- --input examples/force_optimization_demo.json`, danach `http://127.0.0.1:8765`.

Die generierte Oberfläche ist ein SDF-Dreiecksnetz, keine B-Rep. Die FEM bewertet ein Balken-Ersatzmodell, nicht die exakte lokale Spannung im fertigen Körper. Die Optimierung nutzt derzeit klassische FEM und Optimality Criteria; das separate Tiny-Ranker-Netz ist noch nicht in der FEM-Optimierung aktiv.

---

# SolidCraft Skeleton AI — native Rust module V0.5

Port of the **V0.4 multi-station organic geometry module** into an independent Rust crate with no Python dependency. Includes CPU inference for the previously trained **1,624-parameter V0.4 TinyProfileRanker**.

> **Integration status:** An embeddable Rust library and executable CLI, not an installed SolidCraft GUI feature. The `ivox80/solidcraft` repository currently has no Rust app/workspace to hook into. Full SolidCraft UI, undo/redo, B-Rep and native STEP integration are still future work. The V0.4 Python beam FEM/training pipeline is **not yet ported**; ML scores are proposals, not validated design outputs.

## Run

```bash
cd modules/skeleton_ai
cargo test
cargo run --release -- --input examples/two_sided_profiles.json --output two_sided.stl --resolution 70
cargo run --release -- --input examples/two_sided_profiles.json --output two_sided.obj --resolution 70
```

Or load the existing Python V0.4 `examples/v04_custom_stations.json` directly.

Exports a triangle mesh via a native SDF and marching-tetrahedra implementation. Mesh edges are welded by lattice-edge IDs; the CLI rejects non-watertight surface edge incidence. This **does not** guarantee correct solid orientation, absence of self-intersections, local stress safety or manufacturability.

## Both section ends are independently editable

The **start station `t=0`** and **end station `t=1`** are completely independent; any number of interior stations is supported. Each station has its own:

- `profile`: `circle`, `ellipse`, `rectangle`, `rounded_rectangle`, `hexagon`, `triangle`, `tube`, `i_beam`
- `width` and `height` in model units, or `area`/`aspect` for V0.4 input
- `corner`, `wall`, `twist_deg`

Modifying the end station never implicitly modifies the start. In code:

```rust
use solidcraft_skeleton_ai::{SkeletonDocument, set_station, Station, Shape};
// doc: SkeletonDocument from serde_json
// set_station(&mut doc, 0, 1, /* Station { t:1.0, ... } */)?;
```

**Note:** `t` increases from `edges[e][0]` to `edges[e][1]`. Reversing the node order reverses which endpoint is `t=0`.

## Embed in a SolidCraft Rust application

Add to the host application's `Cargo.toml`:

```toml
[dependencies]
solidcraft-skeleton-ai = { path = "../modules/skeleton_ai" }
serde_json = "1"
```

```rust
use solidcraft_skeleton_ai::{SkeletonDocument, generate_mesh};
let doc: SkeletonDocument = serde_json::from_str(json_string)?;
let skeleton = doc.to_skeleton(0.12, 0.35)?;
let mesh = generate_mesh(&skeleton, 80, 8_000_000)?;
assert!(mesh.edge_incidence_watertight());
// Use mesh.vertices / mesh.faces in the application's scene graph.
// Keep the original document JSON as editable feature parameters.
```

You must resolve error types in your host (`String` errors cannot be directly `?`-converted into every error type).

## AI scoring

```rust
use solidcraft_skeleton_ai::inference::{force_features, TinyProfileRanker};
let ai = TinyProfileRanker::bundled()?;
let x = force_features([0.0, 1000.0, 0.0, 0.0, 25.0, 0.0], 1.0, 0.012)?;
let suggestions = ai.rank(&x);
// Check recommended profiles with an actual beam / 3D FEM solver.
```

Weights are exported from the V0.4 trained `.npz` file without retraining. AI inference is purely CPU-based Rust, but no new Rust training implementation is included.

## Limitations / quality warnings

- SDF-interpolated transitions are approximate; intermediate section area is not necessarily exactly preserved.
- The mesh volume includes junction smoothing; it may differ from the target nominal beam-material budget.
- `tube` may close cavities at junctions if smoothing or spherical junctions are enabled.
- The SDF uses a discrete voxel grid and marching tetrahedra, not precise NURBS/B-Rep/STEP.
- Mechanical stiffness is **not** recomputed from the actual exported mesh. Recheck local stress, buckling and geometry with independent FEM before manufacture.
- Rust compilation must be run on a machine with `cargo`; the artifact environment used to prepare this crate did not include a Rust compiler.

## Interaktiver Editor (V0.6, lokal)

```bash
cargo run --release --bin skeleton-editor -- --input examples/two_sided_profiles.json
```

Im Browser `http://127.0.0.1:8765` öffnen. Knoten direkt im 3D-Fenster ziehen, End- und Zwischenquerschnitte unabhängig bearbeiten, JSON speichern und OBJ/STL erzeugen. Die Anbindung an die vollständige SolidCraft-Host-GUI ist noch offen. Details: [EDITOR_ANLEITUNG_DE.md](EDITOR_ANLEITUNG_DE.md).
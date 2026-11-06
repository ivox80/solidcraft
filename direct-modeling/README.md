# SolidCraft Direct Modeling – Rust/WASM Lab

**Experimental module:** Independent Rust geometry prototype with a browser viewer.

## v0.3 (new)

- Convex polygonal solids (flat, outward-facing planar polygon faces)
- Rotated rectangular prisms and faceted cylinders (6 to 48 sides)
- Direct offset of one planar face: rebuild neighboring faces from supporting planes
- Multi-target/multi-cutter *difference* of convex polygonal solids, returning convex fragments
- Face picking, selection, numeric face offset, rotation, undo and JSON import/export in the v0.3 viewer
- Rust backend runs as WebAssembly; Three.js is only the viewer

**Limits:** This is NOT a NURBS modeler, not a general B-Rep kernel, not exact cylindrical geometry, not a general robust CAD boolean engine, and not an integration with SolidCraft's internal geometry backend. Concave models appear as multiple convex fragments, and tolerance/fragment limits apply. Keep the original v0.2 box prototype as a separate reference.

## Run tests

```bash
cargo test --manifest-path direct-modeling/Cargo.toml
cargo check --manifest-path direct-modeling/Cargo.toml --features wasm
```

## Build and run browser

Requires Rust, `wasm-pack` and Python 3:

```bash
cd direct-modeling
wasm-pack build --target web --features wasm
python -m http.server 8080
```

Open [v0.3](http://localhost:8080/web/v03.html) or [v0.2](http://localhost:8080/web/index.html).

An Internet connection is needed to fetch Three.js through esm.sh.

Alternatively, download the artifact **solidcraft-direct-modeling-browser** from the
successful [Direct Modeling WebAssembly GitHub Action](https://github.com/ivox80/solidcraft/actions/workflows/direct-modeling-wasm.yml). Extract the `web/` and `pkg/`
directories together, serve their parent over HTTP, and open `web/v03.html`.
Do not open the HTML via `file://` — JS modules/WASM require a web server.

## Main v0.3 interfaces

- `poly_box_wasm(min, max)`: creates axis-aligned convex solid
- `poly_cylinder_wasm(cx,cy,r,z0,z1,sides)`: faceted cylinder
- `poly_rotate_z_wasm(solid,degrees)`: rotate about the solid's vertex centroid
- `poly_offset_face_wasm(solid,index,delta)`: rebuild a planar face offset
- `poly_subtract_many_wasm(targets,cutters)`: disjoint convex fragments per target
- `poly_volume_wasm(solid)`: volume and basic validation

A convex solid serializes as `{faces: [ [[x,y,z],...], ... ]}` with outward-facing CCW polygons. All units follow the caller (demo: mm).

## Production roadmap

1. Record operations in an editable feature history with stable face references.
2. Replace convex polygonal operations with a proven B-Rep geometry kernel for curved surfaces.
3. Add robust general Boolean solids, tolerant edge healing, fillets and chamfers.
4. Integrate via a versioned Rust API into SolidCraft's application.

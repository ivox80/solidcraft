# Direct Modeling Browser Demos

Two separate demos share the Rust WebAssembly package.

- `index.html`: v0.2 box-only viewer (preserved for regression comparison).
- `v03.html`: v0.3 convex polygonal solids, clickable face selection, rotated box,
  faceted cylinder cutter, multi-target Boolean subtraction, planar face offsets,
  JSON project import/export and undo.

- `v04.html`: v0.4 true rational NURBS surface point evaluation, control grid editing,
  four exact quarter-cylinder surface patches, adjustable display tessellation and
  numeric/3D gizmo edits. **Surface-only**, no B-Rep/NURBS Booleans yet.

Build and start from `direct-modeling/`:

```bash
wasm-pack build --target web --features wasm
python -m http.server 8080
```

Visit http://localhost:8080/web/v04.html (or v03.html for the Boolean demo)

An Internet connection is needed for the Three.js ESM CDN. The geometry operations
run in Rust/WASM. Browser storage is not used: save the JSON file explicitly.
The native experimental kernel does not support curved NURBS/B-Rep, fillets or
watertight manifold operations over arbitrary concave CAD meshes.

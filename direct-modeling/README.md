# SolidCraft Direct Modeling (experimental)

A standalone Rust geometry prototype for SolidCraft. It currently supports **axis-aligned boxes only**:
- validated box geometry and volumes
- direct editing of six planar faces
- exact intersection
- subtraction across multiple targets and cutters
- disjoint box output for a single target's subtraction
- optional WebAssembly interface using wasm-bindgen

**Not yet supported:** arbitrary rotations, B-Rep, NURBS, SubD, fillets, general solids, persistent CAD topology, and full-featured Boolean unions.

## Native tests

```bash
cd direct-modeling
cargo test
```

## Build WebAssembly

Requires Rust and wasm-pack:

```bash
cd direct-modeling
wasm-pack build --target web --features wasm
```

The generated `pkg/` exports `subtract_many_wasm(targets, cutters)`. Each box is
`{ min: [x,y,z], max: [x,y,z] }`. The function returns an array of arrays of boxes,
one result array per target. The external JavaScript app must initialize the WASM
module before using it.

This module is deliberately isolated from existing SolidCraft code. The goal is
to validate behavior and API design before selecting a production B-Rep kernel.

# Rust-WASM 3D-Demo

The browser executes real Rust geometry functions exported through wasm-bindgen.
Three.js renders the resulting axis-aligned boxes; it does **not** implement or
silently substitute the Boolean engine.

## Start locally

Install Rust and [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/).
Then, from the repository root:

```bash
cd direct-modeling
wasm-pack build --target web --features wasm
python -m http.server 8080
```

Open http://localhost:8080/web/ (server is rooted in `direct-modeling`).
An Internet connection is required to retrieve Three.js from esm.sh.

The button **Abziehen (Rust)** subtracts the chosen cutter from all selected
target objects. **Fläche verschieben (Rust)** edits one of the six box faces.
The six-box slab representation is exact for axis-aligned boxes but is not a
general-purpose B-Rep solid.

If `pkg/` does not exist, the browser module fails to load. Run the build first.

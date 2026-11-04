# SolidCraft TextilLab v0.1

Standalone Rust prototype for textile construction, not yet integrated into SolidCraft.

## Features
- Rectangular patterns and strictly convex polygons
- Seam allowance offsets in mm
- Material area and estimated mass
- Flat SVG and OBJ exports
- Unit tests and cross-platform GitHub Actions CI

## Run

```bash
cd textillab
cargo test
cargo run --release --bin textillab-demo -- exports
```

The material constants are illustrative, not certified. There is no cloth physics, sewing simulation, concave polygon support, or interactive CAD interface.

CI runs from the repository root through `.github/workflows/rust-ci.yml`. The main branch is unchanged until the pull request is merged.

#!/usr/bin/env python3
"""Patch a clean Tangency Labs SolidCraft source checkout with native TextilLab.

Usage:
    python native-integration/apply_to_upstream.py /path/to/upstream/solidcraft

Non-destructive anchors: errors out if upstream layout changes. Does not modify
the user's modules repository or install anything into a published SolidCraft
binary. Original license headers are preserved.
"""
from __future__ import annotations

import argparse
from pathlib import Path
import shutil
import sys

MODULE_REPO = Path(__file__).resolve().parents[1]
PANEL_SOURCE = Path(__file__).resolve().parent / "textillab_panel.rs"
RUST_FILES = (
    "Cargo.toml",
    "src/lib.rs",
    "src/unfold.rs",
    "src/obj_import.rs",
    "src/studio_patterns.rs",
    "src/main.rs",
)


def replace_once(path: Path, marker: str, insertion: str) -> None:
    text = path.read_text(encoding="utf-8")
    if insertion in text:
        return
    matches = text.count(marker)
    if matches != 1:
        raise ValueError(
            f"{path}: expected exactly one upstream insertion anchor "
            f"{marker!r}, found {matches}. Upstream may have changed."
        )
    path.write_text(text.replace(marker, insertion + marker, 1), encoding="utf-8")


def integrate(upstream: Path) -> None:
    root_cargo = upstream / "Cargo.toml"
    ui_source = upstream / "crates/ui-egui/src/lib.rs"
    ui_manifest = upstream / "crates/ui-egui/Cargo.toml"
    native_module = upstream / "crates/ui-egui/src/textillab_panel.rs"
    if not all(p.is_file() for p in (root_cargo, ui_source, ui_manifest, PANEL_SOURCE)):
        raise ValueError(
            "No compatible SolidCraft checkout found. Expected crates/ui-egui "
            "and root Cargo.toml."
        )
    root = root_cargo.read_text(encoding="utf-8")
    if 'members = ["crates/*", "apps/*", "xtask"]' not in root:
        raise ValueError("Unexpected upstream workspace layout")

    target_crate = upstream / "crates/textillab"
    source_crate = MODULE_REPO / "textillab"
    for relative_path in RUST_FILES:
        src = source_crate / relative_path
        if not src.is_file():
            raise ValueError(f"Missing local TextilLab core: {src}")
        target = target_crate / relative_path
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(src, target)

    shutil.copyfile(PANEL_SOURCE, native_module)
    replace_once(
        ui_manifest,
        "[dependencies]\n",
        '[dependencies]\ntextillab = { package = "solidcraft-textillab-prototype", path = "../textillab" }\n'
    ) if 'textillab = { package = "solidcraft-textillab-prototype"' not in ui_manifest.read_text(encoding="utf-8") else None

    replace_once(ui_source, "pub mod access;\n", "mod textillab_panel;\n")
    replace_once(
        ui_source,
        "pub struct SolveApp {\n",
        "pub struct SolveApp {\n    textillab_panel: textillab_panel::PanelState,\n",
    )
    replace_once(
        ui_source,
        "        let mut app = SolveApp {\n",
        "        let mut app = SolveApp {\n            textillab_panel: Default::default(),\n",
    )
    replace_once(
        ui_source,
        "        self.cad_ui(ui, &mut report_input);\n",
        "        self.cad_ui(ui, &mut report_input);\n        self.textillab_panel.show(ui.ctx());\n",
    )
    print(f"Native TextilLab egui integration applied to: {upstream}")
    print("Compile using: cargo check -p solidcraft-ui-egui --lib")
    print("Start desktop using: cargo run --release -p solidcraft")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("upstream", type=Path)
    args = parser.parse_args()
    try:
        integrate(args.upstream.expanduser().resolve())
    except (ValueError, OSError) as exc:
        print(f"Integration failed: {exc}", file=sys.stderr)
        sys.exit(1)

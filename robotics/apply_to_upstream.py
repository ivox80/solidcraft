#!/usr/bin/env python3
"""Patch a SolidCraft source checkout to use the integrated Rust robotics panel."""
from pathlib import Path
import argparse
import shutil

parser = argparse.ArgumentParser()
parser.add_argument("solidcraft", help="path to upstream SolidCraft source root")
args = parser.parse_args()
root = Path(args.solidcraft).resolve()
source = Path(__file__).resolve().with_name("robotics_toolbox.rs")
lib = root / "crates" / "ui-egui" / "src" / "lib.rs"
if not lib.exists() or not (root / "Cargo.toml").exists():
    parser.error("Not a SolidCraft root: " + str(root))
text = lib.read_text(encoding="utf-8")
module = "pub mod robotics_toolbox;"
call = "robotics_toolbox::show(self, ui.ctx());"
if module not in text:
    anchor = "pub mod render_workspace;"
    if anchor not in text:
        anchor = "pub mod messages;"
    if text.count(anchor) != 1:
        parser.error("Unexpected SolidCraft module structure")
    text = text.replace(anchor, anchor + "\n" + module, 1)
if call not in text:
    anchor = "        toolbar_custom::window(self, ui.ctx());"
    if text.count(anchor) != 1:
        parser.error("Unexpected SolidCraft cad_ui; review patch manually")
    text = text.replace(anchor, anchor + "\n        " + call, 1)
lib.write_text(text, encoding="utf-8")
target = lib.with_name("robotics_toolbox.rs")
shutil.copyfile(source, target)
print("Native panel installed:", target)
print("Start SolidCraft; Ctrl+Shift+R toggles MACHINE & ROBOTICS.")

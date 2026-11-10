# SolidCraft Skeleton AI V0.7 – Mechanik & Optimierung

## Start (Windows)

1. [Rust](https://rustup.rs/) installieren, wenn noch nicht vorhanden.
2. `modules/skeleton_ai` als Terminalordner öffnen.
3. `cargo test` und anschließend:

```powershell
cargo run --release --bin skeleton-editor -- --input examples/force_optimization_demo.json
```

4. Im Browser `http://127.0.0.1:8765` aufrufen.

## Bedienung

- Ein **grünes Rechteck** markiert eine vollständig eingespannte Lagerung; im rechten Inspector kann sie einem ausgewählten Knoten hinzugefügt oder entfernt werden.
- Der **orange Kraftpfeil** ist per Maus veränderbar. Mit `XY / XZ / YZ` wird die Drag-Ebene gewählt. Über den rechten Inspector können zusätzlich alle drei Kraftkomponenten in Newton und alle drei Momente in Newtonmeter eingegeben werden.
- **FEM berechnen** liefert die linear-elastische 3D-Balkenanalyse. Die magenta gestrichelten Linien zeigen die **stark überhöhte** Verformung, keine reale 1:1-Verformung.
- **Struktur optimieren** verändert Querschnittsgrößen aller aktiven Kanten, prüft die Vorschläge mit dem Solver und speichert die Veränderung in der Undo-Historie. Anfang, Ende und Zwischenstationen werden proportional vergrößert/verkleinert; unterschiedliche Profilarten bleiben erhalten.
- **Materialvolumen Ziel** bestimmt das Budget relativ zur Ausgangskonstruktion. Standard: 100 %, d. h. nur Material zwischen den Verbindungen verteilen. Die Optimierung erzeugt bislang **keine neuen Kanten**.
- **JSON speichern** speichert Geometrie, Lasten, Lagerungen und Materialparameter; STL/OBJ enthalten ausschließlich das Oberflächennetz.

## Technik

- Direkte 3D-Balken-Euler–Bernoulli-FEM, sechs Freiheitsgrade pro Knoten, lineare Elastizität, kleine Verformungen, starre Verzweigungen.
- Punktlasten und Momente, frei wählbare vollständig eingespannte Knoten; das JSON unterstützt zusätzlich partielle Freiheitsgrad-Fixierung über `fixed[6]`.
- Querschnitts-Taper approximiert über harmonisch gemittelte Flächen-, Biege- und Torsionssteifigkeiten.
- Compliance-Optimierung mittels OC-Schritten mit FEM-Rückprüfung; keine neuronale Netz-Inferenz in dieser Optimierung. Das vorhandene Tiny-Profilnetz bleibt separat verfügbar.
- Das SDF-Mesh ist **keine** geprüfte, exakte B-Rep-Geometrie; keine FEM auf diesem Mesh.

## Grenzen

Nicht enthalten sind lokale Kerb-/Kontaktspannungen, plastische Verformung, Knicken, Schubverformung (Timoshenko), Schweiß- und Schraubverbindungen, dynamische Lasten oder Sicherheitszertifizierung. Für einige Querschnittsformen werden Näherungen für Trägheits- und Torsionswerte verwendet. Ergebnisse **nicht als Festigkeitsnachweis** verwenden.
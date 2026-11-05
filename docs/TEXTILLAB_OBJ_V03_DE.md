# TextilLab v0.3 – OBJ einlesen, Schnittkanten auswählen, 2D abwickeln

TextilLab kann nun Dreiecks- und Vierecksnetze als Wavefront-OBJ einlesen,
deren Mesh-Kanten auflisten und entlang gewählter Kanten aufteilen. Die
Abwicklung erzeugt eine **SVG-Vorschau**. Ein nativer SolidCraft-Dialog oder
eine 3D-Mausauswahl sind noch nicht vorhanden; `--interactive` öffnet
eine Auswahl von Kanten **im Terminal**.

## So testest du es unter Windows

1. Rust von https://rustup.rs installieren.
2. Den aktuellen `main`-Branch von https://github.com/ivox80/solidcraft herunterladen
   und entpacken.
3. PowerShell in `textillab` öffnen.

```powershell
cargo test
cargo run --release --bin textillab-obj -- examples/bent-panel.obj --edges --out exports/bent-panel.svg
```

Das erzeugt eine 2D-SVG-Vorschau aus zwei verbundenen Dreiecken.

### Schnittlinie manuell setzen

```powershell
cargo run --release --bin textillab-obj -- examples/bent-panel.obj --seam 1:3 --out exports/split-panel.svg
```

Damit trennen wir beide Dreiecke entlang der gemeinsamen Kante `1:3`.
Die Nummern entsprechen den **1-basierten OBJ-Vertex-Indizes**.

### Interaktiv im Terminal

```powershell
cargo run --release --bin textillab-obj -- examples/bent-panel.obj --interactive --out exports/custom.svg
```

Das Programm listet alle Mesh-Kanten. Du kannst `1:3` und weitere
Kanten eingeben und mit einer Leerzeile den Export starten.

## Einschränkungen

- OBJ-Importer unterstützt Dreiecke und Vierecke, keine allgemeinen N-Gons;
  Vierecke werden entlang ihrer ersten Diagonalen trianguliert.
- OBJ enthält keine verbindliche Längeneinheit. Für millimetergenaue
  Ergebnisse muss das Modell bereits in Millimetern vorliegen.
- Aktuell ausschließlich Kantenwahl per Nummer im Terminal; noch keine
  interaktive Auswahl im SolidCraft-3D-Fenster.
- Die 3D-Abwicklung geht von annähernd isometrisch abwickelbaren Flächen aus.
  Doppelt gekrümmte Meshes können ohne zusätzliche Schnitte scheitern.
- Kein automatischer Zuschnittoptimierer, keine Stoffphysik oder
  automatische Ermittlung sinnvoller Nähte.
- SVG ist derzeit eine **technische Vorschau**, kein geprüftes
  Produktionsschnittmuster mit Randkontur, Passzeichen und Nahtzugaben.

Nächster Schritt: 3D-Kantenauswahl in der CAD-Oberfläche, automatic seam
suggestions, Boundary-Loops und Materialberechnung.

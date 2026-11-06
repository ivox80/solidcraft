# TextilLab Studio v0.4 – grafische Vorschau (Windows)

**Stand:** Separates TextilLab-Studio mit grafischer 3D-Kantenwahl,
Rust-basierter Abwicklung, Nahtzugabe und SVG-Zuschnittvorschau.
Die App **ist noch kein integriertes SolidCraft-Werkzeug**. Sie verwendet
denselben Rust-Geometriekern aus unserem Modul-Repository.

## Starten

1. Rust installieren: https://rustup.rs
2. https://github.com/ivox80/solidcraft öffnen, **Code → Download ZIP**,
   ZIP entpacken.
3. In PowerShell in den entpackten Ordner wechseln:

\`\`\`powershell
cd textillab
cargo test
cargo run --release --bin textillab-studio
\`\`\`

4. Während das Programm läuft, im Browser öffnen:
   **http://127.0.0.1:27840**

Der lokale Server bindet ausschließlich an 127.0.0.1. Keine Anmeldung,
kein Cloud-Upload des geladenen Modells. Mit **Ctrl+C** beendet man ihn.
Falls Port 27840 belegt ist, wird ein Fehler ausgegeben.

## Funktionen

- Ein Beispiel-OBJ ist beim Start automatisch geladen
- Eigene **OBJ-Datei** auswählen, nur Dreiecke und Vierecke
- 3D-Ansicht mit gedrückter Maustaste drehen; Mausrad zoomt
- Innere **Netzkanten anklicken**, um Schnittkanten ein/auszuschalten
- Einzelne Dreiecke durch Klick auswählen (visuelle Auswahl)
- Nahtzugabe in mm und Materialflächengewicht in g/m² ändern
- Rechts: Rust-berechnete 2D-Ansicht mit Nahtlinie (türkis gestrichelt),
  Zuschnittlinie (orange), Stückzahl, Stofffläche und geschätztem Gewicht
- **SVG exportieren** als Vorschau

## Einschränkungen

- Noch kein natives SolidCraft-Feature, keine CAD-Timeline und keine
  UI-Einbettung in das Hauptprogramm
- 3D-Viewport ist eine lokale Canvas-Darstellung, nicht SolidCrafts Renderer
- OBJ-Einheiten müssen angegeben werden (OBJ speichert keine Einheiten)
- Die Auswahl betrifft die **Netzkanten**, keine frei gezeichneten Nähkurven
- Nahtzugaben sind nur für **konvexe, einfache Schnittteile** implementiert
- Abwicklung erfordert abwickelbare Flächen; einige doppelt gekrümmte Netze
  benötigen weitere Schnitte oder künftig materialabhängige Dehnung
- Keine Prüfung auf übereinanderliegende Schnittteile,
  fehlerhafte Selbstüberschneidungen, komplexe Löcher oder Materialfadenlauf
- SVG ist eine **technische Vorschau**, noch kein freigegebenes
  Fertigungsschnittmuster

## Nächster Integrationsschritt

Für eine **native** SolidCraft-Version braucht es einen Fork bzw. eine passende
Erweiterungsschnittstelle des eigentlichen CAD-Quellcodes. Dort wären die
TextilLab-Feature-Daten, die 3D-Kantenwahl, Parameter-Editoren sowie die
Recompute-Funktion in das Dokument- und Command-System einzubauen.

Diese Studio-Version liefert dafür einen bereits testbaren
Interaktionsablauf, hält die Abwicklungslogik aber unabhängig in Rust.

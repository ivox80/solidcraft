# TextilLab v0.2 – Dreiecksflächen abwickeln

TextilLab v0.2 enthält einen ersten Rust-Algorithmus, um **abwickelbare Dreiecksnetze**
von 3D nach 2D zu übertragen. Gleiche 3D-Kantenlängen bleiben in der Ebene erhalten.

## Windows-Schnelltest

Rust installieren: https://rustup.rs

Repository als ZIP herunterladen: https://github.com/ivox80/solidcraft

Im Repository-Ordner PowerShell öffnen und ausführen:

```powershell
cd textillab
cargo test
cargo run --release --bin unfold-demo
```

Ergebnis: `textillab/unfold-example.svg`. Die Datei lässt sich im Browser oder in
einem Vektorprogramm öffnen.

Ein **zweiter Beispielaufruf** setzt eine Trennnaht entlang der Diagonale zwischen
Mesh-Vertex 0 und Vertex 2:

```powershell
cargo run --release --bin unfold-demo -- --split cut-example.svg
```

Die zwei Dreiecke werden dann als getrennte Schnittmuster-Patches exportiert.

## Algorithmus

- Eingang: Dreiecksnnetz aus `vertices: Vec<Vec3>`, `faces: Vec<[usize; 3]>`
  und `seams: BTreeSet<(usize, usize)>`
- Geometrieprüfung gegen ungültige Indizes, entartete Dreiecke und
  nichtmannigfaltige Kanten
- Die erste Dreiecksfläche bildet den Ursprung eines Patches
- Nachbardreiecke werden anhand ihrer gemeinsamen Kante in die Ebene gelegt
- Markierte Nahtkanten trennen die Patches
- Sind geschlossene Flächen ohne Dehnung nicht abwickelbar, bricht der
  Algorithmus bei geometrischen Widersprüchen mit einer Fehlermeldung ab

## Einschränkungen

- Noch kein OBJ-/STEP-Import, kein automatisches Erkennen von Nähten
- Keine vollständige Überlappungsprüfung zwischen verschiedenen Dreiecken
- Kein elastisches Abwickeln von doppelt gekrümmten Oberflächen
- Noch keine Zugaben/Passmarken an den Patch-Grenzen
- SVG-Ausgabe benutzt eine feste Demo-Zeichenfläche und ist derzeit
  eine Vorschau, kein direkt freigegebenes Fertigungsschnittmuster
- Die Funktion läuft als **separater Rust-Kern**, nicht als integrierter
  SolidCraft-Dialog oder parametrisches CAD-Feature

## Weitere Ausbaustufen

1. Dreiecksnetz-Import (OBJ), Auswahl manueller Schnittkanten und
   Randermittlung
2. Automatische Teilung in entwickelbare Patches und Dehnungsoptimierung
3. Materialmodelle und robuste 2D-Schnittmusterausgabe
4. Native Feature- und UI-Einbindung in einem Fork von SolidCraft

Code: `textillab/src/unfold.rs`, Beispiel: `textillab/src/bin/unfold-demo.rs`.

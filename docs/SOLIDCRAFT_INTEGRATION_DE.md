# TextilLab in SolidCraft verwenden (Windows)

## Was bereits funktioniert

TextilLab ist ein separates Rust-Programm. Es berechnet die flache Stoffkontur eines
rechteckigen Schnittmusters, die Nahtzugabe und eine einfache triangulierte OBJ-Fläche.
SolidCraft kann das OBJ als Mesh-Körper importieren.

**Noch nicht enthalten:** TextilLab-Schaltfläche im CAD, bearbeitbare textile Features
in der SolidCraft-Timeline, Nahtsimulation, Stoffphysik oder Kollisionen.

## Voraussetzungen

1. SolidCraft-Desktop von https://solidcraft.io/ installieren.
2. Rust für Windows installieren: https://rustup.rs (Standardoptionen).
3. Das Repository https://github.com/ivox80/solidcraft als ZIP herunterladen und entpacken
   oder mit Git klonen.

## TextilLab kompilieren und Schnittmuster erzeugen

Im entpackten Ordner `solidcraft-main` PowerShell öffnen:

```powershell
cd textillab
cargo test
cargo run --release --bin textillab-demo -- exports
```

Oder in PowerShell von der Repository-Wurzel:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\run-textillab.ps1
```

Das Programm erzeugt:
- `textillab/exports/pattern.svg`: 2D-Schneidkontur und Nahtlinie
- `textillab/exports/pattern.obj`: flache Stoffoberfläche als Dreiecks-Mesh

Die Demonstration nutzt 400 x 250 mm und 12 mm Nahtzugabe. Materialwerte sind Beispiele.
Das OBJ zeigt derzeit die **Nähkontur ohne Nahtzugabe**, das SVG beide Konturen.

## In SolidCraft öffnen

1. SolidCraft-Desktop starten, ein neues Dokument öffnen.
2. Den Importbefehl **Insert Mesh** verwenden (Befehlskennung `file.insert_mesh`).
3. `textillab/exports/pattern.obj` auswählen.
4. SolidCraft legt den Import als Mesh-Körper in einer neuen Komponente an.
5. Speichern, bei Bedarf die Position über die normalen Komponentenwerkzeuge ändern.

Importierte Meshes sind keine parametrischen B-rep-Solids und können nicht unmittelbar
mit beliebigen Solid-Funktionen bearbeitet werden. Für eine native TextilLab-Timeline-
Integration muss das Modul in den upstream-SolidCraft-Quellbaum eingebaut werden.

## Langfristige native Integration

Der eigentliche CAD-Quellcode liegt unter https://github.com/tangency-labs/solidcraft ,
nicht in diesem Modul-Repository. Laut der Entwicklerdokumentation besteht SolidCraft
aus separaten Rust-Crates, u. a. `crates/engine` (Befehlsregister),
`crates/doc` (Feature-Historie), `crates/ui-egui` (Oberfläche)
und `crates/io` (Import/Export).

Eine native Integration müsste:
1. Den TextilLab-Kern als Rust-Crate in einen SolidCraft-Fork übernehmen.
2. Validierte textile Feature-Daten im Dokumentmodell speichern und neu auswerten.
3. Eigene Befehle wie `textile.pattern.create` und `textile.pattern.export` registrieren.
4. Einen Dialog bzw. eine Toolbar in der UI einbauen.
5. Geometrie und Export mit Tests und CI absichern.

Für den Upstream gelten weitere Architektur- und Lizenzregeln; insbesondere sind
Kerneländerungen derzeit laut dessen Contributing Guide eingeschränkt. Ein Modul
in diesem Repository zu kompilieren installiert **kein Plugin** in SolidCraft.

Dokumentation:
- https://docs.solidcraft.io/guide/files.html
- https://docs.solidcraft.io/reference/commands.html
- https://docs.solidcraft.io/dev/contributing.html

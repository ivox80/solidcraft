# SolidCraft Skeleton AI – Interaktiver 3D-Editor (V0.6)

Dieser Editor ist eine **eigenständige lokale Rust-Anwendung**. Im Repository `ivox80/solidcraft` fehlt derzeit noch die eigentliche SolidCraft-Host-GUI. Deshalb ist die Oberfläche als lokal gehosteter Browser-Editor an die native Rust-Geometrie-Crate angebunden, aber noch **nicht** als Plugin in die echte SolidCraft-3D-Ansicht eingebettet.

## Start (Windows, Linux, macOS)

Rust installieren: https://rustup.rs/ . Danach ein Terminal in `modules/skeleton_ai` öffnen:

```sh
cargo test
cargo run --release --bin skeleton-editor -- --input examples/two_sided_profiles.json
```

Danach `http://127.0.0.1:8765` im Browser öffnen. Der Editor bindet ausschließlich an `127.0.0.1`. Anderer Port: `--port 8766`.

Schnelltest ohne Browser:

```sh
cargo run --release --bin skeleton-editor -- --input examples/two_sided_profiles.json --check
```

## Bedienung

- **Knoten:** Linksklick wählen, ziehen um in der links eingestellten Ebene XY / XZ / YZ zu bewegen. Alternativ X/Y/Z rechts numerisch eingeben.
- **Kanten:** Auf eine Verbindung klicken. Rechts erscheinen alle Querschnittsstationen. Anfang und Ende unabhängig ändern; Station hinzufügen oder entfernen; Profile, Breite, Höhe, Verdrehung, Wand- und Eckenparameter bearbeiten.
- **Neue Kanten:** Startknoten wählen → `Verbinden` → Zielknoten anklicken. Knoten und Kanten entfernen über `Auswahl löschen`.
- **Ansicht:** Ziehen auf leerer Fläche dreht die Ansicht, Rad zoomen, `Ansicht einpassen` setzt den Bildausschnitt zurück.
- **Geometrie:** Jede Änderung markiert die Rust-Netzgeometrie als veraltet und veranlasst mit kurzer Verzögerung eine Neuberechnung. Große oder ungültige Netze werden mit Fehlermeldung angezeigt. Für höhere Genauigkeit Auflösung erhöhen.
- **Rückgängig:** `Strg+Z`, Wiederholen `Strg+Y`.
- **Dateien:** Bearbeitbares parametrisches JSON exportieren/importieren; `In Projektordner` schreibt `<eingabename>.edited.json` neben die Eingabedatei, ohne das Original zu überschreiben. STL und OBJ stammen aus dem Rust-SDF-Mesher. Die Geometrie ist **kein B-Rep/STEP**.

## Technische Grenzen

- Kein nativer SolidCraft-Feature-Timeline-Eintrag und kein echtes CAD-B-Rep.
- Noch keine Rust-3D-FEM, keine lokale Spannungsbewertung beim Ziehen, keine dynamische GNN-Nachoptimierung.
- 3D-Körper werden nur mit SDF und Dreiecksnetzen aufgebaut. Die Viewer-Vorschau ist Canvas-2D-Projektion, ohne externe JS-Bibliotheken; bei großen Modellen wird für die Ansicht nur ein Teil der Dreiecke gezeichnet.
- Der Editor akzeptiert JSON-Dokumente nach dem V0.4-Stationenformat, einschließlich `active` als boolesche Werte oder 0/1. Mesh-Exporte werden auf eine geschlossene Kanteninzidenz geprüft; diese allein garantiert noch keine vollständige CAD-Manifold-Qualität.

## Schnittstelle zur späteren SolidCraft-Host-GUI

Die Rust-Crate stellt weiterhin `SkeletonDocument`, `EdgeStations`, `Station`, `generate_mesh` sowie `Mesh` bereit. Die lokale Oberfläche verwendet `/api/state` (GET/POST), `/api/mesh` (GET) und `/api/download` (GET). Für die Integration in einen richtigen CAD-Host sollten Änderungen direkt über den Dokument-/Undo-Stack und das Scene-Graph-/Feature-System des Hosts geführt werden, anstelle der HTTP-Schicht.
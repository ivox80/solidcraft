# TextilLab nativ in SolidCraft – Quellcodeintegration v0.5

Diese Entwicklungsvariante öffnet das TextilLab-Fenster **direkt in einer selbst kompilierten SolidCraft-Desktop-App (egui)**. Die Abwicklung läuft im Rust-Kern, ohne externen Browserserver.

## Funktionsumfang und Grenzen

- Kantenwahl im 3D-Netz **innerhalb** des nativen TextilLab-Fensters, noch nicht im Haupt-Viewport.
- Noch keine native parametrische Feature-Historie innerhalb einer SolidCraft-.sc-Datei.
- SVG-Zuschnitt und eine .textillab.json-Begleitdatei mit Parametern werden exportiert.
- OBJ muss Millimeterkoordinaten enthalten; derzeit keine automatische Einheitenumrechnung.
- Nur abwickelbare Dreiecksnetze; Nahtzugaben benötigen konvexe Konturen.
- Integration in einen **eigenen SolidCraft-Quellcode-Build**, nicht durch Kopieren eines Plugins in die installierte Release-Version.

## Quellcode

Modul-Repository: https://github.com/ivox80/solidcraft

Original-CAD-Quellcode: https://github.com/tangency-labs/solidcraft

Das Integrationsskript befindet sich in native-integration/apply_to_upstream.py.

Gepinnter Upstream-Commit: dc679cae34fb7db4e6b230b45256c1b2554cabca

## Windows – eigenen nativen SolidCraft-Build erzeugen

Voraussetzungen: Git, aktueller Rust-Compiler (rustup.rs) und Visual Studio Build Tools mit Desktopentwicklung für C++/Windows-SDK. Weitere Angaben: https://docs.solidcraft.io/dev/building.html

PowerShell öffnen:

~~~powershell
mkdir C:\SolidCraft-Entwicklung
cd C:\SolidCraft-Entwicklung

git clone https://github.com/ivox80/solidcraft.git modules
git clone https://github.com/tangency-labs/solidcraft.git upstream

git -C upstream checkout dc679cae34fb7db4e6b230b45256c1b2554cabca
py -3 .\modules\native-integration\apply_to_upstream.py .\upstream

cd upstream
cargo check -p solidcraft-ui-egui --lib
cargo run --release -p solidcraft
~~~

Wenn der Build erfolgreich ist, startet die selbst kompilierte CAD-Anwendung mit einem zusätzlichen **TextilLab**-Button oben rechts.

## Bedienung

1. TextilLab im CAD-Fenster öffnen.
2. Pfad zu einem OBJ-Modell eingeben und **Laden** wählen.
3. Links auf **innere Dreieckskanten** klicken, um Trennnähte zu setzen oder zu entfernen.
4. Nahtzugabe und Flächengewicht ändern.
5. Rechts die abgewickelten Konturen prüfen (Türkis: Naht, Orange: Zuschnitt).
6. Ausgabepfad angeben und **SVG + Projekt speichern** klicken.

Das Projekt wird derzeit nur **als Begleitdatei** geschrieben, noch nicht aus einer .sc-Datei wiederhergestellt.

## Upstream-Kompatibilität

Das Integrationsskript bricht bei geänderter Quellstruktur mit einer klaren Fehlermeldung ab. Verwende deshalb für reproduzierbare Builds zuerst den oben gepinnten Upstream-Commit.

## Weitere Schritte für echte parametrische Features

- Textil-Feature-Definition und Persistenz in crates/doc
- Neue textile CommandSpecs mit Eingabe-/Ausgabeschemata in crates/engine
- Geometrieauswahl direkt im SolidCraft-Hauptviewport
- Automatisches Neuberechnen bei Änderungen am Quellmodell
- Undo/Redo-, Timeline-, Serialisierungs- und Geometrie-Regressionstests

**Diese Version ist eine native egui-Vorschau, noch kein vollständiges parametrisches Textilmodul.**

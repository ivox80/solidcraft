# TextilLab v1.2 – Grafisches Schnittatelier

**Status:** Frontend-Arbeitsoberfläche für den bestehenden TextilLab-Generator.
Die geometrischen Grundschnittformeln bleiben unverändert; die bereits
vorliegenden experimentellen Schnitte sind weiterhin **nicht fachlich als
Müller & Sohn verifiziert**.

## Warum dieses Update?

Vorher waren die Eingabefelder umfangreich, das 2D-Fenster klein und die
Schnittteile nur als Textliste sichtbar. Die neue Oberfläche ist auf
den praktischen Arbeitsablauf „Vorlage → Maße → Konstruktion → Prüfen“ ausgerichtet.

## Sichtbare Funktionen

1. **Grafische Grundschnitt-Galerie:** Oberteil, Jacke, Hose, Rock,
   Handschuh, Mütze, Hut, OBJ-Abwicklung als unmittelbar auswählbare
   Karten mit einfachen Vektorillustrationen.
2. **Reale Miniaturvorschauen der Schnittteile:** In der linken
   Schnittteilübersicht werden **tatsächlich berechnete Konturen** als
   Mini-SVG gezeichnet, nicht statische Produktbilder.
3. **Kontext-Maßfelder:** Nur der relevante Bereich ist sichtbar.
   Für Hosen beispielsweise Taille, Hüfte und Schrittlänge
   plus die erweiterten Hosen-Konstruktionsmaße; alle anderen Bereiche
   bleiben im DOM erhalten und funktionieren beim Vorlagenwechsel.
4. **Große 2D-Konstruktion:** Grafische Arbeitsfläche nimmt in
   der Bibliotheksansicht deutlich mehr Platz ein; der bisherige
   3D-OBJ-Modus bleibt ein eigener Workflow.
5. **Direkte Auswahl:** Die Zeichnungen der Vorder-/Hinterhose
   und des Oberteils besitzen bearbeitbare Auswahlkennzeichnungen
   \`data-piece-id\`. Im Bild oder in der Stückliste angeklickte
   Elemente erhalten eine hervorgehobene Kontur.
6. **Bemaßung anklicken:** Die bestehende Oberteil-Bemaßung und
   zusätzlich die Hosenmaße Knieweite und Seitenlänge springen direkt
   zum passenden Eingabefeld.
7. **Zoom, Einpassen und Fokus:** Bedienelemente direkt über der
   2D-Zeichnung. Der Zoom verändert nur die Vorschauansicht, nicht die
   realen Maße oder die exportierte SVG-Geometrie.
8. **Vier Phasen:** Vorlage, Maße, Konstruktion und Prüfen navigieren
   zu den jeweils relevanten Bereichen.
9. **Responsive Layout:** Die Kartenleiste scrollt auf kleinen
   Bildschirmen horizontal; die Zeichnung steht unterhalb der
   Eingabefelder zur Verfügung.

## Vergleiche mit echten Schnittsystemen

Unter der Hosenkonstruktion befindet sich ein Link zum öffentlichen
**Müller-&-Sohn-Grundschnitt Standardhose und Bundfaltenhose**:

https://www.muellerundsohn.com/allgemein/schnittmuster-grundschnitt-hose-und-bundfaltenhose/

- Diese Zeichnung ist eine **fachliche Ziel- und Vergleichsreferenz**,
  **keine** Behauptung, dass der derzeitige TextilLab-Code deren
  Konstruktion exakt reproduziert.
- Die Bilder bleiben beim Originalverlag; wir binden keine urheberrechtlich
  geschützten Zeichnungen ungefragt in das Repository ein.
- Für den konkreten Formelabgleich kann die bestehende
  \`pants-reference-template.json\` mit eigenen Kontrollmaßen und
  bibliografischen Angaben zum vorhandenen Fachbuch gefüllt werden.
- Das öffentliche Beispiel zeigt unter anderem getrennte
  Vorder-/Hinterhose, Fadenlauf und Abnäher. Systematische, nummerierte
  Kontrollpunkte und Bemaßungslogik lassen sich ergänzend an der
  öffentlichen Dokumentation von GRAFIS Hose 60 studieren:
  https://help.grafis.de/de-de/Content/Interaktive_Konstruktionen/Hosen/Hose_60/Masse_Hose_60.htm
  (ein anderes, eigenständiges Schnittkonstruktionssystem).

## Hinweise

- Keine fertige 3D-Anprobe, kein Produktions- und kein beglaubigtes
  Müller-&-Sohn-Schnittmuster.
- Nicht alle Teilungen und Konfektionskanten sind fachlich geprüft.
- Nahtzugaben und Materialverbrauch bleiben vorläufige Näherungen.
- Der 2D-Editor bietet interaktive Auswahl, editierbare Maßzahlen,
  Zoom und Vorlagenwechsel; freie Änderung der Bézier-Kurven direkt
  durch Ziehen der Stützpunkte ist noch nicht enthalten.
- Die Website nutzt den bestehenden JavaScript-Kern; die Rust-Version
  wird durch dieses reine Oberflächen-Update nicht verändert.
- Die Zuordnung Taille–Hüfte wurde für die Hose korrigiert, sodass das
  gleichnamige Oberteilfeld nicht mehr irrtümlich verwendet wird.

## Testkriterien

- \`node --check\` aller eingebundenen Frontend-Dateien
- UI-Regressionschecks für einzigartige HTML-IDs, vollständige
  Vorlagenauswahl, passende Maßgruppen, Zuordnung von SVG-Schnittteilen,
  und Quellenkennzeichnung
- Bestehende Generator- und CLI-Tests auf Windows, Linux und macOS.

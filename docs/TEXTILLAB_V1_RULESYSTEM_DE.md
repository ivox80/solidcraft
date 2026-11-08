# TextilLab v1.0 – Experimentelles regelbasiertes Schnitt-CAD

**Entwicklungsstatus:** TextilLab Experimental Construction System 1.0. In der
Browser-Demo und im Rust-Geometriekern implementiert; derzeit **keine
geprüfte Fertigungs- oder Schneiderkonstruktion** und **kein lizenziertes
Müller-&-Sohn-/Schnittsystem**.

Die Versionen 0.8 bis 1.0 bauen auf den getrennten VT-/RT-Grundschnitten und
zwei Ärmeln aus v0.7 auf.

## v0.8 – grafische technische Konstruktionsansicht

Die Browserzeichnung zeigt – ein- und ausblendbar:

- Hilfslinien für Brust (Br), Taille (Ta), Hüfte (Hü), Saum (Sa),
  Oberarm (OB), Ellenbogen (ELL) und Ärmelsaum (AS)
- Punktnamen: MV/MH (Mitte vorn/hinten), HN (Hals), SP (Schulterpunkt),
  AL (Armloch), BP (Brustpunkt), Ä0, ÄS
- Maßlinien mit Pfeilen und Zahlen: VL (Vorderlänge), RL (Rückenlänge),
  Sch (Schulterbreite als Eingabereferenz), BP/2 (halber Brustpunktabstand),
  ÄL (Ärmellänge), OB (Oberarmumfang als Eingabereferenz)
- K1/K2/K3/K4 als Längenkorrektur-Linien sowie Fadenlauf (FDL)

**Wichtig:** Ein Maßtext kann den vollständigen Körperwert oder eine
Konstruktionsreferenz anzeigen; nicht jede markierte Strecke ist der gesamte
Körperumfang. Insbesondere Schulterbreite und Oberarmumfang sind
Körper-Eingabewerte und keine identische Distanz zu einer halben Schnittkontur.

**Bedienung:** Auf eine Bemaßung klicken, um das zugehörige Maßfeld
zur Bearbeitung zu fokussieren. Die Layer können im Seitenpanel deaktiviert
werden.

## v0.9 – parametrisierte Regeln und Nahtlängenprüfung

Die eigenen TextilLab-Konstruktionsregeln werden als Referenzwerte angezeigt:

| ID | Konstruktionsgröße | Berechnung |
|---|---|---|
| R01 | Brust-Teilungsweite | (Brustumfang + Mehrweite) / 4 |
| R02 | Taillen-Teilungsweite | (Taillenumfang + Mehrweite) / 4 |
| R03 | Hüft-Teilungsweite | (Hüftumfang + Mehrweite) / 4 |
| R04 | Armlochhöhe | eingegebene Armlochtiefe |
| R05 | Vorderlänge | Schulterreferenz bis Taille |
| R06 | Rückenlänge | Schulterreferenz bis Taille |
| R07 | Ärmelkugellänge | Armloch VT + RT + Ziel-Einhalteweite |
| R08 | Seitennaht-Abweichung | absoluter Nahtlängenunterschied VT / RT |

**Funktionierende algorithmische Prüfung:**

- Die 3D-/2D-freien Schnittkonturen sind als Bézier-Kurven mit fein
  abgetasteten Punkten vorhanden. Gemessen werden tatsächliche
  Polygonbogensummen, nicht nur gerade Abstände.
- Die untere Armlochhöhe ist bei VT und RT auf derselben
  Konstruktionslinie ausgerichtet. Das verbessert die Seitennahtbalance.
- Die Höhe der Ärmelkugel wird numerisch variiert, bis ihre **gemessene
  Kurvenlänge** dem Summen-Armloch zuzüglich gewählter Einhalteweite
  entspricht – im Lösungsbereich mit Zielabweichung maximal 0,1 mm.
- Wenn sich das Ziel innerhalb der technischen Höhenbegrenzung nicht
  erreichen lässt, wird der Wert als nicht erreichbar gekennzeichnet.

Die numerisch passende Nahtlänge ist **keine Garantie für eine nähbare
Ärmelkugel**; Form, Ärmelneigung, Kerbenposition, Stoffdehnung,
Einhalteverteilung und Schulterbalance müssen zusätzlich konstruiert und
anprobiert werden.

## v1.0 – Regelbasierte Abwandlungen und Änderungen

**Passformen:** körpernah, Regular, bequem, weit, Oversize,
benutzerdefinierte Mehrweite. Der gewählte Wert wirkt auf die
Konstruktionsmaße, nicht bloß auf das SVG.

**Modellvarianten:** Grundschnitt, Shirt, Bluse, Jackenbasis,
Mantelbasis, ärmellos. Die Varianten nutzen definierte, derzeit
vereinfachte Mehrweiten-/Längenregeln. Die ärmellose Variante entfernt
die zwei Ärmel-Schnittteile tatsächlich aus der Stückliste,
Zeichnung und Materialberechnung.

**Vier Längenkorrekturen (in cm, negativ = kürzer):**

- K1: Brust–Taille → ändert **Vorder- und Rückenlänge**, nicht die Ärmel
- K2: Taille–Hüfte → verändert **Taille–Hüfte und Taille–Saum**
- K3: Unterhalb Hüfte → verändert nur **Taille–Saum**
- K4: Ärmel → verändert nur **Ärmellänge**

Alle Änderungen berechnen den Grundschnitt über seine Körpermaße und
Bézier-Stützpunkte neu. Sie sind **noch keine geometrischen
Schnitt-und-Spreiz-Operationen** mit Stückteilung und Abnäherrotation.

**Projektfunktionen:** Änderungsverlauf mit Rückgängig/Wiederholen;
Export und Import einer JSON-Projektdatei mit den Maß- und
Konstruktionsparametern. Die Datei ist eine eigene TextilLab-Begleitdatei,
nicht eine native SolidCraft-.sc-Timeline-Datei.

## Website

Sofern GitHub Pages für das Repository freigeschaltet ist:

https://ivox80.github.io/solidcraft/textillab/

1. Als Grundschnitt „Oberteil – VT / RT + Ärmel“ auswählen.
2. Körper- und Konstruktionsmaße in cm eingeben.
3. Im Abschnitt **Schnittsystem** Passform, Modell und Einhalteweite einstellen.
4. Unter **Verlängern / Kürzen** die Änderung für K1–K4 einstellen.
5. Konstruktionslinien, Maßangaben, Punktnamen, Änderungslinien und
   Kontrollwerte über **Zeichnungsebenen** ein-/ausblenden.
6. Bei Bedarf die SVG-Ausgabe und das Projekt-JSON herunterladen.

## Rust-CLI

Im Ordner \`textillab\`:

~~~powershell
cargo test
cargo run --release --bin textillab-pattern -- top chest=108 waist=90 hip=110 frontLength=47 backWaist=44 --fit comfort --style blouse --cap-ease 24 --torso-mm 20 --hip-mm 10 --hem-mm 30 --sleeve-mm 15 --out exports/oberteil-v1.svg

cargo run --release --bin textillab-pattern -- top --style sleeveless --out exports/oberteil-aermellos.svg
~~~

Bei der CLI werden die Körpermaße in **cm**, Längenänderungen in **mm**
und die Ziel-Einhalteweite in **mm** angegeben. Der Rust-Kern exportiert
zusätzlich zur Schnitteilkontur benannte Punkte, Bemaßungsreferenzen,
Korrekturlinien und die berechneten Nahtkontrollen.

## Grenzen für reale Konfektion

- Noch **kein** vollständig geprüfter Schnitt nach Müller & Sohn,
  Aldrich oder einer anderen dokumentierten professionellen Lehre.
  Ohne nachvollziehbare und lizenzrechtlich zulässige Quellregeln wird
  eine solche Herkunft nicht behauptet.
- Schulter-/Brustabnäher sind bisher **Konstruktionsmarken**, kein
  geometrisch fachgerecht zugedrehter und verlegter Abnäher.
- Die Korrekturfunktionen rechnen Längen neu; echtes Slash-and-Spread
  und Nahtlängen-Abgleich nach Abnäherdrehung bleiben offen.
- Noch keine Stoffmaterialmodelle, Dehnungszonen, Zug-/Druckeinhaltewerte,
  Qualitätskontrolle auf selbstschneidende Nahtzugaben, passgenaue
  Ärmel-Passzeichen oder Konfektionsgrößen-Graduierung.
- Die SVG-Ausgabe ist eine Vorschau. Produktionsfähigkeit erfordert
  physische Anprobe, Fachprüfung der Nahtlinien, Nahtzugaben und
  Materialeigenschaften.
- Die Browser-Implementierung ist JavaScript; die Rust-Implementierung
  besitzt dasselbe Regelkonzept, ist aber **nicht formal bitgenau
  identisch validiert**.
- Die eigenständige native TextilLab-Integration in SolidCraft befindet
  sich weiterhin in einer separaten Entwicklungslinie; v1.0 in diesem
  Repository ist noch **kein** integriertes parametrisches SolidCraft-.sc
  Feature.

## Quellen und technische Orientierung

Die Regeltexte im Modul sind eigene TextilLab-Entwurfsformeln. Für die
fachliche Weiterarbeit wurden öffentlich verfügbare Erläuterungen zu
Ärmel-Einhalteweite und Längenänderungen herangezogen:

- https://dresspatternmaking.com/patternmaking-basics/terminology/ease-sleeves
- https://utia.tennessee.edu/publications/wp-content/uploads/sites/269/2023/10/W1061E.pdf

Diese Quellen sind **kein** Beleg, dass der eigene TextilLab-Prototyp
die jeweiligen Schneiderkonstruktionssysteme vollständig umsetzt.

## Release-Kriterien

- Rust \`cargo fmt --check\`, \`cargo clippy -D warnings\`, Unit-Tests
  und Release-Build auf Windows, Linux und macOS
- JavaScript-Syntax, Browserregressionen inkl. Änderungszonen,
  Overlay-Layern, Nahtbalance, Passformprofilen
- Rust CLI-Smoke-Tests für reguläre und ärmellose Varianten
- Erst bei vollständig grünem Build Merge in \`main\`

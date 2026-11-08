# TextilLab v0.7 – VT/RT-Oberteil-Grundschnitt v2

## Ziel und Bildreferenz

Die neue Ausgabe orientiert sich an einem technischen Grundschnitt mit **VT**
(Vorderteil) und **RT** (Rückenteil). Beide Konturen werden gesondert
parametrisch konstruiert. Die SVG-Zeichnung enthält:

- die vier Hilfslinien **Brust, Taille, Hüfte und Saum**
- getrennte **Hals-, Schulter- und Armlochkurven** (kubische Bézier-Segmente)
- **Schulter-/Brustabnäher** (front) und Schulterabnäher (back)
- **Taillenabnäher** in beiden Teilen als Markierungen
- **FDL/Fadenlauf**, Markierung des Brustpunkts
- **zwei Ärmel** auf Basis einer separaten Schnittteilkontur mit asymmetrischer Ärmelkugel, Oberarmweite, Handgelenkweite und Fadenlauf
- Naht- und Zuschneidlinien, Vorderteil und gespiegeltes Rückenteil
- Flächen- und Stoffgewichtsberechnung als Näherung für **1 × Vorderteil, 1 × Rückenteil und 2 × Ärmel**

Im Unterschied zum bisherigen Polygonentwurf sind die Konturen mathematisch
durch Kontrollpunkte und Bézier-Splines geführt. VT und RT sind an einen
gemeinsamen Taillen-Nullpunkt ausgerichtet, damit die Hilfslinien übereinander
liegen.

## Neu einstellbare Körpermaße

Die Website nimmt alle Maße in **cm** entgegen. Der Rust-Kern verwendet **mm**.

| Maß | Standard (cm) | Bedeutung |
| --- | ---: | --- |
| Brustumfang | 100 | Umfang auf Brusthöhe |
| Taille | 84 | Umfang auf Taillenhöhe |
| Hüfte | 102 | Umfang auf Hüfthöhe |
| Schulterbreite | 44 | Gesamt-Schulterbreite |
| Mehrweite | 8 | Zusätzliche konstruktive Weite |
| Vorderlänge | 46 | Oberer Schulter-/Halsansatz bis Taille |
| Rückenlänge | 43 | Hinterer Schulter-/Halsansatz bis Taille |
| Brusthöhe | 27 | Hals-/Schulteransatz bis Brustpunkt |
| Brustpunktabstand | 19 | Abstand zwischen beiden Brustpunkten |
| Halsumfang | 38 | Umfang des Halsansatzes |
| Armlochtiefe | 23 | Schulteransatz bis Armlochhöhe |
| Taille–Hüfte | 20 | Vertikaler Hüftabstand |
| Taille–Saum | 29 | Vertikale Saumlänge unter der Taille |
| Schulterabfall | 3 | Vertikaler Unterschied von Halsansatz zur Schulterspitze |
| Oberarmumfang | 34 | Weite des Oberarms / Ansatzpunkt der Ärmelweite |
| Handgelenkumfang | 19 | Basis für Ärmelsaumweite |
| Armlänge | 62 | Länge ab Schulter (bisheriges Standardmaß) |

Brust-/Taillen- und Schulterabnäher werden zunächst als technische Nähmarkierungen
über die Kontur gelegt. **Die Abnäher sind noch nicht rotiert bzw. in die tatsächliche
Zuschnittkontur integriert.** Ihre Nahtlängen und Einhalteweiten sind daher
noch nicht fertigungstauglich. **Die Ärmelkugel ist noch nicht auf die tatsächliche Armlochkurvenlänge abgeglichen und ihre Einhalteweite ist nicht geprüft.** Die äußere Nahtzugabe ist ein einfaches
Polyline-Offset ohne robuste Selbstüberschneidungsauflösung.

## Website

Wenn GitHub Pages aktiviert ist, öffne:
https://ivox80.github.io/solidcraft/textillab/

1. Grundschnitt **Oberteil – VT/RT Grundschnitt v2** auswählen.
2. Unter **Körpermaße** Brust, Taille, Hüfte, Schulter und Mehrweite anpassen.
3. Unter **Oberteil v2 – Konstruktionsmaße** Vorder-/Rückenlänge, Brusthöhe,
   Brustpunktabstand, Halsumfang und Armlochtiefe ändern.
4. Zeichnung mit VT, RT, Hilfslinien, Abnähern und FDL ansehen.
5. **SVG exportieren**.

Beispielzeichnung: docs/textillab/examples/bodice-v2.svg

## Rust-Kern und CLI

~~~powershell
cd textillab
cargo test
cargo run --release --bin textillab-pattern -- top chest=106 waist=86 hip=110 shoulder=44 frontLength=47 backWaist=43 bustHeight=27 bustSpan=19 neck=38 armholeDepth=23 waistToHip=20 hemBelowWaist=29 shoulderSlope=3 --allowance 10 --out exports/oberteil-v2.svg
~~~

HINWEIS: Dies ist ein **geometrischer Grundschnitt-Prototyp**, keine genormte
oder auf echte Körperformen fertig auskonstruierte Passform. Der Abgleich von
Schulternaht, Armloch, Abnähern und Größenverhältnissen muss mit einem
Probeteil erfolgen.

## Nächster Schritt

- präzises Verlegen und Schließen der Schulter- und Taillenabnäher
- Seitennahtlängen und Schulterlinien paarweise überprüfen
- unterschiedliche Oberteilvarianten und ärmellose/mit Ärmel
- am Bildschirm editierbare Kontrollpunkte, grafische Bemaßungen
- robuste Nahtzugaben auch an konkaven Kurven und Ecken
- echte parametrische SolidCraft-Historienfeatures

## Erscheinungsbild / Ärmel in v0.7.1

Das Oberteil verwendet wieder den **dunklen SolidCraft-Stil** wie die anderen Schnittvorlagen: dunkelblauer Hintergrund, **türkise gestrichelte Nahtlinie**, **orange Zuschnittlinie**, rot-orangene Abnäher und hellblaue Hilfslinien.

Rechts neben VT und RT wird die **Ärmel-Schnittkontur (ÄR)** mit Ärmelkugel, Fadenlauf, Hilfslinie auf Oberarmhöhe und zwei Referenzmarken eingeblendet. **2× zuschneiden** ist als Stückzahl gespeichert, daher enthält die SVG-Mengenberechnung nun **vier zu schneidende Teile**.

Die Eingaben „Oberarmumfang“ und „Handgelenkumfang“ sind im Oberteil-Maßformular ergänzt. Die bisherige „Armlänge“ steuert die Ärmellänge.

Es handelt sich weiterhin um eine **konstruktive Vorschau**: Ohne Abgleich von Armloch- und Ärmelkugellänge, Einhalteweite, Stoffdehnung und Nahtverlauf ist der Ärmel noch nicht fertigungsreif.

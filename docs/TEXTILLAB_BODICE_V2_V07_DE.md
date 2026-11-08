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
- Naht- und Zuschneidlinien, Vorderteil und gespiegeltes Rückenteil
- Flächen- und Stoffgewichtsberechnung als Näherung

Im Unterschied zum bisherigen Polygonentwurf sind die Konturen mathematisch
durch Kontrollpunkte und Bézier-Splines geführt. Beide Teile sind an einen
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

Brust-/Taillen- und Schulterabnäher werden zunächst als technische Nähmarkierungen
über die Kontur gelegt. **Die Abnäher sind noch nicht rotiert bzw. in die tatsächliche
Zuschnittkontur integriert.** Ihre Nahtlängen und Einhalteweiten sind daher
noch nicht fertigungstauglich. Die äußere Nahtzugabe ist ein einfaches
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

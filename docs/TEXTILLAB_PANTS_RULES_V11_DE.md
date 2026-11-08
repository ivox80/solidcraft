# TextilLab v1.1 – Regelbasierte Hosenkonstruktion und Fachbuchabgleich

## Fachliche Trennung

**Stand:** technische Konstruktions-Prototypen mit nachvollziehbaren Regeln,
keine Produktionsfreigabe.

Die Software trennt ab sofort:

1. **Konstruktionssystem**: definierte Regel-ID, Buch-/Quellenangabe,
   Körpermaße und Zugaben
2. **Konstruktionspunkte**: benannte Punkte und voneinander getrennte
   Vorder-/Hinterhose, Schrittkurven, Knie-, Saum-, Bund- und Hüftlinien
3. **Schnittkontur**: mit kubischen Bézierkurven und definierter Kopienzahl
4. **Prüfung**: gemessene Seiten- und Innenbeinnähte, Warnhinweise,
   Stoffflächen- und Gewichtsschätzung
5. **Vergleich**: gemessene Zahlen eines zweiten dokumentierten Systems
   ohne Übernahme von Textabbildungen oder geschützten Buchseiten

### Keine falsche Systembezeichnung

Der derzeit ausführbare Hosen-Regelsatz heißt ausdrücklich
\`textillab.pants.experimental.1\`. Er besteht aus **eigenen experimentellen
Formeln** und darf NICHT als Grundschnitt nach Müller & Sohn etikettiert
werden. Im Dropdown ist „Müller & Sohn“ deshalb noch deaktiviert.

Müller-&-Sohn-Bücher des Nutzers sind besonders nützlich für die Prüfung der
eigenen Ergebnisse. Für eine eigenständige, belegbare Implementierung müssen
wir **exakte Fachbuchausgabe, Seitenzahl, zugrunde gelegte Maßtabelle und
Regeln** kontrollieren, bevor konkrete Formeln oder Figurenregeln behauptet
oder implementiert werden. Der bloße Besitz eines Buches bedeutet keine
Erlaubnis, Text oder Grafiken des Buches öffentlich zu vervielfältigen.

## Neuer Hosen-Grundschnitt

Die neue Hose besitzt nur noch zwei Grundschnitt-Typen, jeweils **2×**
zuzuschneiden (Vorderhose VH, Hinterhose HH). Bund und Taschen bleiben
separate **künftige** Modellentwicklungen, statt fälschlich als fertige
Schnittteile angezeigt zu werden.

Bei beiden Teilen werden eigenständige glatte Konturen berechnet:

- Taille / Hüfte mit Seitennahtformung
- Schrittlinie mit separater vorderer und hinterer Schrittausladung
- gekrümmte vordere und hintere Schrittkurve (keine identischen Kurven)
- Innenbeinnaht und Seitennaht mit jeweils eigenem Kurvenverlauf
- Knielinie und Saumlinie, Fadenlauf/Bügelfalte (FDL)
- markierter Taillenabnäher hinten
- benannte Bezugspunkte WC/WS/HS/HC/CR/RS/KS/KI/AS/AI
- numerischer Vergleich der zugehörigen Nahtlängen

### Körpermaße in cm

| Parameter | Standard | Wirkung |
|---|---:|---|
| Taille | 84 | Bundabschnitt / Seitennaht oben |
| Hüfte | 102 | Hüftbreiten und Schrittansatz |
| Schrittlänge | 80 | Höhe des Schrittpunktes relativ zur Saumlinie |
| Seitenlänge | 108 | Gesamthöhe des Beines |
| Leibhöhe | 28 | unabhängiger Kontrollwert zur Schrittlinie |
| Taille–Hüfte | 20 | Hüftlinie |
| Taille–Knie | 60 | Knielinie |
| Oberschenkel | 64 | schrittnahe Weitenführung |
| Knie | 48 | Beinweite auf Kniehöhe |
| Saum je Bein | 44 | Saumweite |
| Hüft-Zugabe | 4 | Weitenverteilung |
| Taillen-Zugabe | 2 | Weitenverteilung |
| Hinterer Abnäher | 2 | rechnerische Taillenformungsreserve |
| Zusätzliche Schrittausladung | 0 | vordere / hintere Schrittpunkte |

Die voreingestellten Werte sind **Beispielwerte**, keine Normgrößentabelle.

### TextilLab-Regeln H01–H10

| ID | Bedeutung | Herkunft |
|---|---|---|
| H01 | Hüftviertel mit Zugabe | TextilLab Experimental |
| H02 | Taillenviertel mit Zugabe | TextilLab Experimental |
| H03 | Schrittlinie aus Seitenlänge minus Schrittlänge | direkt gemessene Beziehung |
| H04 | vordere zusätzliche Schrittausladung | TextilLab Experimental |
| H05 | hintere zusätzliche Schrittausladung | TextilLab Experimental |
| H06–H07 | Verteilung Knieweite VT/HH | TextilLab Experimental |
| H08 | Differenz Seitennahtlängen | gemessene Polylinienlängen |
| H09 | Differenz Innenbeinnahtlängen | gemessene Polylinienlängen |
| H10 | hintere Abnäherreserve | TextilLab Experimental |

Alle einzelnen Formeln stehen sowohl im JS-Datei-Quellcode
\`docs/textillab/pants-system.js\` als auch im Rust-Code
\`textillab/src/pants_rules.rs\`; die App zeigt die Formeln im Tooltip
der jeweiligen Regelzeile.

Wichtig: Ein gemessener Nahtlängenunterschied von z. B. 7 mm ist
**kein automatisches „Ergebnis korrekt“**. Was bei welchem Modell wie
viel eingehalten, gedehnt oder ausgeglichen wird, hängt vom Schnittsystem,
Stoff und der Passform ab. Ebenso muss die Schrittkurve separat geprüft
werden. Die benutzten Offset-Nahtzugaben sind noch nicht robust genug
für fertigen Zuschnitt auf allen konkaven Konturen.

## Fachbuch-Vergleich

Für das Einlesen eigener Kontrollwerte dient
\`docs/textillab/examples/pants-reference-template.json\`.

Schema: \`textillab-reference/1\`.

Der Datensatz erfordert Buchtitel, Auflage/Ausgabe und Seitenangabe
sowie dieselben Körpermaße (Taille, Hüfte, Schrittlänge, Seitenlänge,
Leibhöhe). Ohne passende Ausgangsmaße verweigert das Programm einen
Scheinvergleich.

Beispielablauf:

1. Buchausgabe wählen, z. B. „DOB Schnittkonstruktionen für Röcke und
   Hosen“, 29. Auflage (2022), wenn **genau diese** Ausgabe vorliegt.
2. Aus deren vollständiger Konstruktion nur eigene gemessene
   Kontrollwerte der Schnittteile in die Vergleichstabelle eintragen,
   keine gescannten Abbildungen im öffentlichen Repo ablegen.
3. JSON unter **Buch-Referenzvergleich** laden.
4. Software zeigt die Differenz \`TextilLab – Buchkontrollwert\` in mm.
5. Bei Abweichungen die entsprechenden H-Regeln gezielt prüfen,
   kommentieren und nach fachlicher Validierung parametrisieren.

Aktueller Vergleich: Seiten-/Innenbein-, Taillen- und
Schrittkurvenlängen für VH/HH. Weitere Vergleichspunkte und
Konstruktionselemente lassen sich später ergänzen.

## Rust-CLI

~~~powershell
cd textillab
cargo test
cargo run --release --bin textillab-pattern -- trousers waist=90 hip=112 inseam=82 outseam=110 crotchDepth=28 waistToHip=20 waistToKnee=61 thigh=66 knee=49 hem=44 hipEase=4 waistEase=2 dartIntake=2 --allowance 10 --out exports/hose-v11.svg
~~~

Alle neuen Hosenparameter werden in **cm**, Nahtzugaben in **mm**
angegeben. Die 4 zu schneidenden Teile werden für den Flächen-/Gewichtswert
berücksichtigt. Die SVG enthält die Referenz-ID des Konstruktionssystems.

## Gewählte externe Fachreferenzen

Die folgenden Quellen dienten zur **Recherche und fachlichen
Orientierung**, nicht zur Behauptung, die Formeln daraus wort- oder
maßgleich implementiert zu haben:

- Müller & Sohn: [DOB Schnittkonstruktionen für Röcke und Hosen](https://www.muellerundsohn.com/shop/dob-schnittkonstruktionen-fuer-roecke-und-hosen/),
  29. Auflage (2022), einschließlich Maß-, Zugaben- und
  Konstruktionsübersichten
- Müller & Sohn: [DOB Schnittkonstruktionen Hosen](https://www.muellerundsohn.com/shop/dob-schnittkonstruktionen-hosen/),
  3. Auflage (2026), verschiedene Hosenarten und Figurtypen
- The Shapes of Fabric: [Learn How to Draft the Basic Pants Pattern](https://www.theshapesoffabric.com/2020/08/16/learn-how-to-draft-the-basic-pants-pattern/),
  frei zugängliche visuelle Anleitung für Körpermessung, getrennte
  Schrittkurven, Bügelfalte und Vorder-/Hinterhose
- FreeSewing: [Dokumentation der Einstellungen](https://freesewing.eu/docs/editor/menus/settings/),
  Trennung von Modelloptionen, Körpermaßen, Sichtbarkeit einzelner
  Teile, Druckmaßen und Nahtzugabe

## Offene fachliche Arbeiten

- Eingabe konkreter und rechtlich zulässig übernommener Müller-&-Sohn-
  Berechnungsregeln mit Varianten und Quellenmetadaten
- Prüfung der Leibhöhe, der Hüft- und Oberschenkelweite, der
  tatsächlichen Schrittkurven und der Saumhöhe
- Auflösen/Schließen von Abnähern mit geometrischer Aktualisierung
- Bund, Taschen, Verschluss, Reißverschluss, Verknöpfung und Varianten
- automatische Einschnitt-/Einhalteverteilung und Test auf
  Überlappungen und Selbstschnitt der Kontur
- Größentabellen, Gradierung, Körperformkorrekturen und reale Probestücke
- dieselbe Rückverfolgbarkeit für Oberteil, Ärmel und Jacke
- native parametrische SolidCraft-Feature-Historie

**Die Version ist ein nachvollziehbarerer CAD-Prototyp, noch kein
Müller-&-Sohn-verifiziertes Fertigungssystem.**

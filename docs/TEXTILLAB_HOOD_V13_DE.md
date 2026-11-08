# TextilLab v1.3 – Kapuzenatelier und Nahtlängenprüfung

## Ergebnisse und Quellenstatus

TextilLab besitzt jetzt einen **parametrischen zweiteiligen Kapuzenprototyp**
mit zwei gespiegelten Kapuzenseiten, deren reale Nahtbogenlängen rechnerisch
mit den vorgegebenen Halslochlängen des Kleidungsstücks verglichen werden.
Der JS-Browser und die Rust-Bibliothek implementieren die Funktion.

Browser: https://ivox80.github.io/solidcraft/textillab/

- Damen: examples/kapuze-damen-v13.svg
- Herren (aufgelockert): examples/kapuze-herren-locker-v13.svg
- Kind: examples/kapuze-kind-v13.svg

Fachliche Anregung: Müller & Sohn, [Schnittkonstruktion Kapuze]
(https://www.muellerundsohn.com/allgemein/schnittkonstruktion-kapuze/).
Der öffentlich sichtbare Beitrag nennt die knapp anliegende und
aufgelockerte Kapuzenform und Hinweise zu weiteren Varianten. Die
eigenen TextilLab-Kurven sind jedoch **nicht die nachgezeichnete,
verifizierte Müller-&-Sohn-Konstruktion**. HAKA- und Kinderregeln aus
Fachbüchern werden nicht erfunden oder als offiziell gekennzeichnet.

### Unabhängige Eingabeprofile (alle Maße in cm)

| Profil | Kopf | Hals hinten/Seite | Hals vorn/Seite | Höhe | Tiefe |
| --- | ---: | ---: | ---: | ---: | ---: |
| Damen / DOB | 57 | 10 | 12 | 32 | 27 |
| Herren / HAKA | 59 | 10,5 | 12,5 | 34 | 28 |
| Kinder (8 J.) | 52 | 8 | 9,5 | 28 | 24 |

Das sind **nur beispielhafte Eingabewerte**, keine geprüften
Konfektions-/Größentabellen. Die Halslochlängen müssen **an der
tatsächlichen Nählinie des späteren Oberteils** gemessen werden.
Bei der aufgelockerten Variante verwendet der Prototyp eigene
Zuschläge von +3 cm Höhe und +4,5 cm Tiefe.

### Nahtlängen und Behandlungsarten

Der Generator baut die hintere und vordere Halslochkurve aus
kubischen Beziersegmenten auf. Eine Bisektionsrechnung bestimmt deren
Längsausdehnung, bis die **gemessenen Bogenlängen** die Sollwerte
innerhalb von ±0,25 mm treffen. Die gegenüberliegende Mittelnaht ist
spiegelgleich.

- Gleichlang: Nahtlinie A und B innerhalb der Toleranz gleich.
- Einhalten: Teil A ist um eine **ausdrücklich gewählte** Mehrweite
  länger; vorderer und hinterer Halsbereich teilen den Betrag auf.
- Dehnen: Teil A ist kürzer, aber nur mit **ausdrücklich bestätigter
  gemessener Materialdehnungsfähigkeit**; sonst wird die Rechnung gesperrt.

Der **allgemeine Nahtprüfer** befindet sich unter
docs/textillab/seam-contracts.js. Browser: hood-system.js.
Rust: textillab/src/hood_rules.rs.

### Kinder-Schutzmaßnahmen

Kinder sind ein eigener Profilmodus; das Alter muss 1–14 Jahre
betragen. Halskordeln werden softwareseitig gesperrt. Relevante
Sicherheitsquelle: DIN EN 14682:2015-03; Kinderbekleidung –
https://www.kindergesundheit-info.de/themen/sicher-aufwachsen/sicherheit-im-alltag/kleidung

Die Software gewährleistet damit noch **keine** umfassende Sicherheit:
Sicht und Gehör, Spielraum, Belastung, Passform und jeweilige
rechtliche Anforderungen müssen am realen Kleidungsstück geprüft werden.

### Rust-CLI

~~~powershell
cd textillab
cargo test hood_rules
cargo run --release --bin textillab-pattern -- hood --profile women head=57 backNeck=10 frontNeck=12 --allowance 10 --out smoke-exports/hood.svg
cargo run --release --bin textillab-pattern -- hood --profile men --style relaxed --out smoke-exports/hood-men.svg
cargo run --release --bin textillab-pattern -- hood --profile child ageYears=8 --out smoke-exports/hood-child.svg
cargo run --release --bin textillab-pattern -- hood --neck-mode ease --planned-ease 6 --out smoke-exports/hood-ease.svg
cargo run --release --bin textillab-pattern -- hood --neck-mode stretch --stretch 4 --material-stretch-verified --out smoke-exports/hood-stretch.svg
~~~

Wichtig: Alle Körpermaße in cm, Nahtzugabe/Einhalten in mm,
Dehnungsfähigkeit in Prozent.

## Verbindlich offene Fachprüfung

- Die Originalregeln aus der konkreten Müller-&-Sohn-Buchausgabe
  müssen vor einer behaupteten Fachkonformität dokumentiert und mit
  Probeteilen geprüft werden.
- Die **aktuell nur eingegebenen Halsloch-Anschlussmaße** sind noch
  nicht direkt aus einer parametrischen Jacken-/Oberteilzeichnung
  abgeleitet. Die Software kann deshalb nicht nachweisen, dass ein
  anderes Kleidungsstück tatsächlich dieselbe Nahtlinie hat.
- Passform, Halsbalance, Gesichtsfeld, Nahtverarbeitung,
  Abnäherschluss, Einhalteverteilung sowie Zugaben-Offsets bei
  konkaven Kurven und Ecken sind nicht fertigungsvalidiert.
- Normkonformität für Kinderbekleidung ist nicht automatisch gegeben.

Deshalb bleibt die Ausgabe explizit als **KONSTRUKTIONSPROTOTYP,
NICHT FÜR FERTIGUNG VALIDIERT** gekennzeichnet – auch wenn alle
Nahtlängenprüfungen OK melden.

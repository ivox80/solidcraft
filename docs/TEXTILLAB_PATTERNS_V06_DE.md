# TextilLab v0.6 – Parametrischer Grundschnitt-Katalog

Neu: sieben Grundschnitt-Familien mit individuell eingebbaren Körpermaßen.
Die Vorschau ist verfügbar in der Browser-Demo unter docs/textillab und als
separater Rust-Schnittmustergenerator.

## Grundschnitt-Familien

| Vorlage | Teile | Haupteingaben |
| --- | --- | --- |
| Oberteil / T-Shirt | Vorderteil, Rückenteil, Ärmel | Brust, Taille, Schulter, Oberteil-/Armlänge |
| Jacke | 2 Vorderteile, Rücken, Ärmel, provisorischer Kragen | Brust, Taille, Schulter, Länge, Mehrweite |
| Hose | Vorderhose, Hinterhose, gerader Bund | Hüfte, Taille, Schrittlänge |
| Handschuh | Handfläche, Handrücken, vereinfachter Daumen | Handumfang, Handlänge |
| Mütze | 4 Keile, Bündchen | Kopfumfang |
| Hut | 6 Kronensegmente, Hutdeckel, Krempe mit Loch | Kopfumfang |
| Rock | Vorderteil, Rückenteil, Bund | Taille, Hüfte, Länge |

## Bedienung im Browser

Falls GitHub Pages aktiviert ist:
https://ivox80.github.io/solidcraft/textillab/

- Links im Dropdown „Vorlage wählen“ eine Grundschnitt-Familie auswählen.
- Unter „Körpermaße anpassen (cm)“ Brust, Taille, Hüfte usw. eingeben.
- Bei Bedarf die Mehrweite und Nahtzugabe einstellen.
- Rechts wird die 2D-Schnittmuster-Skizze neu erzeugt; links stehen
  die Stückliste sowie Hinweise pro Teil.
- SVG exportieren.

Die Datei wird im Browser berechnet. Das 3D-OBJ-Abwicklungstool steht weiter
als Dropdown-Eintrag „3D-Modell abwickeln (OBJ)“ zur Verfügung.

## Rust-CLI (Windows, Linux, macOS)

Im Repository-Ordner:

~~~powershell
cd textillab
cargo test
cargo run --release --bin textillab-pattern -- trousers hip=112 waist=90 inseam=82 --allowance 10 --out exports/hose.svg
cargo run --release --bin textillab-pattern -- top chest=105 waist=86 shoulder=45 sleeve=62 --out exports/oberteil.svg
cargo run --release --bin textillab-pattern -- hat head=59 --out exports/hut.svg
~~~

Eingabemaße: chest waist hip back shoulder sleeve inseam head hand
handLength ease, jeweils in **cm**. Die Nahtzugabe ist in **mm**.
Flächengewicht über „--density“ in g/m².

## Entwicklungsstatus / Sicherheit

**Die Zeichnungen sind Konstruktions-Prototypen, keine fertig graduierten
oder auf Passform geprüften Schneider-Grundschnitte.** Zu prüfen:

- Armloch und Ärmelkugel einschließlich Einhalteweite
- Passgenaue Schrittkurve, Sitzhöhe und Abnäher bei Hosen
- Fingerlängen und Fingerzwischenstücke bei Handschuhen
- Nahtlängen sowie Umfangsabgleich bei Mützen und Hüten
- Produktionsreife Nahtzugaben bei konkaven Konturen und Innenlöchern
- Materialdehnung, Fadenlauf, Stoffbruch, Verschnitt und Größenabstufung

Nahtzugaben werden als einfache Polylinien-Offsets dargestellt.
Bei starken Einbuchtungen / scharfen Winkeln können sie
ungültige oder überlappende Zuschnittkonturen ergeben. Die
Massen- und Flächenwerte sind deshalb derzeit **Schätzwerte**.
Die SVG-Dateien erst nach Überprüfung, Anprobe und ggf. Anpassung
für reale Fertigung verwenden.

Die Browser-Konstruktionsformeln werden in JavaScript umgesetzt; es gibt
parallel eine Rust-Implementierung. Beide sind derzeit **vergleichbare
Prototypen, keine formal nachgewiesen identischen Geometriekerne**.

Nächster Schritt: anatomische Kurven, Mess- und Referenzpunkte,
realistische Grundschnittformeln, Größenkontrolle, Eingriffs-Handles
in der CAD-Grafik und Integration als parametrische SolidCraft-Features.

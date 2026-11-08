# HoodLab v1.3 – öffentliche Müller-&-Sohn-Kapuzenmaße und 3D-Vorschau

## Herkunft und Umfang

Öffentliche Referenz (Redaktion, 4. August 2025):
https://www.muellerundsohn.com/allgemein/schnittkonstruktion-kapuze/

Die Veröffentlichungsseite zeigt und beschreibt eine anliegende und eine
aufgelockerte DOB-Kapuze sowie eine abnäherlose Version. Außerdem erwähnt
sie Kapuzen mit Seitennaht und Mittelstreifen. **Die veröffentlichte
Textbeschreibung reicht nicht aus, um alle Kurven, Übergänge, Abnäherrotationen
und Nahtformungen des originalen Systems exakt nachzubauen.**

**Deshalb enthält diese Umsetzung eine eigenständig gezeichnete Geometrie,
deren kontrollierbare Eckwerte aus den öffentlich zugänglichen Maßbeziehungen
stammen. Weder die originale Verlagsskizze noch Buchtexte oder Abbildungen
werden in das öffentliche GitHub-Projekt kopiert.**

Weitere Kapuzenvarianten sind im Verlags-Dossier dokumentiert:
https://www.muellerundsohn.com/shop/schnitt-technik-kapuzen/

## Öffentliche Zahlenbeziehungen, in eigenen Worten aufgearbeitet

| ID | Relation für die Schnittgrundkonstruktion | Gültigkeit |
|----|------------------------------------------|-------------|
| M01 | Ausgangshöhe setzt sich aus Halsansatzbreite und Vertiefung des vorderen Halslochs zusammen. | Enger Entwurf |
| M02 | Kapuzenhöhe im nahen Bereich ca. 30–34 cm. | Eng |
| M03 | Kapuzentiefe ca. 26–28 cm. | Eng |
| M04 | Ein Bezugspunkt wird um 1,5 cm versetzt. | Eng |
| M05 | Als hinterer Halsloch-Bezug wird 0,5 cm weniger als die Rückenhalsslänge verwendet. | Eng |
| M06 | Für eine Falte bzw. einen Abnäher stehen 3–4 cm zur Verfügung. | Eng |
| M07 | Zwei weitere konstruktive Verschiebungen betragen jeweils 2 cm. | Eng |
| M08 | Vordere Kapuzenansatzstrecke orientiert sich an der Vorderhalsslänge. | Eng |
| M09 | Lockere Variante erhöht eine Anfangsstrecke um 4–5 cm; Höhenbereich 30–36 cm. | Locker |
| M10 | Abnäherlose Kapuze: 35 cm Höhe, 31 cm Tiefe; vorderer Übertritt 2 cm, eigene Halslochvergrößerungen. | Abnäherlos |

Der Code enthält die Schlüssel unter
\`TextilLabHoodSystem.sourceRelations\`. Die Formeln sind paraphrasierte
einzelne Sachbeziehungen, keine Abschrift ganzer Werkseiten.

Die auf der Website genannten originalen Punktnummern sind nicht
gleichbedeutend mit den **K1...K10**-Punktetiketten im TextilLab-SVG;
letztere sind **eigene konstruktive Kontrollreferenzen**.

## 2D-Maßfelder

- Kopfumfang, vordere/hintere Halsansatzlänge, Halsspiegelbreite,
  vordere Halslochvertiefung
- Kapuzenhöhe und -tiefe
- Abnäherreserve, Gesichtsübertritt
- Damen / DOB, Herren / HAKA, Kinder als **getrennte Auswahlprofile**
- Eng, locker, abnäherlos als Modellentwurf
- Nahtverfahren: exakt passend, gezielt dehnen, gezielt einhalten
- zulässige Verarbeitungsdifferenz
- allgemeine Nahtzugabe und Flächengewicht

**Wichtig:** Herren- und Kinder-Schnitte werden in diesem Stand nicht
durch ein belegtes HAKA-/Kinder-Grundschnittsystem berechnet. Ihre
Profile sind ausdrücklich als experimentell gekennzeichnet. Die
Konstruktion skaliert keine angeblichen Männer-/Frauen-/Kinderformen
auf Basis unbewiesener Pauschalannahmen.

## Nahtlängenprüfung

Die tatsächliche Kurvenlänge der Halsansatzsegmente wird numerisch
gemessen. Eine Bisektionssuche dimensioniert beide Kapuzenansatzsegmente
unabhängig voneinander, bis sie die Eingabewerte für vorderes bzw.
hinteres Kleidungs-Halsloch erreichen. Dabei wird die öffentlich
erwähnte hintere Reduktion um 0,5 cm als **Referenzwert** dokumentiert
und für die Standardverbindung \`match\` durch eine eigene Korrektur
wieder ausgeglichen.

- \`match\`: Differenz höchstens 0,3 mm
- \`stretch\`: Kapuzennaht darf nur kürzer sein und muss innerhalb einer
  explizit zulässigen Dehnung liegen
- \`ease\`: Kapuzennaht darf nur länger sein und muss innerhalb einer
  explizit zulässigen Einhalteweite liegen

Die gegenüberliegenden Kapuzenmittelnahtstücke entstehen gespiegelt
und sind daher gleich lang. **Dies ist nur eine geometrische Prüfung**:
das tatsächliche Halsloch des Kleidungsstücks muss separat gemessen
und als Parameter eingetragen werden. Nahtlänge allein beweist keine
korrekte Einsetzkurve, gute Passform oder Einhalteverteilung.

## Schematische 3D-Vorschau

Für **alle vorhandenen Kleidungsgrundkategorien** wurde eine einfache
Volumenansicht ergänzt, mit einem deutlich detaillierteren
Kapuzenmodell. Die 3D-Ansicht erlaubt:

- Maus-/Touch-Drehen sowie Mausrad-Zoom
- Zurücksetzen der Perspektive
- Drahtgitter-Ansicht (Ein/Aus)
- Kopf-Referenzvolumen (Ein/Aus)
- OBJ-Export der eigenen geometrischen Modellflächen
- Aktualisierung bei Änderungen an Kapuzenhöhe/-tiefe und Kopfumfang

Die 3D-Kapuze besteht aus einem parametrisierten dreidimensionalen
Oberflächen-Mesh, das eine Kopf-Referenz umschließt. Es werden
keine physikalischen Stoffeigenschaften, Materialspannung, Falten,
Kollisionen oder nähtechnische Drapierungsregeln simuliert.
**Sie ist kein digitaler Passformnachweis und das 3D-Mesh entspricht
noch nicht geometrisch exakt dem abgewickelten 2D-Schnittteil.**

## Grenzen für eine fachlich gesicherte Weiterentwicklung

- Originale Konstruktionszeichnungen und vollständig beschriebene
  Punkte aus konkreter DOB-/HAKA-/Kinderbuchausgabe müssen geprüft werden.
- Kinderkapuzen benötigen zusätzliche Sicherheitskontrolle auf
  Sichtfeld und Kordel-/Zugbandrisiken.
- Gesichtsöffnung, Abnäherrotation und Nahtzugaben noch nicht
  fertigungsgeprüft.
- Die Flächen-/Gewichtsschätzung verwendet eine Umfangsformel für
  die zusätzliche Nahtzugabefläche, **keinen robust erzeugten äußeren
  Offset**. Im SVG wird ausdrücklich keine fertige Zuschnittkante
  behauptet.
- Ein Produktivstatus darf erst gesetzt werden, wenn Nahtpaarungen,
  Kopfpassform, Stoffverhalten und fachliches Regelsystem validiert sind.

## Testverfahren

\`node --test docs/textillab/hood-and-3d.test.js\`

Die Tests überprüfen:
- nachvollziehbare öffentliche Maßbeziehungen mit Quellen-ID
- getrennte vordere/hintere Halsansatzlängen mit numerischem Abgleich
- Richtungsregeln für Einhalten und Dehnen
- Unterschiede Damen / Herren / Kinder, einschließlich Warnungen
- Fehlerbehandlung bei unplausiblen Körpermaßen
- Stoffflächen-/Gewichtskalkulation
- deterministische 3D-Volumenmodelle mit endlichen Koordinaten
- Kopfvolumen und OBJ-Ausgabe
- Orbit, Zoom, Drahtgitter in einem simulierten Canvas-Test
- Anwesenheit aller UI-Steuerelemente

Alle übrigen Grundschnitt- und CAD-Module bleiben erhalten.

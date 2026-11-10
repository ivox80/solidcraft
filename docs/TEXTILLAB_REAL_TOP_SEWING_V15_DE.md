# TextilLab v1.5 – paarweise vernähte Oberteile mit Stoffdrapierung

## Behobener grundlegender Modellierungsfehler

Bis v1.4 zeigte die 3D-Oberteilvorschau ein Torsorohr und zwei
unabhängige Ärmelrohre, die keinerlei Abhängigkeit von den echten
2D-Nähkanten der Schnittteile hatten. Mit diesen Körpern ließ sich
weder die Passform noch das Einnähen kontrollieren.

Die neue Oberteil-3D-Baugruppe entsteht direkt aus dem vorhandenen
Bodice-V2-Schnitt. Die Schnittkonturen werden entlang ihrer
benannten Konstruktionssegmente abgetastet, trianguliert und vor
dem Drapieren anhand **expliziter Kantenpaare** in 3D verbunden.

Es entstehen vier gespiegelte Rumpfhälften
(VT rechts/links, RT rechts/links) und zwei Ärmel.

## Nähpartner

| Paarung | Schnittpartner | Verarbeitung |
|---|---|---|
| Schulter links/rechts | VT-Schulter ↔ RT-Schulter | Länge passend |
| Seitennaht links/rechts | VT-Seitennaht ↔ RT-Seitennaht | Länge passend |
| Ärmelkugel vorn links/rechts | Ärmelkugel Vorderabschnitt ↔ VT-Armloch | Geplant einhalten |
| Ärmelkugel hinten links/rechts | Ärmelkugel Rückenabschnitt ↔ RT-Armloch | Geplant einhalten |
| Unterarm links/rechts | Ärmel-Vordernaht ↔ Ärmel-Rückennaht | Länge passend |
| Stoffbruch (keine Nähnaht) | linke ↔ rechte Hälfte | Spiegelpunkte teilen |

Damit existieren zehn tatsächliche Paare von zu verbindenden Nähkanten
und zwei zusammengeführte Stoffbruchlinien.

Jedes Paar referenziert zwei eigenständige und **wirklich berechnete
2D-Nähkanten**. Die gesampelten Punkte auf beiden Seiten
werden in eine gemeinsame 3D-Vertex-Topologie überführt, sodass
der geometrische Nahtspalt gleich null ist. Die ursprünglichen
Ruhelängen beider Stoffteile bleiben dabei separat als physikalische
Federconstraints erhalten.

### Ärmelkugel nicht nur insgesamt anpassen

Eine einzige Ziel-Gesamtlänge konnte bisher die Einhalteweite
falsch verteilen: beispielsweise ein zu kurzer vorderer und
ein zu langer hinterer Ärmelkugelabschnitt. TextilLab löst nun
numerisch **zwei unabhängige Teilziele**:

- Vorderer Abschnitt = vorderes Armloch + 40 % der geplanten Mehrweite
- Hinterer Abschnitt = hinteres Armloch + 60 % der geplanten Mehrweite

Bei 22 mm geplanter Ärmel-Mehrweite ergeben sich etwa 8,8 mm vorn
und 13,2 mm hinten (je Ärmel). Das ist eine **eigene
Konstruktionsregel**, kein beanspruchter Müller-&-Sohn-Grundschnitt.
Die kappe wird über Höhe und exzentrische Position ihres
Scheitelpunktes angepasst. Für die Unterarmnähte bleibt die
tatsächlich berechnete Länge gleich.

Geplantes Einhalten wird als **nicht fertig ausgeführte
Verarbeitungsoperation** angezeigt; eine nur geometrisch geschlossene
Kante beweist nicht, dass der Stoff sich korrekt einhalten lässt.
Material, Kerben und Einhaltezonen müssen am realen Kleidungsstück
geprüft werden.

## Parametrische Körper- und Stoffvorschau

Der Stoff liegt als Polygonnetz mit einem vereinfachten
Position-Based-Dynamics-Federverfahren vor. Die Kantenruhelängen
stammen aus den ursprünglichen, flachen Schnittteilen.
Die Schwerkraft, Zugsteifigkeit, Biegesteifigkeit und eine
angenäherte Kollision mit Körper/Armen beeinflussen die Vorschau.

Der gemeinsame Bereich „3D-Anprobe: Körper und Stoff“ funktioniert
jetzt sowohl bei der Kapuze als auch beim Oberteil. Es gibt:

- unterschiedliche Beispiel-Maßpuppen für Damen, Herren und Kinder
- gekoppelte Oberteil-/Körpermaße oder unabhängige Mannequin-Maße
- Baumwolle, Jersey, Denim, Softshell als illustrative Stoff-Presets
- Live-Einstellung von Stoffsteifigkeit, Drapierung und Körperabstand
- transparente Nahtliste mit Paarung und Längendifferenz in mm
- OBJ-Export der verknüpften 3D-Netzgeometrie

Der 3D-Körper ist weiterhin eine **vereinfachte parametrische Puppe**,
kein validierter anatomischer Avatar. Profile ändern zunächst
Körpermaße, noch nicht die authentischen Schnittkonstruktionsregeln
einer DOB-, HAKA- oder Kinder-Fachausgabe.

## Offene Kanten und wichtige Grenzen

- Halsausschnitt, Saum und Ärmelabschluss bleiben bewusst offen.
- Im ärmellosen Modus bleiben beide Armlöcher frei; es werden
  **keine fiktiven Ärmel erzeugt**.
- Schulter-/Brust- und Taillenabnäher sind noch Markierungen. Sie
  müssen vor einer fertigungsfähigen Konstruktion geometrisch
  geschlossen und die Nählinien entsprechend neu berechnet werden.
- Die virtuelle Einhalteweite ist zwar numerisch zugeordnet, aber
  noch nicht mit Knipsen und Material-Tests abgenommen.
- Für das Stoffnetz fehlen validierte anisotrope Kett-/Schuss-Steifigkeit,
  Scherverhalten, Materialdicke, Reibung, plastische Verformung,
  echte Faltauflösung und eine professionelle Kontaktbehandlung.
- Numerische Kantenverzüge können groß werden. Diese werden
  **nicht** als korrekt kaschiert. Der Bericht meldet die
  Verformungskennzahlen und keine Fertigungsfreigabe.

Das Modell ist somit ein deutlich besserer **technischer
Nähbaugruppen-Prototyp**, aber noch keine mit CLO, Browzwear
oder professionellem FEM-/XPBD-Stoffsolver gleichwertige Anprobe.

## Quellcode und Tests

- docs/textillab/top-sewing.js – Nähpartner, Vernetzung, Vertex-Weld,
  Körperkollision und einfache Stoffdynamik
- docs/textillab/bodice-v2.js – parametrische 2D-Schnittkurven und
  variabler Ärmelkugelscheitelpunkt
- docs/textillab/drafting-system.js – separater numerischer Abgleich
  vordere/hintere Ärmelkugel
- docs/textillab/garment-3d.js – 3D-Darstellung und OBJ
- docs/textillab/app.js – Oberteil-Nähbericht und gemeinsame
  Körper-/Stoffeingabe
- docs/textillab/top-sewing.test.js – Nähkantenpaarung,
  Verzug, Stoffverhalten, ärmellose Variante, UI, OBJ und Grundschnitt

Browser: https://ivox80.github.io/solidcraft/textillab/

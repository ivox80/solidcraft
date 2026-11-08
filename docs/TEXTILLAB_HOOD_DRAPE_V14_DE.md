# TextilLab v1.4: Schnittgebundene Kapuzen-3D-Anprobe

## Was sich tatsächlich geändert hat

Bis v1.3 wurde die 3D-Kapuze unabhängig von der Schnittkontur als
Ringkörper modelliert. Dadurch existierte kein geometrischer Nachweis,
dass beide Kapuzenhälften entsprechend dem 2D-Schnitt verbunden sind.

Version 1.4 baut zwei 3D-Seitenflächen **aus den drei gesampelten
Kanten der tatsächlichen 2D-Schnittzeichnung**:

- Mittlere Nahtkurve (vom Nacken über die obere Kontur nach vorn)
- Offene Gesichtsöffnung
- Halsloch-Ansatzlinie des Kapuzenteils

Die Flächen werden mit einem transfinitem Coons-Patch aufgebaut.
Linke und rechte Kapuzenhälfte verwenden für die Mittelnaht dieselben
Vertexindizes. Der geometrische Mittelnahtspalt ist deshalb exakt 0 mm.
Die Halsloch-Ansatzlinie bleibt dagegen bewusst offen: Es ist noch
kein real konstruierter Jacken-Halsausschnitt als Nähpartner vorhanden.

## Parametrische Maßpuppe

Der Browser erstellt jetzt einen Körper aus Kopf, Hals, Schultern,
Torso und optional dargestellten Armen, Händen, Beinen und Füßen.
Die Maße können unabhängig geändert werden:

- Körperhöhe
- Schulterbreite
- Brustumfang, Taillenumfang, Hüftumfang
- Halsumfang
- Kopfumfang aus dem eigentlichen Kapuzen-Grundschnitt

Damen-, Herren- und Kinderprofile enthalten unterschiedliche
Beispielwerte. Das ist **keine** validierte antropometrische
Grundgrößentabelle oder gescannte, anatomisch genaue 3D-Menschengestalt.

Die Schaltfläche Ganzkörper stellt zusätzliche Körperbereiche dar.
Standardmäßig bleibt der sichtbare Bereich auf Kopf und Schultern
ausgerichtet, damit die Nahtkonturen kontrollierbar sind.

## Stoffdrapierung – experimentelles numerisches Modell

Der Stoff besteht aus den triangulierten Schnittteilen. Die Federn
erhalten ihre Ruhelängen aus der 2D-Schnittfläche. Ein begrenztes
Position-Based-Dynamics-Verfahren verschiebt die freie Oberfläche und
reagiert auf:

- Zugsteifigkeit
- Biegesteifigkeit
- Schwerkraftparameter
- Kopfkontakt und Abstand (Clearance)

Baumwolle, Jersey, Denim und Softshell besitzen unterschiedliche
**illustrative** Startparameter, keine gemessenen Materialkennwerte.

Die 3D-Anprobe zeigt den Mittelnahtspalt in Millimeter,
den RMS-Kantenverzug, den maximalen Kantenverzug und die Anzahl der
Kollisionskorrekturen. Hohe Verzüge werden ausdrücklich als
**Passform nicht freigegeben** angezeigt.

## Einschränkungen und weitere fachliche Aufgaben

Die Mittelnaht ist topologisch verbunden – das beweist noch nicht,
dass der Stoff eine plausible oder tragbare Kapuze ergibt.
Nähende des Halsteils, reale Abnäher, Einhalteverteilung, Fadenlauf,
Flächenmasse, Reibung, Falteffekte und anisotrope Stoffeigenschaften
sind nicht physikalisch validiert. Ferner kann das vereinfachte Modell
lokale Kantenverzüge und Durchdringungen nicht mit vollständiger
Stabilitätsgarantie verhindern.

Eine vollständige Lösung erfordert:
1. Gegennaht mit dem tatsächlichen Oberteil-/Jacken-Halsloch verbinden.
2. Kerben, Einschnittpunkte und geschlossene Abnäher als harte
   Naht-Zwangsbedingungen definieren.
3. Meshing und Cloth-Solver mit materialkalibrierten
   anisotropen Scher-, Dehn- und Biegesteifigkeiten ausbauen.
4. Körpergeometrie gegen reale Messdaten und Anprobe validieren.
5. Kinderbekleidung und gewerblichen Zuschnitt gesondert prüfen.

Die SVG-Zuschnittzeichnung und die OBJ-3D-Ausgabe bleiben ein
**experimenteller Prototyp**, kein fertigungstaugliches Schnittmuster.
Die konkrete Konstruktionsmethode nach Müller & Sohn muss vor einer
fachlichen Freigabe weiterhin anhand der Fachbücher und einer
physischen Musteranprobe validiert werden.

## Tests

Die Datei docs/textillab/hood-drape.test.js deckt ab:
- gemeinsam indizierte Mittelnaht statt separat gelegter Körper
- reale Änderung von Kopfumfang und Körpermaßen
- unterschiedliche Stoffparameter erzeugen unterschiedliche Drapes
- Fehler bei fehlenden/inkonsistenten 2D-Nahtkurven
- 3D-OBJ, Bedienfelder und bisherige 3D-Viewer-Bedienung.

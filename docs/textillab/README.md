# TextilLab Online-Demo

Diese Browser-Demo zeigt die TextilLab-Studio-Oberfläche ohne lokale Installation:
3D-OBJ laden, innere Dreieckskanten anklicken, Trennnähte setzen, Abwicklung
ansehen und SVG-Schnittmuster exportieren.

**Wichtiger Unterschied:** Die Website rechnet im Browser mit einer
JavaScript-Umsetzung des Abwicklungsalgorithmus. Die native SolidCraft-Version
und das lokale Studio verwenden den Rust-Kern. Damit ist die Online-Version
eine eigenständige interaktive Vorschau, kein Rust-WASM-Build und kein
parametrisches SolidCraft-Plugin.

Das OBJ-Modell bleibt im Browser. Es wird nicht an einen Server hochgeladen.

## GitHub Pages aktivieren

Der Administrator des Repositories ivox80/solidcraft öffnet:
https://github.com/ivox80/solidcraft/settings/pages

- Source: Deploy from a branch
- Branch: main
- Folder: /docs
- Save

Danach liegt die Demo typischerweise unter:
https://ivox80.github.io/solidcraft/textillab/

Bis GitHub Pages aktiviert ist, ist die URL nicht garantiert erreichbar.

## Grenzen

Die Website verarbeitet Dreiecke und triangulierte Vierecke sowie
abwickelbare Netze; für komplexe doppelt gekrümmte Oberflächen gibt es
noch keine Dehnungsoptimierung. Nahtzugaben für konkave Konturen,
Überlappungsprüfung, Materialfadenlauf und Fertigungsfreigabe fehlen.

Die Website ist **keine Fertigungsschnittmuster-Software**.

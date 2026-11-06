import * as THREE from 'three';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
import { TransformControls } from 'three/addons/controls/TransformControls.js';
import init, {
  nurbs_panel_wasm, nurbs_cylinder_wasm, nurbs_evaluate_wasm,
  nurbs_tessellate_wasm, nurbs_edit_control_wasm, nurbs_patch_topology_wasm,
} from '../pkg/solidcraft_direct_modeling.js';

// All surface evaluation, control editing and tessellation happen inside Rust/WASM.
// This script only presents the model and forwards direct-manipulation gestures.
const $ = id => document.getElementById(id);
const clone = value => structuredClone(value);
const log = message => { $('log').textContent = message; };
const colors = [0x3faad5, 0x78c5a6, 0xd0a2e6, 0xe6b16c, 0x9cbbef];
let patches = [], active = 0, controlU = 1, controlV = 1;
let preset = 'panel', ready = false, history = [], surfaceMeshes = [], controlMarkers = [];
let surfaceTriangles = 0, surfaceArea = 0, topologyReport = null;

const scene = new THREE.Scene();
scene.background = new THREE.Color(0x101720);
const viewport = $('viewport');
const camera = new THREE.PerspectiveCamera(44, 1, 0.1, 4000);
camera.up.set(0, 0, 1);
camera.position.set(123, -152, 112);
const renderer = new THREE.WebGLRenderer({ antialias: true });
renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
viewport.prepend(renderer.domElement);
const orbit = new OrbitControls(camera, renderer.domElement);
orbit.enableDamping = true;
orbit.target.set(0, 0, 12);
orbit.update();
scene.add(new THREE.HemisphereLight(0xebf4ff, 0x192f42, 2.4));
const sun = new THREE.DirectionalLight(0xffffff, 2.3);
sun.position.set(80, -120, 180);
scene.add(sun);
const grid = new THREE.GridHelper(260, 26, 0x446174, 0x273e51);
grid.rotation.x = Math.PI / 2;
scene.add(grid);
scene.add(new THREE.AxesHelper(38));
const visual = new THREE.Group();
scene.add(visual);
const transform = new TransformControls(camera, renderer.domElement);
transform.setSize(0.72);
scene.add(transform.getHelper());
transform.addEventListener('dragging-changed', event => {
  orbit.enabled = !event.value;
});

function checkpoint() {
  history.push({ patches: clone(patches), preset, active, controlU, controlV });
  if (history.length > 40) history.shift();
}

function setPreset(name) {
  if (ready && patches.length) checkpoint();
  if (name === 'panel') {
    patches = [{ name: 'Freiformfläche', surface: nurbs_panel_wasm() }];
    active = 0;
    controlU = 1; controlV = 1;
  } else {
    patches = nurbs_cylinder_wasm(35, -45, 45).map((surface, i) => ({
      name: 'Zylinder – Viertel ' + (i + 1), surface,
    }));
    active = 0;
    controlU = 1; controlV = 0;
  }
  preset = name;
  redraw();
  log(name === 'panel'
    ? 'Rationale kubische Freiformfläche erstellt (Rust).'
    : 'Vier exakte rationale Viertel-Zylinderflächen erstellt (Rust); Endkappen fehlen.');
}

function selectControl(patchIndex, u, v) {
  active = patchIndex;
  controlU = u;
  controlV = v;
  redraw();
}

function refreshInputs() {
  const patchSelect = $('patch');
  patchSelect.replaceChildren();
  for (let i = 0; i < patches.length; i++) {
    patchSelect.add(new Option(patches[i].name, String(i)));
  }
  patchSelect.value = String(active);
  const surface = patches[active]?.surface;
  const nu = surface?.control_points.length ?? 0;
  const nv = surface?.control_points[0]?.length ?? 0;
  controlU = Math.max(0, Math.min(nu - 1, controlU));
  controlV = Math.max(0, Math.min(nv - 1, controlV));
  for (const [field, count, picked] of [
    ['iu', nu, controlU], ['iv', nv, controlV],
  ]) {
    const element = $(field);
    element.replaceChildren();
    for (let i = 0; i < count; i++) {
      element.add(new Option(String(i), String(i)));
    }
    element.value = String(picked);
  }
  if (surface) {
    const p = surface.control_points[controlU][controlV];
    for (const [field, value] of [['x', p[0]], ['y', p[1]], ['z', p[2]],
      ['w', surface.weights[controlU][controlV]]]) {
      $(field).value = String(Number(value.toFixed(7)));
    }
    $('info').textContent = 'Grad U/V: ' + surface.degree_u + '/' + surface.degree_v +
      '\nKontrollpunkte: ' + nu + ' × ' + nv;
  } else {
    $('info').textContent = 'Keine Fläche vorhanden.';
  }
  $('topology').textContent = topologyReport
    ? ('Patch-Flächen: ' + topologyReport.face_count + '\nVerbunde Nahtkanten: ' + topologyReport.seam_count +
       '\nOffene Ränder: ' + topologyReport.open_boundary_count +
       '\nNichtmanifold-Gruppen: ' + topologyReport.nonmanifold_edge_groups +
       '\nGetrennte Komponenten: ' + topologyReport.connected_components)
    : 'Keine Nahtanalyse verfügbar.';
  $('metrics').textContent = 'Render-Dreiecke: ' + surfaceTriangles.toLocaleString('de-DE') +
    '\nTessellierte Fläche (ca.): ' + surfaceArea.toFixed(1) + ' mm²';
  $('undo').disabled = !ready || history.length === 0;
  $('save').disabled = !ready;
  $('apply').disabled = !ready || !surface;
  $('probe').disabled = !ready || !surface;
}

function disposeChildren() {
  transform.detach();
  for (const child of [...visual.children]) {
    visual.remove(child);
    child.geometry?.dispose();
    child.material?.dispose();
  }
  surfaceMeshes = [];
  controlMarkers = [];
}

function addLine(points, color, opacity = 0.7) {
  const geometry = new THREE.BufferGeometry().setFromPoints(
    points.map(point => new THREE.Vector3(...point))
  );
  const material = new THREE.LineBasicMaterial({ color, transparent: true, opacity });
  const line = new THREE.Line(geometry, material);
  visual.add(line);
}

function surfaceToMesh(surface, color, index, steps) {
  const data = nurbs_tessellate_wasm(surface, steps, steps);
  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute('position', new THREE.Float32BufferAttribute(data.positions.flat(), 3));
  geometry.setAttribute('normal', new THREE.Float32BufferAttribute(data.normals.flat(), 3));
  geometry.setIndex(data.indices);
  const material = new THREE.MeshStandardMaterial({
    color,
    metalness: 0.16,
    roughness: 0.48,
    side: THREE.DoubleSide,
    transparent: true,
    opacity: active === index ? 0.90 : 0.65,
  });
  const mesh = new THREE.Mesh(geometry, material);
  mesh.userData.patch = index;
  visual.add(mesh);
  surfaceMeshes.push(mesh);
  surfaceTriangles += data.indices.length / 3;
  const v = data.positions;
  for (let i = 0; i < data.indices.length; i += 3) {
    const a = new THREE.Vector3(...v[data.indices[i]]);
    const b = new THREE.Vector3(...v[data.indices[i + 1]]);
    const c = new THREE.Vector3(...v[data.indices[i + 2]]);
    surfaceArea += b.sub(a).cross(c.sub(a)).length() * 0.5;
  }
}

function redraw() {
  if (!ready) return;
  disposeChildren();
  const steps = Number($('resolution').value);
  surfaceTriangles = 0;
  surfaceArea = 0;
  for (let k = 0; k < patches.length; k++) {
    const surface = patches[k].surface;
    surfaceToMesh(surface, colors[k % colors.length], k, steps);
    if (!$('lattice').checked) continue;
    const rows = surface.control_points;
    for (const row of rows) addLine(row, 0x83c9ef, 0.7);
    for (let j = 0; j < rows[0].length; j++) {
      addLine(rows.map(row => row[j]), 0x83c9ef, 0.7);
    }
    for (let i = 0; i < rows.length; i++) {
      for (let j = 0; j < rows[i].length; j++) {
        const isActive = k === active && i === controlU && j === controlV;
        const marker = new THREE.Mesh(
          new THREE.SphereGeometry(isActive ? 2.0 : 1.45, 12, 9),
          new THREE.MeshStandardMaterial({
            color: isActive ? 0xffc968 : 0x78d3ff,
            roughness: 0.25,
            depthTest: true,
          })
        );
        marker.position.set(...rows[i][j]);
        marker.userData = { patch: k, u: i, v: j };
        visual.add(marker);
        controlMarkers.push(marker);
      }
    }
  }
  // The seam analyzer detects sampled G0 adjacency, not closed CAD solids.
  try {
    topologyReport = nurbs_patch_topology_wasm(patches.map(p => p.surface), 1e-6);
    if ($('edges').checked) {
      for (const open of topologyReport.open_boundaries) {
        const surface = patches[open.face].surface;
        const ud = [surface.knots_u[surface.degree_u],
          surface.knots_u[surface.control_points.length]];
        const vd = [surface.knots_v[surface.degree_v],
          surface.knots_v[surface.control_points[0].length]];
        const pts = [];
        for (let i = 0; i <= 20; i++) {
          const t = i / 20;
          const u = open.edge === 'u_min' ? ud[0] :
            open.edge === 'u_max' ? ud[1] : ud[0] + (ud[1] - ud[0]) * t;
          const v = open.edge === 'v_min' ? vd[0] :
            open.edge === 'v_max' ? vd[1] : vd[0] + (vd[1] - vd[0]) * t;
          pts.push(nurbs_evaluate_wasm(surface, u, v));
        }
        addLine(pts, 0xff996e, 0.98);
      }
    }
  } catch (e) {
    topologyReport = null;
    log('Nahtanalyse fehlgeschlagen: ' + e);
  }
  const activeMarker = controlMarkers.find(marker =>
    marker.userData.patch === active &&
    marker.userData.u === controlU && marker.userData.v === controlV);
  if (activeMarker) transform.attach(activeMarker);
  $('resolutionValue').textContent = steps + ' × ' + steps;
  refreshInputs();
}

function applyPoint(point, weight, label = 'Kontrollpunkt verändert') {
  const patch = patches[active];
  if (!patch) return;
  const old = patch.surface.control_points[controlU][controlV];
  const oldWeight = patch.surface.weights[controlU][controlV];
  if (point.every((value, i) => value === old[i]) && weight === oldWeight) {
    redraw();
    return;
  }
  try {
    const newSurface = nurbs_edit_control_wasm(
      patch.surface, controlU, controlV, point, weight
    );
    // Ensure the edited patch still has a displayable surface.
    nurbs_tessellate_wasm(newSurface, 4, 4);
    checkpoint();
    patch.surface = newSurface;
    redraw();
    log(label + ': U' + controlU + '/V' + controlV + ' (' +
      point.map(p => p.toFixed(2)).join(', ') + '), Gewicht ' + weight.toFixed(3));
  } catch (err) {
    redraw();
    log('Änderung abgelehnt: ' + err);
  }
}

transform.addEventListener('mouseUp', () => {
  if (!ready || !transform.object) return;
  const obj = transform.object;
  const { patch, u, v } = obj.userData;
  if (patch !== active || u !== controlU || v !== controlV) return;
  const p = obj.position.toArray();
  const weight = patches[active].surface.weights[controlU][controlV];
  applyPoint(p, weight, '3D-Gizmo verschoben');
});

$('panel').onclick = () => setPreset('panel');
$('cylinder').onclick = () => setPreset('cylinder');
$('patch').onchange = () => {
  active = Number($('patch').value);
  const grid = patches[active].surface.control_points;
  controlU = Math.floor(grid.length / 2);
  controlV = Math.floor(grid[0].length / 2);
  redraw();
};
$('iu').onchange = () => selectControl(active, Number($('iu').value), controlV);
$('iv').onchange = () => selectControl(active, controlU, Number($('iv').value));
$('lattice').onchange = redraw;
$('edges').onchange = redraw;
$('resolution').oninput = () => {
  $('resolutionValue').textContent = $('resolution').value + ' × ' + $('resolution').value;
  redraw();
};
$('apply').onclick = () => {
  const p = [Number($('x').value), Number($('y').value), Number($('z').value)];
  const w = Number($('w').value);
  if (!p.every(Number.isFinite) || !Number.isFinite(w)) {
    return log('Bitte gültige Zahlen eingeben.');
  }
  applyPoint(p, w, 'Numerische Bearbeitung');
};
$('undo').onclick = () => {
  if (!history.length) return;
  const state = history.pop();
  patches = state.patches;
  preset = state.preset;
  active = state.active;
  controlU = state.controlU;
  controlV = state.controlV;
  redraw();
  log('Letzte Geometrieänderung rückgängig gemacht.');
};
$('probe').onclick = () => {
  try {
    const u = Number($('u').value);
    const v = Number($('v').value);
    const point = nurbs_evaluate_wasm(patches[active].surface, u, v);
    const radius = Math.hypot(point[0], point[1]);
    $('probeOut').textContent =
      'X: ' + point[0].toFixed(6) + ' mm\n' +
      'Y: ' + point[1].toFixed(6) + ' mm\n' +
      'Z: ' + point[2].toFixed(6) + ' mm' +
      (preset === 'cylinder' ? '\nRadius aus X/Y: ' + radius.toFixed(6) + ' mm' : '');
  } catch (err) {
    $('probeOut').textContent = 'Auswertung fehlgeschlagen: ' + err;
  }
};
$('save').onclick = () => {
  const body = JSON.stringify({ format: 'solidcraft-nurbs-v04', preset, patches }, null, 2);
  const url = URL.createObjectURL(new Blob([body], { type: 'application/json' }));
  const a = document.createElement('a');
  a.href = url; a.download = 'solidcraft-nurbs-v04.json'; a.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
};
$('open').onclick = () => $('file').click();
$('file').onchange = async event => {
  const file = event.target.files?.[0];
  if (!file) return;
  try {
    if (file.size > 3_000_000) throw Error('Datei ist zu groß.');
    const obj = JSON.parse(await file.text());
    if (obj.format !== 'solidcraft-nurbs-v04' ||
      !Array.isArray(obj.patches) || obj.patches.length === 0 ||
      obj.patches.length > 16) throw Error('Unbekanntes Dateiformat.');
    for (const patch of obj.patches) {
      if (typeof patch.name !== 'string' || patch.name.length > 100)
        throw Error('Ungültiger Flächenname.');
      nurbs_tessellate_wasm(patch.surface, 4, 4); // Rust validation
    }
    checkpoint();
    patches = obj.patches;
    preset = obj.preset === 'cylinder' ? 'cylinder' : 'panel';
    active = 0; controlU = 0; controlV = 0;
    redraw(); log(obj.patches.length + ' NURBS-Flächen aus JSON geladen.');
  } catch (err) {
    log('Import fehlgeschlagen: ' + err);
  } finally { $('file').value = ''; }
};

const picker = new THREE.Raycaster();
const pointer = new THREE.Vector2();
let down = null;
renderer.domElement.addEventListener('pointerdown', event => {
  down = [event.clientX, event.clientY];
});
renderer.domElement.addEventListener('pointerup', event => {
  if (!down || Math.hypot(event.clientX - down[0], event.clientY - down[1]) > 4) {
    down = null; return;
  }
  down = null;
  const rect = renderer.domElement.getBoundingClientRect();
  pointer.x = (event.clientX - rect.left) / rect.width * 2 - 1;
  pointer.y = -(event.clientY - rect.top) / rect.height * 2 + 1;
  picker.setFromCamera(pointer, camera);
  // Prioritize control point selection instead of selecting underlying surface.
  const hit = picker.intersectObjects(controlMarkers, false)[0];
  if (hit) {
    const { patch, u, v } = hit.object.userData;
    selectControl(patch, u, v);
    return;
  }
  const surfaceHit = picker.intersectObjects(surfaceMeshes, false)[0];
  if (surfaceHit && active !== surfaceHit.object.userData.patch) {
    active = surfaceHit.object.userData.patch;
    controlU = 0; controlV = 0;
    redraw();
  }
});

function resize() {
  const rect = viewport.getBoundingClientRect();
  camera.aspect = rect.width / Math.max(1, rect.height);
  camera.updateProjectionMatrix();
  renderer.setSize(rect.width, Math.max(1, rect.height));
}
new ResizeObserver(resize).observe(viewport);
function animate() {
  requestAnimationFrame(animate);
  orbit.update();
  renderer.render(scene, camera);
}
animate();resize();

try {
  await init();
  ready = true;
  $('status').textContent = '● Rust NURBS aktiv';
  $('status').className = 'ok';
  setPreset('panel');
} catch (err) {
  $('status').textContent = '● WebAssembly nicht geladen';
  $('status').className = 'error';
  log('NURBS-WASM konnte nicht geladen werden: ' + err +
    '\nZum Testen: wasm-pack build --target web --features wasm und lokalen HTTP-Server starten.');
}

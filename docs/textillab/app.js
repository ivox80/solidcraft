"use strict";

// Client-side online preview: mesh unfolding uses browser-engine.js; garment blocks use pattern-library.js.
const $ = (id) => document.getElementById(id);
const canvas = $("modelCanvas");
const ctx = canvas.getContext("2d");
const state = {
  source: "", name: "", vertices: [], faces: [], edges: [],
  seams: new Set(), selectedFace: -1, hoveredEdge: null,
  yaw: -0.48, pitch: 0.42, zoom: 1.0, projected: [],
  pointer: null, currentSvg: "", pending: null, revision: 0,
};

function setStatus(message, bad = false) {
  $("status").textContent = message;
  document.querySelector(".statusdot").style.background = bad ? "#f58c86" : "#4ac7b0";
}
function clearStats() {
  ["pieces", "sewnArea", "cutArea", "mass"].forEach((id) => $(id).textContent = "–");
  $("exportBtn").disabled = true;
}
function parseObj(source) {
  const vertices = [], faces = [];
  const lines = source.split(/\r?\n/);
  for (let lineNo = 0; lineNo < lines.length; lineNo++) {
    const words = lines[lineNo].split("#")[0].trim().split(/\s+/);
    if (words[0] === "v") {
      const p = words.slice(1, 4).map(Number);
      if (p.length < 3 || !p.every(Number.isFinite))
        throw new Error("Ungültige Koordinate in Zeile " + (lineNo + 1));
      vertices.push(p);
    } else if (words[0] === "f") {
      if (words.length < 4 || words.length > 5)
        throw new Error("Nur Dreiecke und Vierecke unterstützt (Zeile " + (lineNo + 1) + ")");
      const f = words.slice(1).map((token) => {
        const raw = Number(token.split("/")[0]);
        const i = raw > 0 ? raw - 1 : vertices.length + raw;
        if (!Number.isInteger(raw) || raw === 0 || i < 0 || i >= vertices.length)
          throw new Error("Ungültiger Vertexindex in Zeile " + (lineNo + 1));
        return i;
      });
      faces.push([f[0], f[1], f[2]]);
      if (f.length === 4) faces.push([f[0], f[2], f[3]]);
    }
  }
  if (!faces.length) throw new Error("OBJ enthält keine Dreiecksflächen");
  if (faces.length > 10000) throw new Error("Maximal 10.000 Dreiecke in der Studio-Vorschau");
  const edges = new Map();
  faces.forEach((face, fi) => {
    for (let i = 0; i < 3; i++) {
      const a = Math.min(face[i], face[(i + 1) % 3]);
      const b = Math.max(face[i], face[(i + 1) % 3]);
      const key = a + ":" + b;
      if (!edges.has(key)) edges.set(key, { a, b, faces: [] });
      edges.get(key).faces.push(fi);
    }
  });
  return { vertices, faces, edges: [...edges.values()].sort((x, y) => x.a - y.a || x.b - y.b) };
}
function seamKey(edge) {
  return (edge.a + 1) + ":" + (edge.b + 1);
}
async function loadModel(source, name) {
  const parsed = parseObj(source);
  state.source = source;
  state.name = name;
  Object.assign(state, parsed);
  state.seams.clear();
  state.selectedFace = -1;
  state.hoveredEdge = null;
  resetCamera();
  $("fileName").textContent = name;
  $("meshStats").textContent = state.vertices.length + " Punkte · " + state.faces.length + " Dreiecke";
  updateSeamList();
  await updatePreview();
}
function resetCamera() {
  state.yaw = -0.48;
  state.pitch = 0.42;
  state.zoom = 1;
  draw();
}
function cameraPoints() {
  if (!state.vertices.length) return [];
  const center = [0, 0, 0];
  for (const p of state.vertices) for (let i = 0; i < 3; i++) center[i] += p[i];
  for (let i = 0; i < 3; i++) center[i] /= state.vertices.length;
  const cy = Math.cos(state.yaw), sy = Math.sin(state.yaw);
  const cp = Math.cos(state.pitch), sp = Math.sin(state.pitch);
  return state.vertices.map(([px, py, pz]) => {
    const x = px - center[0], y = py - center[1], z = pz - center[2];
    const rx = cy * x - sy * y;
    const ry = sy * x + cy * y;
    return [rx, -cp * ry + sp * z, sp * ry + cp * z];
  });
}
function projectVertices(w, h) {
  const p = cameraPoints();
  if (!p.length) return [];
  const minx = Math.min(...p.map(v => v[0]));
  const maxx = Math.max(...p.map(v => v[0]));
  const miny = Math.min(...p.map(v => v[1]));
  const maxy = Math.max(...p.map(v => v[1]));
  const spanx = Math.max(1, maxx - minx);
  const spany = Math.max(1, maxy - miny);
  const scale = Math.min(w / (spanx * 1.5), h / (spany * 1.5)) * state.zoom;
  const cx = (minx + maxx) / 2, cy = (miny + maxy) / 2;
  return p.map(([x, y, z]) => ({ x: w / 2 + (x - cx) * scale, y: h / 2 + (y - cy) * scale, z }));
}
function draw() {
  const box = canvas.getBoundingClientRect();
  const w = box.width, h = box.height;
  if (w <= 0 || h <= 0) return;
  const dpr = Math.min(window.devicePixelRatio || 1, 2);
  canvas.width = Math.max(1, Math.round(w * dpr));
  canvas.height = Math.max(1, Math.round(h * dpr));
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.clearRect(0, 0, w, h);
  ctx.lineWidth = 1;
  ctx.strokeStyle = "#203449";
  ctx.setLineDash([2, 7]);
  for (let x = w / 2 % 38; x < w; x += 38) {
    ctx.beginPath(); ctx.moveTo(x, 0); ctx.lineTo(x, h); ctx.stroke();
  }
  for (let y = h / 2 % 38; y < h; y += 38) {
    ctx.beginPath(); ctx.moveTo(0, y); ctx.lineTo(w, y); ctx.stroke();
  }
  ctx.setLineDash([]);
  state.projected = projectVertices(w, h);
  const ordered = state.faces.map((f, i) => ({
    face: f, index: i, depth: f.reduce((s, v) => s + state.projected[v].z, 0) / 3,
  })).sort((a, b) => a.depth - b.depth);
  for (const { face, index } of ordered) {
    ctx.beginPath();
    face.forEach((v, i) => {
      const p = state.projected[v];
      if (i === 0) ctx.moveTo(p.x, p.y);
      else ctx.lineTo(p.x, p.y);
    });
    ctx.closePath();
    ctx.fillStyle = index === state.selectedFace ? "#396784d4" : "#407e9f33";
    ctx.fill();
    ctx.lineWidth = 0.55;
    ctx.strokeStyle = "#507b9860";
    ctx.stroke();
  }
  for (const edge of state.edges) {
    const a = state.projected[edge.a], b = state.projected[edge.b];
    const selected = state.seams.has(seamKey(edge));
    const hovered = edge === state.hoveredEdge;
    ctx.strokeStyle = selected ? "#f39184" : hovered ? "#f1d794" : edge.faces.length === 2 ? "#80c6d9" : "#5f879e";
    ctx.lineWidth = selected ? 3.6 : hovered ? 3 : edge.faces.length === 2 ? 1.8 : 1.25;
    ctx.beginPath(); ctx.moveTo(a.x, a.y); ctx.lineTo(b.x, b.y); ctx.stroke();
    if (selected) {
      ctx.beginPath(); ctx.arc((a.x + b.x) / 2, (a.y + b.y) / 2, 3.5, 0, Math.PI * 2);
      ctx.fillStyle = "#ffb0a0"; ctx.fill();
    }
  }
  if (state.hoveredEdge) {
    const edge = state.hoveredEdge;
    const a = state.projected[edge.a], b = state.projected[edge.b];
    const x = (a.x + b.x) / 2, y = (a.y + b.y) / 2;
    const label = seamKey(edge) + (edge.faces.length === 2 ? " · Klick: Schnitt" : " · Rand");
    ctx.font = "12px Segoe UI, sans-serif";
    const tw = ctx.measureText(label).width;
    const lx = Math.min(w - tw - 16, Math.max(8, x + 12));
    const ly = Math.max(24, y - 14);
    ctx.fillStyle = "#17212df0"; ctx.fillRect(lx - 6, ly - 17, tw + 12, 24);
    ctx.fillStyle = "#f2e8d3"; ctx.fillText(label, lx, ly);
  }
}
function edgeDistance(p, a, b) {
  const dx = b.x - a.x, dy = b.y - a.y;
  const k = Math.max(0, Math.min(1, ((p.x - a.x) * dx + (p.y - a.y) * dy) / Math.max(1e-9, dx * dx + dy * dy)));
  return Math.hypot(p.x - a.x - k * dx, p.y - a.y - k * dy);
}
function pickEdge(p) {
  let match = null, distance = 12;
  for (const edge of state.edges) {
    const d = edgeDistance(p, state.projected[edge.a], state.projected[edge.b]);
    if (d < distance) { match = edge; distance = d; }
  }
  return match;
}
function pointInside(p, a, b, c) {
  const sign = (p1, p2, p3) => (p1.x - p3.x) * (p2.y - p3.y) - (p2.x - p3.x) * (p1.y - p3.y);
  const d1 = sign(p, a, b), d2 = sign(p, b, c), d3 = sign(p, c, a);
  return !((d1 < 0 || d2 < 0 || d3 < 0) && (d1 > 0 || d2 > 0 || d3 > 0));
}
function pickFace(p) {
  for (let i = state.faces.length - 1; i >= 0; i--) {
    const face = state.faces[i];
    if (pointInside(p, ...face.map(v => state.projected[v]))) return i;
  }
  return -1;
}
function updateSeamList() {
  $("seamCount").textContent = state.seams.size;
  const list = $("seamList");
  list.replaceChildren();
  if (!state.seams.size) {
    const msg = document.createElement("span");
    msg.className = "muted"; msg.textContent = "Noch keine";
    list.appendChild(msg);
    return;
  }
  for (const key of [...state.seams].sort()) {
    const button = document.createElement("button");
    button.className = "seamitem";
    button.title = "Schnittkante entfernen";
    button.textContent = key + " ×";
    button.addEventListener("click", () => {
      state.seams.delete(key); updateSeamList(); draw(); updatePreview();
    });
    list.appendChild(button);
  }
}

function catalogMeasurements() {
  const result = {};
  for (const name of Object.keys(TextilLabPatternLibrary.defaults)) {
    result[name] = $("m-" + name).value;
  }
  for (const name of Object.keys(TextilLabBodiceV2.defaults)) {
    result[name] = $("m-" + name).value;
  }
  for (const name of Object.keys(TextilLabPantsSystem.defaults)) {
    // Shared body values live in the general panel; pants-only waistToHip
    // intentionally has a separate input (avoids duplicate HTML IDs).
    const id=name==="waistToHip"?"m-pantsWaistToHip":"m-"+name;
    if (!Object.hasOwn(result,name) || (name==="waistToHip"&&$("templateType").value==="trousers")) result[name] = $(id).value;
  }
  return result;
}

const history={items:[],index:-1,applying:false};
let latestDraft=null;
function draftOptions() {
  return {
    fit:$("draftFit").value,
    style:$("draftStyle").value,
    targetEaseMm:Number($("draftEase").value),
    allowance:Number($("allowance").value),
    density:Number($("density").value),
    alterations:Object.fromEntries(
      Object.keys(TextilLabDraftingSystem.zones).map(zone=>[zone, Number($("zone-"+zone).value)])),
    layers:Object.fromEntries(
      Object.keys(TextilLabDraftingSystem.layers).map(layer=>[layer, $("layer-"+layer).checked]))
  };
}
function draftSnapshot() {
  return {schema:"textillab-construction/1.0",measurements:catalogMeasurements(),settings:draftOptions()};
}
function checkpointDraft() {
  if(history.applying||$("templateType").value!=="top")return;
  const value=JSON.stringify(draftSnapshot());
  if(history.items[history.index]===value)return;
  history.items.splice(history.index+1);
  history.items.push(value);
  if(history.items.length>50)history.items.shift();
  history.index=history.items.length-1;
  $("draftUndo").disabled=history.index<=0;
  $("draftRedo").disabled=history.index>=history.items.length-1;
}
function applySnapshot(saved) {
  if(saved.schema!=="textillab-construction/1.0"||
    !saved.measurements||!saved.settings)throw Error("Unbekannte TextilLab-Projektversion.");
  const m=saved.measurements,c=saved.settings;
  const validMeasureNames=[...Object.keys(TextilLabPatternLibrary.defaults),...Object.keys(TextilLabBodiceV2.defaults)];
  history.applying=true;
  try {
    for(const key of validMeasureNames)
      if(Object.hasOwn(m,key)) $("m-"+key).value=String(m[key]);
    if(c.fit && c.fit in TextilLabDraftingSystem.fits)$("draftFit").value=c.fit;
    if(c.style && c.style in TextilLabDraftingSystem.styles)$("draftStyle").value=c.style;
    if(c.targetEaseMm!=null)$("draftEase").value=String(c.targetEaseMm);
    if(c.allowance!=null)$("allowance").value=String(c.allowance);
    if(c.density!=null)$("density").value=String(c.density);
    for(const zone of Object.keys(TextilLabDraftingSystem.zones))
      if(c.alterations && Object.hasOwn(c.alterations,zone))
        $("zone-"+zone).value=String(c.alterations[zone]);
    for(const layer of Object.keys(TextilLabDraftingSystem.layers))
      if(c.layers && Object.hasOwn(c.layers,layer))
        $("layer-"+layer).checked=Boolean(c.layers[layer]);
    $("templateType").value="top";
    updateCatalog();
  } finally {history.applying=false;}
}
function showDraftReport(result) {
  const container=$("draftRuleReport");
  container.replaceChildren();
  if(!result.rules)return;
  const cap=result.capBalance,measures=result.measures;
  const status=document.createElement("div");
  status.className="draft-check"+(!cap.reachable||Math.abs(cap.actualEaseMm-cap.targetEaseMm)>1||measures.sideDifferenceMm>5?" warning":"");
  status.textContent="Ärmelkugel "+measures.sleeveCapMm.toFixed(1)+
    " mm · Armloch "+measures.totalArmholeMm.toFixed(1)+
    " mm · Einhalteweite "+measures.sleeveEaseMm.toFixed(1)+
    " mm (Ziel "+cap.targetEaseMm.toFixed(1)+" mm)";
  container.appendChild(status);
  for(const rule of result.rules) {
    const line=document.createElement("div");line.className="rule-line";
    const name=document.createElement("strong");name.textContent=rule.id+" · "+rule.title;
    name.title=rule.formula+" | "+rule.depends.join(", ");
    const val=document.createElement("span");val.textContent=rule.value.toFixed(1)+" "+rule.unit;
    line.append(name,val);container.appendChild(line);
  }
  const disclosure=document.createElement("p");
  disclosure.className="v1-help";
  disclosure.textContent="TextilLab Experimental 1 · eigene Rechenregeln, keine zertifizierte Schneiderlehre. Nahtlängen sind gemessen, Passform und Abnäher nicht freigegeben.";
  container.appendChild(disclosure);
}
function downloadDraft(filename, data, type) {
  const url=URL.createObjectURL(new Blob([data],{type}));
  const a=document.createElement("a");a.href=url;a.download=filename;
  document.body.append(a);a.click();a.remove();
  setTimeout(()=>URL.revokeObjectURL(url),1000);
}


let currentPantsDraft=null;
let currentPantsReference=null;
function showPantsReport(result) {
  const container=$("pantsRuleReport");container.replaceChildren();
  const status=document.createElement("div");
  status.className="draft-check"+(result.warnings.length?" warning":"");
  status.textContent="Seitennähte |Δ| "+result.checks.sideDifferenceMm.toFixed(1)+
    " mm · Innenbein |Δ| "+result.checks.inseamDifferenceMm.toFixed(1)+" mm";
  container.append(status);
  for(const rule of result.rules) {
    const div=document.createElement("div");div.className="rule-line";
    const name=document.createElement("strong");
    name.textContent=rule.id+" "+rule.title;name.title=rule.formula;
    const val=document.createElement("span");val.textContent=rule.value.toFixed(1)+" "+rule.unit;
    div.append(name,val);container.append(div);
  }
  for(const message of result.warnings){
    const note=document.createElement("div");note.className="v1-help";
    note.textContent="• "+message;container.append(note);
  }
  const cite=document.createElement("div");cite.className="v1-help";
  cite.textContent="Regelquelle: "+result.system.label+" · "+result.system.status;
  container.append(cite);
  if(currentPantsReference)showPantsComparison(currentPantsReference,result);
}
function showPantsComparison(reference,draft) {
  const container=$("pantsReferenceReport");container.replaceChildren();
  try {
    const comparison=TextilLabRulesetRegistry.compare(reference,draft);
    const heading=document.createElement("div");
    heading.className="draft-check";
    heading.textContent=comparison.book.title+" · "+comparison.book.edition+
      " · S. "+comparison.book.pages;
    container.append(heading);
    for(const item of comparison.differences) {
      const div=document.createElement("div");div.className="rule-line";
      const key=document.createElement("strong");key.textContent=item.label;
      const val=document.createElement("span");val.textContent=
        (item.differenceMm>=0?"+":"")+item.differenceMm.toFixed(1)+" mm";
      div.append(key,val);container.append(div);
    }
  }catch(error){
    const msg=document.createElement("div");msg.className="draft-check warning";
    msg.textContent=error.message;container.append(msg);
  }
}

function createPieceThumbnail(part) {
  // Draw a real miniature from the computed cutting contour (not a stock icon).
  const points=(part.outline||[]).map(p=>Array.isArray(p)?p:[p.x,p.y]);
  if(points.length<3||points.some(p=>p.length<2||!p.every(Number.isFinite)))return null;
  const minX=Math.min(...points.map(p=>p[0])),maxX=Math.max(...points.map(p=>p[0]));
  const minY=Math.min(...points.map(p=>p[1])),maxY=Math.max(...points.map(p=>p[1]));
  const scale=Math.min(67/Math.max(1,maxX-minX),85/Math.max(1,maxY-minY));
  const ox=8+(67-(maxX-minX)*scale)/2,oy=8+(85-(maxY-minY)*scale)/2;
  const svg=document.createElementNS("http://www.w3.org/2000/svg","svg");
  svg.setAttribute("viewBox","0 0 83 101");
  svg.setAttribute("aria-hidden","true");
  svg.classList.add("piece-thumb");
  const polygon=document.createElementNS("http://www.w3.org/2000/svg","polygon");
  polygon.setAttribute("points",points.map(p=>
    [(ox+(p[0]-minX)*scale).toFixed(2),(oy+(p[1]-minY)*scale).toFixed(2)].join(",")
  ).join(" "));
  svg.appendChild(polygon);
  return svg;
}


let currentHoodResult=null;
function hoodInputs(){
  return {
    head:$("hoodHead").value,
    backNeck:$("hoodBackNeck").value,
    frontNeck:$("hoodFrontNeck").value,
    hoodHeight:$("hoodHeight").value,
    hoodDepth:$("hoodDepth").value,
    ageYears:$("hoodAgeYears").value
  };
}
function hoodSettings(){
  return {
    profile:$("hoodProfile").value,
    style:$("hoodStyle").value,
    neckMode:$("hoodNeckMode").value,
    plannedEaseCm:$("hoodPlannedEase").value,
    stretchPct:$("hoodStretchPct").value,
    materialStretchVerified:$("hoodVerifiedMaterial").checked,
    allowanceMm:$("allowance").value,
    density:$("density").value
  };
}
function showHoodReport(result){
  const container=$("hoodSeamReport");container.replaceChildren();
  const status=document.createElement("div");
  status.className="hood-check "+(result.report.ok?"pass":"fail");
  const first=document.createElement("strong");
  first.textContent=result.report.ok?"Nahtlinien rechnerisch passend":"Nahtkontrolle nicht bestanden";
  const second=document.createElement("span");
  second.textContent=" · Nicht für Fertigung validiert";
  status.append(first,second);container.append(status);
  for(const item of result.report.seams){
    const row=document.createElement("div");row.className="hood-seam "+(item.ok?"ok":"bad");
    const name=document.createElement("strong");name.textContent=item.label;
    const values=document.createElement("span");
    values.textContent=item.aMm.toFixed(2)+" / "+item.bMm.toFixed(2)+
      " mm · Δ "+item.deltaMm.toFixed(2)+" mm · "+item.status;
    row.append(name,values);container.append(row);
  }
  const banner=document.createElement("p");banner.className="hint";
  banner.textContent=result.profile.group==="child"?
    "Kinderprofil: Kordeln am Hals gesperrt. Sichtfeld und Passform am echten Probeteil prüfen.":
    "Eingetragene Halslochlängen am tatsächlichen Kleidungsstück verifizieren. Schnitt ist ein Prototyp.";
  container.append(banner);
}
function updateHoodRows(){
  const group=$("hoodProfile").value,mode=$("hoodNeckMode").value;
  $("hoodChildAgeRow").hidden=group!=="child";
  $("hoodEaseRow").hidden=mode!=="ease";
  $("hoodStretchRow").hidden=mode!=="stretch";
  $("hoodVerifiedRow").hidden=mode!=="stretch";
}

function updateCatalog() {
  const kind = $("templateType").value;
  const name = TextilLabPatternLibrary.types.find(x => x[0] === kind)?.[1] || kind;
  document.body.classList.add("library-mode");
  document.body.classList.toggle("bodice-v2-mode", kind === "top");
  document.body.classList.toggle("pants-mode", kind === "trousers");
  document.body.classList.toggle("hood-mode", kind === "hood");
  $("modelPanelTitle").textContent = "Grundschnitt · " + name;
  try {
    const result = kind === "top"
      ? TextilLabDraftingSystem.generate(catalogMeasurements(), draftOptions())
      : kind === "trousers"
        ? TextilLabPantsSystem.render(catalogMeasurements(), $("allowance").value, $("density").value)
        : kind === "hood"
          ? TextilLabHoodSystem.render(hoodInputs(), hoodSettings())
          : TextilLabPatternLibrary.render(kind, catalogMeasurements(), $("allowance").value, $("density").value);
    latestDraft=kind==="top"?result:null;
    if(kind==="top")showDraftReport(result);
    else $("draftRuleReport").replaceChildren();
    currentPantsDraft=kind==="trousers"?result:null;
    if(kind==="trousers")showPantsReport(result);
    else $("pantsRuleReport").replaceChildren();
    currentHoodResult=kind==="hood"?result:null;
    if(kind==="hood")showHoodReport(result);
    else $("hoodSeamReport").replaceChildren();
    state.currentSvg = result.svg;
    $("preview").innerHTML = result.svg;
    $("pieces").textContent = String(result.pieces);
    $("sewnArea").textContent = (result.sewnArea / 1e6).toFixed(4);
    $("cutArea").textContent = (result.cutArea / 1e6).toFixed(4);
    $("mass").textContent = result.mass.toFixed(1);
    $("exportBtn").disabled = false;
    const catalog = $("libraryInfo");
    catalog.replaceChildren();
    const title = document.createElement("div");
    title.className = "title"; title.textContent = name;
    catalog.appendChild(title);
    const guide = document.createElement("div");
    guide.className = "smallhint";
    guide.textContent = "Die Schnittteile werden aus deinen Maßen neu konstruiert. Alle angegebenen Maße sind in cm, die SVG-Schnittkonturen in mm.";
    catalog.appendChild(guide);
    for (const part of result.parts) {
      const item = document.createElement("div"); item.className = "piece";
      const heading = document.createElement("strong");
      heading.textContent = part.name + " · " + part.count + "×" + (part.fold ? " · Stoffbruch" : "");
      const info = document.createElement("span"); info.textContent = part.notes || "Schnittteil";
      const thumb=createPieceThumbnail(part);
      const copy=document.createElement("div");copy.className="piece-copy";
      copy.append(heading,info);
      if(thumb)item.appendChild(thumb);
      item.appendChild(copy);
      catalog.appendChild(item);
    }
    const caution = document.createElement("div");
    caution.className = "caution";
    caution.textContent = "Konstruktions-Prototyp: Passform, Rundungen, Nähte und Materialverbrauch vor dem Zuschnitt mit einem Probeteil prüfen.";
    catalog.appendChild(caution);
    $("errorDetail").textContent = kind==="hood"?(result.report.ok?"Nähte rechnerisch passend; Prototyp":"Nahtanschlüsse FEHLER"):
      kind==="top"?"Schnittsystem 1.0 · Konzept":"Grundschnitt-Konzept";
    setStatus("Parametrischer Grundschnitt aktualisiert: " + name + " · " + result.pieces + " Teile"+
      (kind==="hood"?" · Nahtkontrolle "+(result.report.ok?"OK":"FEHLER")+" · nicht fertigungstauglich":""));
    checkpointDraft();
  } catch (error) {
    state.currentSvg = "";
    latestDraft=null;
    currentHoodResult=null;
    if(kind==="hood"){
      const list=$("hoodSeamReport");list.replaceChildren();
      const warning=document.createElement("div");warning.className="hood-check fail";
      warning.textContent="Konstruktion gesperrt: "+error.message;
      list.appendChild(warning);
    }
    clearStats();
    $("preview").replaceChildren();
    const label = document.createElement("div"); label.className = "placeholder";
    label.textContent = error.message;
    $("preview").appendChild(label);
    setStatus(error.message, true);
    $("errorDetail").textContent = "Maße überprüfen";
  }
}
async function updatePreview() {
  if ($("templateType").value !== "mesh") {
    updateCatalog();
    return;
  }
  document.body.classList.remove("library-mode", "bodice-v2-mode", "pants-mode", "hood-mode");
  $("modelPanelTitle").textContent = "3D-Modell · Schnittlinien bearbeiten";
  if (!state.source) return;
  $("errorDetail").textContent = "";
  setStatus("Schnittmuster im Browser berechnen …");
  try {
    const result = TextilLabBrowserEngine.generate(state, {
      seams: state.seams,
      allowance: $("allowance").value,
      density: $("density").value,
      unit: $("units").value,
    });
    state.currentSvg = result.svg;
    $("preview").innerHTML = result.svg; // SVG generated exclusively from numeric geometry.
    $("pieces").textContent = String(result.pieces);
    $("sewnArea").textContent = (result.sewnArea / 1e6).toFixed(4);
    $("cutArea").textContent = (result.cutArea / 1e6).toFixed(4);
    $("mass").textContent = result.mass.toFixed(1);
    $("exportBtn").disabled = false;
    setStatus("Schnittmuster direkt im Browser berechnet · " + result.pieces + " Teil(e)");
  } catch (error) {
    state.currentSvg = "";
    clearStats();
    $("preview").replaceChildren();
    const msg = document.createElement("div");
    msg.className = "placeholder";
    msg.textContent = error.message;
    $("preview").appendChild(msg);
    $("errorDetail").textContent = "Abwicklung nicht möglich";
    setStatus(error.message, true);
  }
}
function position(e) {
  const r = canvas.getBoundingClientRect();
  return { x: e.clientX - r.left, y: e.clientY - r.top };
}
canvas.addEventListener("pointerdown", e => {
  canvas.setPointerCapture(e.pointerId);
  state.pointer = { last: position(e), initial: position(e), dragged: false };
});
canvas.addEventListener("pointermove", e => {
  const p = position(e);
  if (state.pointer && e.buttons) {
    const dx = p.x - state.pointer.last.x, dy = p.y - state.pointer.last.y;
    if (Math.hypot(p.x - state.pointer.initial.x, p.y - state.pointer.initial.y) > 5)
      state.pointer.dragged = true;
    if (state.pointer.dragged) {
      state.yaw += dx * 0.008;
      state.pitch = Math.max(-1.4, Math.min(1.4, state.pitch + dy * 0.008));
    }
    state.pointer.last = p;
    state.hoveredEdge = null;
  } else {
    state.hoveredEdge = pickEdge(p);
  }
  draw();
});
canvas.addEventListener("pointerup", e => {
  if (!state.pointer) return;
  const wasDrag = state.pointer.dragged;
  state.pointer = null;
  if (wasDrag) return;
  const p = position(e);
  const edge = pickEdge(p);
  if (edge && edge.faces.length === 2) {
    const key = seamKey(edge);
    if (state.seams.has(key)) state.seams.delete(key);
    else state.seams.add(key);
    updateSeamList();
    updatePreview();
  } else if (edge) {
    setStatus("Außenkante ist bereits eine Stoffkante – bitte innere Kante wählen.");
  } else {
    state.selectedFace = pickFace(p);
    $("faceInfo").textContent = state.selectedFace < 0 ? "Kante wählen" : "Dreieck #" + (state.selectedFace + 1) + " gewählt";
  }
  draw();
});
canvas.addEventListener("pointercancel", () => { state.pointer = null; });
canvas.addEventListener("pointerleave", () => {
  if (!state.pointer) { state.hoveredEdge = null; draw(); }
});
canvas.addEventListener("wheel", e => {
  e.preventDefault();
  state.zoom = Math.max(0.25, Math.min(8, state.zoom * (e.deltaY > 0 ? 0.9 : 1.1)));
  draw();
}, { passive: false });
$("viewBtn").addEventListener("click", resetCamera);
$("clearBtn").addEventListener("click", () => {
  state.seams.clear(); updateSeamList(); draw(); updatePreview();
});
for (const id of ["allowance", "density", "units"]) {
  $(id).addEventListener("change", () => { draw(); updatePreview(); checkpointDraft(); });
}
$("material").addEventListener("change", () => {
  if ($("material").value !== "custom") $("density").value = $("material").value;
  updatePreview();
  checkpointDraft();
});
$("density").addEventListener("input", () => { $("material").value = "custom"; });
$("objFile").addEventListener("change", async e => {
  const file = e.target.files?.[0];
  if (!file) return;
  if (file.size > 2_000_000) return setStatus("OBJ-Datei darf maximal 2 MB groß sein.", true);
  try { await loadModel(await file.text(), file.name); }
  catch (error) { setStatus(error.message, true); }
});
async function loadExample() {
  try {
    const response = await fetch("./example.obj");
    if (!response.ok) throw new Error("Beispiel nicht verfügbar");
    await loadModel(await response.text(), "bent-panel.obj · Demo");
  } catch (error) { setStatus(error.message, true); }
}
$("sampleBtn").addEventListener("click", loadExample);
$("exportBtn").addEventListener("click", () => {
  if (!state.currentSvg) return;
  const url = URL.createObjectURL(new Blob([state.currentSvg], { type: "image/svg+xml;charset=utf-8" }));
  const link = document.createElement("a");
  link.href = url; link.download = "textillab-schnittmuster.svg";
  document.body.append(link); link.click(); link.remove();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
});
$("templateType").addEventListener("change", () => {
  updatePreview();
  checkpointDraft();
  draw();
});
for (const key of [...new Set([...Object.keys(TextilLabPatternLibrary.defaults), ...Object.keys(TextilLabBodiceV2.defaults), ...Object.keys(TextilLabPantsSystem.defaults)])]) {
  const input=$("m-"+key);
  input.addEventListener("input", () => {
    if(key==="ease")$("draftFit").value="custom";
    if ($("templateType").value !== "mesh") {updateCatalog();checkpointDraft();}
  });
}
// Pants-specific waist-to-hip input is distinct from the top drafting input.
$("m-pantsWaistToHip").addEventListener("input",()=>{
  if($("templateType").value==="trousers")updateCatalog();
});

function draftChanged() {
  if($("templateType").value==="top"){
    if($("draftFit").value!=="custom") $("m-ease").value=
      TextilLabDraftingSystem.fits[$("draftFit").value].ease;
    updateCatalog();
    checkpointDraft();
  }
}
for(const id of ["draftFit","draftStyle","draftEase",...Object.keys(TextilLabDraftingSystem.zones).map(k=>"zone-"+k)]) {
  $(id).addEventListener("change", draftChanged);
}
for(const id of Object.keys(TextilLabDraftingSystem.layers).map(k=>"layer-"+k)) {
  $(id).addEventListener("change",draftChanged);
}
$("draftUndo").addEventListener("click",()=>{
  if(history.index<=0)return;
  history.index--;
  applySnapshot(JSON.parse(history.items[history.index]));
  $("draftUndo").disabled=history.index<=0;
  $("draftRedo").disabled=false;
});
$("draftRedo").addEventListener("click",()=>{
  if(history.index>=history.items.length-1)return;
  history.index++;
  applySnapshot(JSON.parse(history.items[history.index]));
  $("draftUndo").disabled=false;
  $("draftRedo").disabled=history.index>=history.items.length-1;
});
$("draftSave").addEventListener("click",()=>{
  try {
    const project=draftSnapshot();
    project.report=latestDraft?
      {rules:latestDraft.rules,capBalance:latestDraft.capBalance}:null;
    downloadDraft("textillab-grundschnitt-v1.json",JSON.stringify(project,null,2),"application/json");
  }catch(e){setStatus(e.message,true);}
});
$("draftLoad").addEventListener("change",async e=>{
  const file=e.target.files?.[0];if(!file)return;
  if(file.size>1024*1024){setStatus("Projektdatei zu groß.",true);return;}
  try{
    const parsed=JSON.parse(await file.text());
    applySnapshot(parsed);
    checkpointDraft();
    setStatus("TextilLab-Projekt geladen: "+file.name);
  }catch(error){setStatus("Projekt konnte nicht gelesen werden: "+error.message,true);}
  e.target.value="";
});
$("preview").addEventListener("click",event=>{
  const target=event.target.closest?.("[data-measure]");
  const measurement=target?.getAttribute("data-measure");
  if(!measurement || $("templateType").value!=="top")return;
  const field=$("m-"+measurement);
  if(!field)return;
  field.closest("details")?.setAttribute("open","");
  field.scrollIntoView({block:"center",behavior:"smooth"});
  field.focus();field.select();
  setStatus("Maß "+measurement+" zur Bearbeitung ausgewählt.");
});


$("pantsReferenceFile").addEventListener("change",async event=>{
  const file=event.target.files?.[0];if(!file)return;
  try {
    if(file.size>1024*1024)throw Error("Referenzdatei größer als 1 MB.");
    const ref=JSON.parse(await file.text());
    if(!currentPantsDraft)throw Error("Zuerst die Hose auswählen.");
    TextilLabRulesetRegistry.compare(ref,currentPantsDraft);
    currentPantsReference=ref;
    showPantsComparison(ref,currentPantsDraft);
    setStatus("Buchreferenz verglichen: "+ref.book.title);
  }catch(err){
    setStatus(err.message,true);
    const box=$("pantsReferenceReport");box.textContent="Vergleich nicht möglich: "+err.message;
  }
  event.target.value="";
});


for(const id of ["hoodHead","hoodBackNeck","hoodFrontNeck","hoodHeight","hoodDepth","hoodAgeYears",
  "hoodStyle","hoodPlannedEase","hoodStretchPct","hoodVerifiedMaterial"]){
  $(id).addEventListener(id==="hoodVerifiedMaterial"?"change":"input",()=>{
    if($("templateType").value==="hood")updateCatalog();
  });
}
$("hoodNeckMode").addEventListener("change",()=>{
  updateHoodRows();
  if($("templateType").value==="hood")updateCatalog();
});
$("hoodProfile").addEventListener("change",()=>{
  const defaults=TextilLabHoodSystem.profiles[$("hoodProfile").value];
  // Explicit profile switch loads *example* measurements; never silently
  // derive adult or child dimensions from the other body-category profiles.
  for(const [field,id] of [["head","hoodHead"],["backNeck","hoodBackNeck"],
    ["frontNeck","hoodFrontNeck"],["hoodHeight","hoodHeight"],["hoodDepth","hoodDepth"]]){
    $(id).value=defaults[field];
  }
  if(defaults.ageYears!=null)$("hoodAgeYears").value=defaults.ageYears;
  updateHoodRows();
  if($("templateType").value==="hood")updateCatalog();
});
updateHoodRows();

document.body.classList.add("library-mode");
$("draftUndo").disabled=true;
$("draftRedo").disabled=true;
new ResizeObserver(draw).observe(canvas);
loadExample();

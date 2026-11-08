/* SolidCraft TextilLab 1.2 interactive studio controls. Geometry stays unchanged. */
(function(root){
"use strict";
const TYPES=[
 ["top","Oberteil","VT · RT · Ärmel"],
 ["jacket","Jacke","Kragen · Ärmel"],
 ["trousers","Hose","Vorder- / Hinterhose"],
 ["hood","Kapuze","2 Seiten · Nahtprüfung"],
 ["skirt","Rock","Abnäher · Bund"],
 ["glove","Handschuh","Hand · Daumen"],
 ["beanie","Mütze","Keile · Bündchen"],
 ["hat","Hut","Krone · Krempe"],
 ["mesh","3D-Abwicklung","OBJ · Schnittkanten"]
];
const CORE={
 top:["chest","waist","hip","shoulder","sleeve","ease"],
 jacket:["chest","waist","hip","back","shoulder","sleeve","ease"],
 trousers:["waist","hip","inseam"],hood:[],
 skirt:["waist","hip"],glove:["hand","handLength"],
 beanie:["head"],hat:["head"],mesh:[]
};
const PARTS={top:["VT","RT","AR"],trousers:["VH","HH"],hood:["KH"]};
const ICONS={
 top:'<path d="M19 10 26 7l6 6 6-6 7 3 8 13-8 5-5-7v29H16V21l-5 7-8-5z"/><path d="M26 7v5l6 6 6-6V7"/>',
 jacket:'<path d="M20 9 26 6l6 6 6-6 6 3 9 14-7 5-5-7v29H16V21l-5 7-7-5z"/><path d="m26 6 6 12 6-12M32 18v31M23 16l5 6m13-6-5 6"/><circle cx="34" cy="29" r="1"/><circle cx="34" cy="36" r="1"/>',
 trousers:'<path d="M14 8h36l-2 19-8 24H30l2-23-2 23H20L12 27z"/><path d="M14 15h36M32 8v20m-16 1 11 2M48 29l-12 2"/>',
 hood:'<path d="M15 49c-4-19-3-31 11-40 14-8 29 4 29 19 0 11-8 16-16 23H18z"/><path d="M15 49 22 45c12-8 12-17 23-22M27 11c5 5 12 11 13 18"/>',
 skirt:'<path d="M19 10h26l-2 8 9 31H12l9-31z"/><path d="M20 17h24M25 10l2 7m10-7-2 7"/>',
 glove:'<path d="M19 51 9 37c-2-3 3-7 6-4l5 5V13c0-5 7-5 7 0v16-22c0-5 7-5 7 0v23-19c0-5 7-5 7 0v21-13c0-5 7-5 7 0v19c0 7-4 12-10 15z"/>',
 beanie:'<path d="M12 36c0-16 7-27 20-27s20 11 20 27v7H12z"/><path d="M11 36h42v10H11zM32 9v27M18 19l14 17m14-17L32 36"/>',
 hat:'<path d="M21 33V20c0-8 5-12 11-12s11 4 11 12v13M21 33h22"/><path d="M11 33h42l6 10c-10 8-44 8-54 0zM21 21h22"/>',
 mesh:'<path d="m32 6 23 13v26L32 58 9 45V19zM32 6v26M9 19l23 13 23-13M32 32v26M9 45l23-13 23 13"/><circle cx="32" cy="32" r="3"/>'
};
function coreFields(type){return CORE[type]||[]}
function partCode(type,index){return PARTS[type]?.[index]||""}
const api={types:TYPES,coreFields,partCode};
if(typeof module!=="undefined"&&module.exports)module.exports=api;
root.TextilLabWorkspace=api;
if(typeof document==="undefined")return;
const $=id=>document.getElementById(id);
const host=$("templateStrip"),template=$("templateType"),preview=$("preview");
if(!host||!template||!preview)return;
const cards=new Map();
for(const [type,name,description] of TYPES){
 const btn=document.createElement("button");
 btn.type="button";btn.className="template-card";btn.dataset.template=type;
 btn.setAttribute("aria-pressed","false");btn.title="Vorlage "+name+" auswählen";
 const icon=document.createElementNS("http://www.w3.org/2000/svg","svg");
 icon.setAttribute("viewBox","0 0 64 64");icon.setAttribute("aria-hidden","true");
 icon.innerHTML=ICONS[type];
 const copy=document.createElement("span");copy.className="card-text";
 const heading=document.createElement("strong");heading.textContent=name;
 const detail=document.createElement("small");detail.textContent=description;
 copy.append(heading,detail);btn.append(icon,copy);
 btn.addEventListener("click",()=>{
  if(template.value===type)return;
  template.value=type;
  template.dispatchEvent(new Event("change",{bubbles:true}));
 });
 host.append(btn);cards.set(type,btn);
}
let selectedIndex=-1,zoom=1,lastSvg=null;
function selectPiece(index){
 const pieces=[...document.querySelectorAll("#libraryInfo .piece")];
 const part=pieces[index];
 selectedIndex=part?index:-1;
 const id=partCode(template.value,index);
 pieces.forEach((node,i)=>{
  node.classList.toggle("is-selected",i===selectedIndex);
  node.setAttribute("aria-selected",String(i===selectedIndex));
 });
 for(const node of preview.querySelectorAll("g[data-piece-id]")){
  node.classList.toggle("selected-piece",!!id&&node.getAttribute("data-piece-id")===id);
 }
 const title=$("selectedPieceTitle"),hint=$("selectedPieceInfo");
 if(!part){
  title.textContent=template.value==="mesh"?"3D-Geometrie":"Schnittteil auswählen";
  hint.textContent="Schnittteil in der Zeichnung oder links auswählen.";
 }else{
  title.textContent=part.querySelector("strong")?.textContent||"Schnittteil";
  hint.textContent=part.querySelector("span")?.textContent||"Parametrisches Schnittteil";
 }
}
function adaptPieces(){
 const elements=[...document.querySelectorAll("#libraryInfo .piece")];
 for(const [i,part]of elements.entries()){
  part.dataset.pieceIndex=String(i);
  part.setAttribute("role","button");part.tabIndex=0;
  part.setAttribute("aria-label","Schnittteil "+(part.querySelector("strong")?.textContent||i+1));
 }
 if(elements.length)selectPiece(Math.min(Math.max(0,selectedIndex),elements.length-1));
 else selectPiece(-1);
}
const info=$("libraryInfo");
info.addEventListener("click",e=>{
 const part=e.target.closest(".piece");
 if(part&&info.contains(part))selectPiece(Number(part.dataset.pieceIndex));
});
info.addEventListener("keydown",e=>{
 if(e.key!=="Enter"&&e.key!==" ")return;
 const part=e.target.closest(".piece");if(!part)return;
 e.preventDefault();selectPiece(Number(part.dataset.pieceIndex));
});
preview.addEventListener("click",e=>{
 const g=e.target.closest?.("g[data-piece-id]");
 if(g&&preview.contains(g)){
  const index=(PARTS[template.value]||[]).indexOf(g.getAttribute("data-piece-id"));
  if(index>=0)selectPiece(index);
 }
});
new MutationObserver(changes=>{
 if(changes.some(c=>c.type==="childList"))adaptPieces();
}).observe(info,{childList:true});
function filterMeasures(){
 const main=document.querySelector(".sidebar > .section > .measure-details");
 if(!main)return;
 const displayed=new Set(coreFields(template.value));
 for(const label of main.querySelectorAll(".measure-input")){
  const input=label.querySelector("input[id^='m-']");if(!input)continue;
  label.dataset.hidden=String(!displayed.has(input.id.slice(2)));
 }
 main.style.display=(template.value==="mesh"||template.value==="hood")?"none":"";
 main.open=true;
}
function refreshZoom(){
 const svg=preview.querySelector("svg");
 if(!svg)return;
 if(svg!==lastSvg){lastSvg=svg;svg.dataset.baseViewBox=svg.getAttribute("viewBox")||"";}
 const v=(svg.dataset.baseViewBox||"").trim().split(/\s+/).map(Number);
 if(v.length!==4||!v.every(Number.isFinite)||v[2]<=0||v[3]<=0)return;
 const [x,y,w,h]=v,nw=w/zoom,nh=h/zoom;
 svg.setAttribute("viewBox",[
  x+(w-nw)/2,y+(h-nh)/2,nw,nh
 ].map(n=>Number(n.toFixed(3))).join(" "));
 $("zoomValue").textContent=Math.round(zoom*100)+"%";
 $("zoomFit").classList.toggle("is-active",zoom===1);
}
function setZoom(v){zoom=Math.max(.65,Math.min(2.5,Math.round(v*20)/20));refreshZoom()}
$("zoomIn").addEventListener("click",()=>setZoom(zoom+.2));
$("zoomOut").addEventListener("click",()=>setZoom(zoom-.2));
$("zoomFit").addEventListener("click",()=>setZoom(1));
$("focusDrawing").addEventListener("click",()=>{
 const focus=document.body.classList.toggle("focus-pattern");
 $("focusDrawing").setAttribute("aria-pressed",String(focus));
 $("focusDrawing").classList.toggle("is-active",focus);
 $("focusDrawing").textContent=focus?"Zurück":"Fokus";
 if(typeof draw==="function")draw();
 refreshZoom();
});
new MutationObserver(changes=>{
 if(!changes.some(c=>c.type==="childList"))return;
 lastSvg=null;zoom=1;refreshZoom();
 if(selectedIndex>=0)selectPiece(selectedIndex);
}).observe(preview,{childList:true});
function setStep(id){
 for(const button of document.querySelectorAll(".workflow-step")){
  const active=button.dataset.step===id;
  button.classList.toggle("is-active",active);
  button.setAttribute("aria-current",active?"step":"false");
 }
 const t=template.value;
 const targets={
  template:host,
  measure:t==="hood"?document.querySelector(".hood-setup .measure-details"):
   t==="trousers"?
   document.querySelector(".pants-setup .measure-details"):
   document.querySelector(".sidebar > .section > .measure-details"),
  construction:t==="hood"?document.querySelector(".hood-setup"):
   t==="trousers"?document.querySelector(".pants-setup"):
   t==="top"?document.querySelector(".v1-panel"):preview,
  check:t==="hood"?$("hoodSeamReport"):
   t==="trousers"?$("pantsRuleReport"):
   t==="top"?$("draftRuleReport"):$("cutArea")
 };
 targets[id]?.scrollIntoView({behavior:"smooth",block:"nearest"});
}
for(const button of document.querySelectorAll(".workflow-step")){
 button.addEventListener("click",()=>setStep(button.dataset.step));
}
preview.addEventListener("click",e=>{
 if(template.value!=="trousers")return;
 const target=e.target.closest?.("[data-measure]");
 const field=target?$("m-"+target.getAttribute("data-measure")):null;
 if(!field)return;
 field.closest("details")?.setAttribute("open","");
 setStep("measure");
 field.scrollIntoView({behavior:"smooth",block:"center"});
 field.focus();field.select();
});
function sync(){
 const type=template.value;
 for(const [id,card] of cards){
  const active=id===type;
  card.classList.toggle("is-current",active);
  card.setAttribute("aria-pressed",String(active));
 }
 const item=TYPES.find(row=>row[0]===type);
 $("workspaceContext").textContent=item?item[1]+" · "+item[2]:"Vorlage wählen";
 $("drawingPaneTitle").textContent=type==="mesh"?"2D-Abwicklung":"2D-Konstruktion · "+(item?.[1]||type);
 $("drawingHint").textContent=type==="mesh"?
  "Meshkante anklicken → Trennnaht setzen":
  "Schnittteil oder Maß anklicken → direkt bearbeiten";
 filterMeasures();selectedIndex=-1;adaptPieces();setZoom(1);
}
template.addEventListener("change",sync);
sync();
})(typeof globalThis!=="undefined"?globalThis:this);

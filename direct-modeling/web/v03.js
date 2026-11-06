import * as THREE from 'three';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
import init, {
  poly_box_wasm, poly_cylinder_wasm, poly_rotate_z_wasm,
  poly_offset_face_wasm, poly_subtract_many_wasm, poly_volume_wasm,
} from '../pkg/solidcraft_direct_modeling.js';

// Geometric operations run in Rust/WASM, not JavaScript. No fallback.
const $ = id => document.getElementById(id);
const clone = value => structuredClone(value);
const log = message => { $('log').textContent = message; };
let objects = [], selected = new Set(), picked = null, history = [], ready = false, counter = 0;
let cutterId = 'C';

function makeDemo() {
  objects = [
    { id:'A', name:'Grundkörper A', color:0x319bce, parts:[poly_box_wasm([-47,-30,-18],[8,28,20])] },
    { id:'B', name:'Gedrehter Körper B', color:0x72c5a4,
      parts:[poly_rotate_z_wasm(poly_box_wasm([12,-25,-20],[67,25,20]),22)] },
    { id:'C', name:'Zylinder-Werkzeug', color:0xf4a85e,
      parts:[poly_cylinder_wasm(8,0,14,-36,36,20)] },
  ];
  selected = new Set(['A','B']);
  picked = null;
  cutterId='C';
}
function remember() {
  history.push(clone(objects));
  if(history.length>32)history.shift();
}
function objVolume(obj) {
  return obj.parts.reduce((sum,p)=>sum+poly_volume_wasm(p),0);
}
const viewport=$('viewport'), scene=new THREE.Scene();
scene.background=new THREE.Color(0x10151b);
const camera=new THREE.PerspectiveCamera(44,1,.1,4000);
camera.up.set(0,0,1);
camera.position.set(125,-158,110);
const renderer=new THREE.WebGLRenderer({antialias:true});
renderer.setPixelRatio(Math.min(window.devicePixelRatio,2));
viewport.prepend(renderer.domElement);
const orbit=new OrbitControls(camera,renderer.domElement);
orbit.enableDamping=true;
orbit.target.set(5,0,0);orbit.update();
scene.add(new THREE.HemisphereLight(0xe5f4ff,0x1a2e3c,2.3));
const sunlight=new THREE.DirectionalLight(0xffffff,2.4);
sunlight.position.set(80,-110,150);scene.add(sunlight);
const grid=new THREE.GridHelper(240,24,0x3e6378,0x263a49);
grid.rotation.x=Math.PI/2;scene.add(grid);
const axis=new THREE.AxesHelper(48);scene.add(axis);
const renderGroup=new THREE.Group();scene.add(renderGroup);
const raycaster=new THREE.Raycaster(),pointer=new THREE.Vector2();
let clickable=[];

function clearMesh() {
  for(const child of [...renderGroup.children]){
    renderGroup.remove(child);
    child.geometry?.dispose();
    child.material?.dispose();
  }
  clickable=[];
}
function draw() {
  clearMesh();
  for(const obj of objects)for(let partIndex=0;partIndex<obj.parts.length;partIndex++){
    const part=obj.parts[partIndex];
    for(let faceIndex=0;faceIndex<part.faces.length;faceIndex++){
      const face=part.faces[faceIndex];
      if(face.length<3)continue;
      const positions=[];
      for(let i=1;i<face.length-1;i++){
        positions.push(...face[0],...face[i],...face[i+1]);
      }
      const geo=new THREE.BufferGeometry();
      geo.setAttribute('position',new THREE.Float32BufferAttribute(positions,3));
      geo.computeVertexNormals();
      const isPicked=picked && picked.id===obj.id &&
        picked.partIndex===partIndex && picked.faceIndex===faceIndex;
      const color=isPicked?0xffd05e:obj.color;
      const material=new THREE.MeshStandardMaterial({
        color, roughness:.63, metalness:.13, side:THREE.DoubleSide,
        transparent:true,opacity:selected.has(obj.id)?.92:.63,depthWrite:true
      });
      const mesh=new THREE.Mesh(geo,material);
      mesh.userData={id:obj.id,partIndex,faceIndex};
      renderGroup.add(mesh);clickable.push(mesh);
      const outline=new THREE.BufferGeometry().setFromPoints(face.map(x=>new THREE.Vector3(...x)));
      const edge=new THREE.LineLoop(outline,new THREE.LineBasicMaterial({
        color:selected.has(obj.id)?0xd2edff:0x647889,transparent:true,opacity:.85
      }));
      renderGroup.add(edge);
    }
  }
}
function drawTree() {
  $('tree').replaceChildren();
  const cutter=$('cutter'),current=cutterId;
  cutter.replaceChildren();
  for(const obj of objects){
    const opt=new Option(obj.name,obj.id);
    cutter.add(opt);
    const button=document.createElement('button');
    button.className='tree'+(selected.has(obj.id)?' on':'');
    button.textContent=(selected.has(obj.id)?'☑ ':'☐ ')+obj.name+
      ' · '+obj.parts.length+' Teil(e) · '+objVolume(obj).toFixed(1)+' mm³';
    button.onclick=()=>{
      if(selected.has(obj.id))selected.delete(obj.id);else selected.add(obj.id);
      picked=null;refresh();
    };
    $('tree').append(button);
  }
  cutterId=objects.some(o=>o.id===current)?current:(objects.at(-1)?.id??'');
  cutter.value=cutterId;
  $('undo').disabled=!ready||!history.length;
  $('moveFace').disabled=!ready||!picked;
  $('rotate').disabled=!ready||selected.size!==1;
  $('subtract').disabled=!ready||objects.filter(o=>selected.has(o.id)&&o.id!==cutterId).length===0;
  $('newBox').disabled=!ready;$('newCylinder').disabled=!ready;
  $('save').disabled=!ready;
  if(!picked){
    $('inspect').textContent=selected.size+' Körper ausgewählt. Auf eine Fläche klicken, um sie direkt zu bearbeiten.';
  }else{
    const obj=objects.find(o=>o.id===picked.id);
    $('inspect').textContent=obj.name+' – Teil '+(picked.partIndex+1)+
      ', Fläche '+(picked.faceIndex+1)+'\nVolumen: '+objVolume(obj).toFixed(2)+' mm³';
  }
}
function refresh(){drawTree();draw();}

$('cutter').addEventListener('change',()=>{cutterId=$('cutter').value;drawTree();});
$('reset').onclick=()=>{remember();makeDemo();refresh();log('Demo zurückgesetzt.');};
$('undo').onclick=()=>{
  if(!history.length)return;
  objects=history.pop();picked=null;selected=new Set(objects.filter(o=>o.id!=='C').map(o=>o.id));
  refresh();log('Letzten Modellierungsschritt rückgängig gemacht.');
};
$('newBox').onclick=()=>{
  const i=++counter,dx=i*15;
  const solid=poly_box_wasm([-15+dx,-16,-15],[15+dx,16,15]);
  remember();
  const id='N'+i;objects.push({id,name:'Quader '+i,color:0x80a7dc,parts:[solid]});
  selected=new Set([id]);picked=null;refresh();log('Neuen Quader erstellt.');
};
$('newCylinder').onclick=()=>{
  const i=++counter,dx=i*15;
  const solid=poly_cylinder_wasm(dx,0,12,-15,15,20);
  remember();const id='N'+i;
  objects.push({id,name:'Zylinder '+i,color:0xa8b3e4,parts:[solid]});
  selected=new Set([id]);picked=null;refresh();log('Neuen polygonalen Zylinder erstellt.');
};
$('moveFace').onclick=()=>{
  if(!picked)return;
  const delta=Number($('offset').value);
  if(!Number.isFinite(delta))return log('Bitte gültigen Versatz eingeben.');
  const obj=objects.find(o=>o.id===picked.id);
  try{
    const newPart=poly_offset_face_wasm(obj.parts[picked.partIndex],picked.faceIndex,delta);
    remember();
    obj.parts[picked.partIndex]=newPart;
    picked=null;refresh();
    log('Flächenversatz '+delta+' mm berechnet und Nachbarflächen neu aufgebaut (Rust).');
  }catch(e){log('Flächenversatz nicht möglich: '+e);}
};
$('rotate').onclick=()=>{
  const obj=objects.find(o=>selected.has(o.id));
  if(selected.size!==1||!obj)return;
  const angle=Number($('angle').value);
  if(!Number.isFinite(angle))return log('Ungültiger Winkel.');
  try{
    const parts=obj.parts.map(p=>poly_rotate_z_wasm(p,angle));
    remember();obj.parts=parts;picked=null;refresh();
    log('Drehung um '+angle+'° ausgeführt (Rust).');
  }catch(e){log('Drehfehler: '+e);}
};
$('subtract').onclick=()=>{
  const tool=objects.find(o=>o.id===cutterId);
  const targets=objects.filter(o=>selected.has(o.id)&&o.id!==cutterId);
  if(!tool||!targets.length)return log('Mindestens einen anderen Zielkörper markieren.');
  try{
    const flat=targets.flatMap(o=>o.parts);
    const result=poly_subtract_many_wasm(flat,tool.parts);
    if(!Array.isArray(result)||result.length!==flat.length)throw new Error('Ungültiges Rust-Ergebnis');
    remember();
    let index=0;
    for(const target of targets){
      target.parts=target.parts.flatMap(()=>result[index++]);
    }
    picked=null;refresh();
    log('Boolean-Differenz berechnet: '+targets.length+' Zielkörper, '+
      tool.parts.length+' Werkzeugteile. Ergebnis: '+targets.reduce((sum,t)=>sum+t.parts.length,0)+' konvexe Teile.');
  }catch(e){log('Boolean-Fehler: '+e);}
};
$('save').onclick=()=>{
  const json=JSON.stringify({format:'solidcraft-convex-v03',objects},null,2);
  const url=URL.createObjectURL(new Blob([json],{type:'application/json'}));
  const a=document.createElement('a');a.href=url;a.download='solidcraft-direct-v03.json';
  a.click();setTimeout(()=>URL.revokeObjectURL(url),1000);
};
$('open').onclick=()=>$('file').click();
$('file').onchange=async e=>{
  const file=e.target.files?.[0];if(!file)return;
  try{
    if(file.size>5_000_000)throw new Error('Datei ist zu groß.');
    const data=JSON.parse(await file.text());
    if(data.format!=='solidcraft-convex-v03'||!Array.isArray(data.objects)||data.objects.length>128)
      throw new Error('Unbekanntes Dateiformat.');
    const ids=new Set();
    for(const obj of data.objects){
      if(typeof obj.id!=='string'||!obj.id||ids.has(obj.id)||
         typeof obj.name!=='string'||!Number.isInteger(obj.color)||
         !Array.isArray(obj.parts)||obj.parts.length>512)
        throw new Error('Ungültige Objektdaten.');
      ids.add(obj.id);
      for(const p of obj.parts)poly_volume_wasm(p); // validate with Rust
    }
    remember();objects=data.objects;selected=new Set();
    picked=null;counter=objects.length;refresh();
    log('Projekt geladen: '+objects.length+' Objekte.');
  }catch(err){log('Importfehler: '+err);}
  finally{$('file').value='';}
};

let pointerDown=null;
renderer.domElement.addEventListener('pointerdown',e=>{
  pointerDown=[e.clientX,e.clientY];
});
renderer.domElement.addEventListener('pointerup',e=>{
  if(!pointerDown||Math.hypot(e.clientX-pointerDown[0],e.clientY-pointerDown[1])>5)return;
  const rect=renderer.domElement.getBoundingClientRect();
  pointer.x=((e.clientX-rect.left)/rect.width)*2-1;
  pointer.y=-((e.clientY-rect.top)/rect.height)*2+1;
  raycaster.setFromCamera(pointer,camera);
  const hits=raycaster.intersectObjects(clickable,false);
  if(hits.length){
    picked={...hits[0].object.userData};
    selected=new Set([picked.id]);
    refresh();
  }
  pointerDown=null;
});
const resize=()=>{
  const rect=viewport.getBoundingClientRect();
  camera.aspect=rect.width/Math.max(1,rect.height);
  camera.updateProjectionMatrix();renderer.setSize(rect.width,Math.max(1,rect.height));
};
new ResizeObserver(resize).observe(viewport);
function frame(){requestAnimationFrame(frame);orbit.update();renderer.render(scene,camera);}
frame();resize();
try{
  await init();
  ready=true;makeDemo();refresh();
  $('status').textContent='● Rust/WASM v0.3 verbunden';
  $('status').style.color='#72e3a4';
  log('Bereit: 2 Zielkörper und ein polygonaler Zylinder-Werkzeugkörper. Wähle eine Fläche oder starte den Boolean.');
}catch(e){
  $('status').textContent='● Rust/WASM fehlgeschlagen';
  $('status').style.color='#f4a28b';
  log('WebAssembly konnte nicht geladen werden: '+e+'\nBitte wasm-pack build --target web --features wasm ausführen und den lokalen HTTP-Server verwenden.');
}

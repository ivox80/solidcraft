import * as THREE from 'three';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
import init, { subtract_many_wasm, move_face_wasm } from '../pkg/solidcraft_direct_modeling.js';

// The viewer renders Rust outputs; geometry operations do not have a JavaScript fallback.
const $ = id => document.getElementById(id);
const box = (min,max) => ({min:[...min],max:[...max]});
const demo = () => [
  {id:'A',name:'Grundkörper A',parts:[box([-55,-25,-20],[10,30,22])],color:0x379ec9},
  {id:'B',name:'Grundkörper B',parts:[box([15,-22,-18],[62,27,18])],color:0x78b3a0},
  {id:'C',name:'Werkzeug C',parts:[box([-8,-40,-8],[28,40,8])],color:0xf0ad5c}
];
let objects=demo(), selected=new Set(['A','B']), undo=[], loaded=false;
function clone(value){return structuredClone(value);}
function log(msg){$('log').textContent=msg;}
function redrawUI(){
  $('objects').replaceChildren();
  $('cutter').replaceChildren();
  for(const obj of objects){
    const opt=document.createElement('option');opt.value=obj.id;opt.textContent=obj.name;$('cutter').append(opt);
    const btn=document.createElement('button');btn.className='item'+(selected.has(obj.id)?' selected':'');
    btn.textContent=(selected.has(obj.id)?'☑ ':'☐ ')+obj.name+' ('+obj.parts.length+' Teilkörper)';
    btn.onclick=()=>{if(selected.has(obj.id))selected.delete(obj.id);else selected.add(obj.id);render();};
    $('objects').append(btn);
  }
  $('cutter').value=objects.some(x=>x.id==='C')?'C':(objects.at(-1)?.id??'');
  $('selected-label').textContent= selected.size===1? objects.find(x=>selected.has(x.id))?.name : selected.size+' ausgewählt';
  $('move').disabled=!loaded||selected.size!==1;
  $('subtract').disabled=!loaded||selected.size===0;
  $('undo').disabled=!loaded||undo.length===0;
}
const scene=new THREE.Scene();scene.background=new THREE.Color(0x11151c);
const viewport=$('viewport');
const camera=new THREE.PerspectiveCamera(42,1,0.1,2000);camera.position.set(140,110,155);
const renderer=new THREE.WebGLRenderer({antialias:true});renderer.setPixelRatio(Math.min(window.devicePixelRatio,2));
viewport.prepend(renderer.domElement);
const controls=new OrbitControls(camera,renderer.domElement);controls.enableDamping=true;
scene.add(new THREE.HemisphereLight(0xeaf5ff,0x243447,2.2));
const sun=new THREE.DirectionalLight(0xffffff,2.2);sun.position.set(80,140,90);scene.add(sun);
scene.add(new THREE.GridHelper(250,25,0x466071,0x283643));
const dynamic=new THREE.Group();scene.add(dynamic);
function draw(){
  while(dynamic.children.length){
    const child=dynamic.children[0];dynamic.remove(child);child.geometry.dispose();
    child.material.dispose();
  }
  for(const obj of objects)for(const part of obj.parts){
    const size=part.max.map((v,i)=>v-part.min[i]);
    if(size.some(n=>n<=0))continue;
    const center=part.max.map((v,i)=>(v+part.min[i])/2);
    const geo=new THREE.BoxGeometry(...size);
    const mat=new THREE.MeshStandardMaterial({color:obj.color,transparent:true,opacity:selected.has(obj.id)?0.93:0.53,roughness:.6,metalness:.1});
    const mesh=new THREE.Mesh(geo,mat);mesh.position.set(...center);dynamic.add(mesh);
    const edge=new THREE.LineSegments(new THREE.EdgesGeometry(geo),new THREE.LineBasicMaterial({color:selected.has(obj.id)?0xf0f7ff:0x536478}));
    edge.position.copy(mesh.position);dynamic.add(edge);
  }
}
function render(){redrawUI();draw();}
$('reset').onclick=()=>{undo.push(clone(objects));objects=demo();selected=new Set(['A','B']);render();log('Demo zurückgesetzt');};
$('move').onclick=()=>{
  if(selected.size!==1)return;
  const obj=objects.find(o=>selected.has(o.id));if(!obj)return;
  const axis=Number($('axis').value),positive=$('side').value==='max',delta=Number($('delta').value);
  try{
    const updated=obj.parts.map(part=>move_face_wasm(part,axis,positive,delta));
    undo.push(clone(objects));obj.parts=updated;render();log(obj.name+': Fläche auf Achse '+axis+' um '+delta+' mm bewegt (Rust).');
  }catch(e){log('Fehler: '+e);}
};
$('subtract').onclick=()=>{
  const cutter=objects.find(o=>o.id===$('cutter').value);
  const targets=objects.filter(o=>selected.has(o.id)&&o.id!==cutter?.id);
  if(!cutter||!targets.length){log('Bitte mindestens einen Zielkörper auswählen (nicht nur den Werkzeugkörper).');return;}
  try{
    const flat=targets.flatMap(o=>o.parts);
    const result=subtract_many_wasm(flat,cutter.parts);
    if(!Array.isArray(result)||result.length!==flat.length)throw Error('Ungültiges Rust-Ergebnis');
    undo.push(clone(objects));let i=0;
    for(const o of targets){o.parts=o.parts.flatMap(()=>result[i++]);}
    render();log('Rust Boolean erfolgreich: '+targets.length+' Zielkörper, '+cutter.parts.length+' Werkzeugkörper.');
  }catch(e){log('Fehler: '+e);}
};
$('undo').onclick=()=>{if(!undo.length)return;objects=undo.pop();render();log('Rückgängig gemacht.');};
function resize(){
  const {width,height}=viewport.getBoundingClientRect();
  camera.aspect=width/Math.max(height,1);camera.updateProjectionMatrix();
  renderer.setSize(width,Math.max(height,1));
}
new ResizeObserver(resize).observe(viewport);
function animate(){requestAnimationFrame(animate);controls.update();renderer.render(scene,camera);}
animate();resize();render();
try{
  await init();loaded=true;$('status').textContent='● Rust WebAssembly aktiv';
  $('status').style.color='#75dc9a';log('Rust/WASM initialisiert. Zwei Zielkörper und ein Werkzeugkörper sind vorbereitet.');
}catch(e){
  $('status').textContent='● WASM nicht verfügbar';$('status').style.color='#ffad88';
  log('WASM konnte nicht geladen werden: '+e+'\nBitte zuerst wasm-pack build --target web --features wasm im Ordner direct-modeling ausführen und einen lokalen HTTP-Server verwenden.');
}
render();
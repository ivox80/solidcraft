/* TextilLab procedural 3D garment viewer.
 * This is a dimension-responsive geometric drape, not a cloth solver or
 * proof that a 2D block will sew into this exact 3D volume.
 * Orthographic Canvas renderer, no external network or runtime dependencies.
 */
(function(root){
"use strict";
const PI=Math.PI,clamp=(x,a,b)=>Math.max(a,Math.min(b,x));
const createMesh=(name,color,body=false)=>({name,color,body,vertices:[],faces:[],edgeGroups:[]});
function v(m,x,y,z){m.vertices.push([x,y,z]);return m.vertices.length-1}
function tri(m,a,b,c){m.faces.push([a,b,c])}
function tube(m,radii,ys,segments=30,openAngle=2*PI){
 const rings=[];
 for(let i=0;i<ys.length;i++){
  const ring=[];
  for(let k=0;k<=segments;k++){
   const a=-openAngle/2+openAngle*k/segments;
   const rx=radii[i][0],rz=radii[i][1];
   ring.push(v(m,Math.sin(a)*rx,ys[i],-Math.cos(a)*rz));
  }
  rings.push(ring);
 }
 for(let i=0;i<rings.length-1;i++)
  for(let k=0;k<segments;k++){
   const a=rings[i][k],b=rings[i][k+1],c=rings[i+1][k],d=rings[i+1][k+1];
   tri(m,a,b,c);tri(m,b,d,c);
  }
 return rings;
}
function hoodModel(spec){
 const h=spec.heightMm||320,depth=spec.depthMm||270;
 const headR=(spec.headCircMm||570)/(2*PI);
 const rx=Math.max(headR*1.13,105),rz=Math.max(headR*1.13,depth*.47);
 const neckFactor=clamp((spec.baseNeckMm||220)/220,.7,1.45);
 const overlap=spec.faceOverlapMm||0;
 const m=createMesh("Kapuze 3D","#52aabe");
 const rings=[],steps=24,arcs=36;
 for(let i=0;i<=steps;i++){
  const t=i/steps;
  const y=-24+h*t;
  const crownFactor=Math.sqrt(Math.max(.05,1-.81*t*t));
  const w=rx*(1.15-.29*t)*crownFactor*(1+(neckFactor-1)*(1-t));
  const d=rz*(1.1-.18*t)*crownFactor;
  const ring=[];
  for(let k=0;k<=arcs;k++){
   const a=-2.05+4.1*k/arcs;
   const x=Math.sin(a)*w;
   const z=-Math.cos(a)*d + (depth*.035*t)+overlap*Math.max(0,(Math.abs(a)-1.55)/.5);
   ring.push(v(m,x,y,z));
  }
  rings.push(ring);
 }
 for(let i=0;i<steps;i++)for(let k=0;k<arcs;k++){
  const a=rings[i][k],b=rings[i][k+1],
        c=rings[i+1][k],d=rings[i+1][k+1];
  tri(m,a,b,c);tri(m,b,d,c);
 }
 const top=v(m,0,h+8,0);
 for(let k=0;k<arcs;k++)tri(m,rings.at(-1)[k],rings.at(-1)[k+1],top);
 m.edgeGroups=[
  {indices:rings[0],color:"#f9b879",width:1.75},
  {indices:rings.map(r=>r[0]),color:"#f3bc92",width:2.1},
  {indices:rings.map(r=>r.at(-1)),color:"#f3bc92",width:2.1},
  {indices:rings.map(r=>r[Math.floor(arcs/2)]),color:"#a0eef1",width:1.25}
 ];
 const head=createMesh("Kopf-Referenz","#bbad9c",true);
 const r=headR*.89,center=h*.47,radials=28,lats=22;
 const vertices=[];
 for(let j=0;j<=lats;j++){
  const t=(j/lats)*PI,y=center+Math.cos(t)*headR*1.27;
  const ring=[];
  for(let k=0;k<=radials;k++){
   const a=2*PI*k/radials;
   ring.push(v(head,r*Math.sin(t)*Math.cos(a),y,headR*.96*Math.sin(t)*Math.sin(a)+22));
  }
  vertices.push(ring);
 }
 for(let j=0;j<lats;j++)for(let k=0;k<radials;k++)
  tri(head,vertices[j][k],vertices[j+1][k],vertices[j][k+1]),
  tri(head,vertices[j][k+1],vertices[j+1][k],vertices[j+1][k+1]);
 return [head,m];
}
function bodyModel(type,data={}){
 const chest=(Number(data.chest)||100)*10,waist=(Number(data.waist)||84)*10,
  hip=(Number(data.hip)||102)*10;
 const v=Number(data.ease)||8;
 const outfit=createMesh("Volumen "+type,"#51b5c3");
 if(type==="trousers"){
  const hp=(Number(data.hip)||102)*10/(2*PI);
  tube(outfit,[[hp*.92,95],[hp*1.03,115],[hp*.94,106]],[0,130,270],30);
  const meshes=[outfit];
  for(const side of [-1,1]){
   const leg=createMesh("Hosenbein "+side,"#3c92a6");
   const x=side*hp*.54,inseam=(Number(data.inseam)||80)*10;
   const ls=tube(leg,[[hp*.58,70],[hp*.51,63],[hp*.42,58]],[10,-inseam*.48,-inseam],26);
   for(const vert of leg.vertices)vert[0]+=x;
   leg.edgeGroups=[{indices:ls.at(-1),color:"#eaa778",width:1.5}];
   meshes.push(leg);
  }
  return meshes;
 }
 if(type==="skirt"){
  const radius=waist/(2*PI);
  tube(outfit,[[hip/(2*PI)*1.2,120],[radius,98]],[ -350,120],36);
  return [outfit];
 }
 if(type==="beanie"||type==="hat"){
  const r=((Number(data.head)||57)*10)/(2*PI);
  const rings=tube(outfit,[[r,r],[r*1.01,r*1.01],[r*.84,r*.84],[r*.38,r*.38],[.5,.5]],[-90,20,100,160,180],36);
  if(type==="hat"){
   const b=createMesh("Hutkrempe","#5ca7ba");
   const inner=[],outer=[];
   for(let k=0;k<=40;k++){
    const a=2*PI*k/40;
    inner.push(vtx(b,r*Math.cos(a),-90,r*Math.sin(a)));
    outer.push(vtx(b,r*1.56*Math.cos(a),-100,r*1.56*Math.sin(a)));
   }
   for(let k=0;k<40;k++)tri(b,inner[k],outer[k],inner[k+1]),tri(b,inner[k+1],outer[k],outer[k+1]);
   return [outfit,b];
  }
  return [outfit];
 }
 if(type==="glove"){
  const hand=(Number(data.hand)||22)*10/(2*PI);
  tube(outfit,[[hand*.80,hand*.4],[hand*1.1,hand*.35],[hand*.7,hand*.32]],[0,120,180],26);
  return [outfit];
 }
 return [outfit];
}
function vtx(mesh,x,y,z){return v(mesh,x,y,z)}
function modelFor(type,data={}){
 if(type==="hood")return hoodModel(data);
 if(type==="top"||type==="jacket"){
  const chest=(Number(data.chest)||100)*10,waist=(Number(data.waist)||84)*10,
    hip=(Number(data.hip)||102)*10,armLength=(Number(data.sleeve)||62)*10;
  const torso=createMesh("Oberkörper","#4ba5b7");
  tube(torso,[[hip/(2*PI)*.80,100],[waist/(2*PI)*.82,98],
    [chest/(2*PI)*.83,118],[chest/(2*PI)*.60,100]],[-255,-80,135,215],32);
  const group=[torso];
  for(const side of [-1,1]){
   const sleeve=createMesh("Ärmel "+side,"#4897ae");
   const center=side*chest/(2*PI)*1.0;
   const rings=tube(sleeve,[[45,46],[52,50],[34,35]],[170,-armLength*.3,-armLength*.78],22);
   sleeve.vertices.forEach((p,i)=>{
    const ratio=(170-p[1])/(170+armLength*.78);
    p[0]+=center+side*ratio*armLength*.32;
   });
   sleeve.edgeGroups=[{indices:rings.at(-1),color:"#dfaf8f",width:1.15}];
   group.push(sleeve);
  }
  return group;
 }
 return bodyModel(type,data);
}
function rgb(hex){const s=hex.replace("#","");return [0,2,4].map(i=>parseInt(s.slice(i,i+2),16))}
function shade(color,f){
 const c=rgb(color);return "rgb("+c.map(n=>clamp(Math.round(n*f),0,255)).join(",")+")";
}
function exportOBJ(meshes){
 let result=["# TextilLab schematic model; not a physical drape or garment-fit proof"];
 let offset=1;
 for(const m of meshes){
  if(m.body)continue;
  result.push("o "+m.name.replace(/\s+/g,"_"));
  for(const a of m.vertices)result.push("v "+a.map(x=>x.toFixed(5)).join(" "));
  for(const face of m.faces)result.push("f "+face.map(i=>i+offset).join(" "));
  offset+=m.vertices.length;
 }
 return result.join("\n")+"\n";
}
function createRenderer(canvas){
 const ctx=canvas.getContext("2d");if(!ctx)throw Error("3D Canvas nicht verfügbar");
 const state={yaw:-.55,pitch:-.18,zoom:1,wire:false,body:true,
  meshes:[],drag:null,model:null};
 function rotate(pt){
  const cy=Math.cos(state.yaw),sy=Math.sin(state.yaw),
   cp=Math.cos(state.pitch),sp=Math.sin(state.pitch);
  const xx=pt[0]*cy-pt[2]*sy,zz=pt[0]*sy+pt[2]*cy;
  return [xx,pt[1]*cp-zz*sp,pt[1]*sp+zz*cp];
 }
 function redraw(){
  const rect=canvas.getBoundingClientRect(),w=rect.width,h=rect.height;
  if(w<2||h<2)return;
  const dpr=Math.min(root.devicePixelRatio||1,2);
  canvas.width=Math.round(w*dpr);canvas.height=Math.round(h*dpr);
  ctx.setTransform(dpr,0,0,dpr,0,0);
  const grad=ctx.createLinearGradient(0,0,w,h);
  grad.addColorStop(0,"#1d3545");grad.addColorStop(1,"#0d1b28");
  ctx.fillStyle=grad;ctx.fillRect(0,0,w,h);
  ctx.strokeStyle="#39566b38";ctx.lineWidth=1;
  for(let x=12;x<w;x+=33){ctx.beginPath();ctx.moveTo(x,0);ctx.lineTo(x,h);ctx.stroke()}
  for(let y=12;y<h;y+=33){ctx.beginPath();ctx.moveTo(0,y);ctx.lineTo(w,y);ctx.stroke()}
  const meshes=state.meshes.filter(m=>!m.body||state.body);
  if(!meshes.length)return;
  const points=meshes.flatMap(m=>m.vertices.map(rotate));
  const ext=Math.max(200,...points.map(p=>Math.abs(p[0])*2.4),...points.map(p=>Math.abs(p[1])*2.0));
  const scale=Math.min(w,h)/(ext)*state.zoom*.86;
  const transformed=new Map(),all=[];
  const project=p=>[w*.5+p[0]*scale,h*.55-p[1]*scale,p[2]];
  for(const mesh of meshes){
   const rotated=mesh.vertices.map(rotate);
   transformed.set(mesh,rotated.map(project));
   for(const f of mesh.faces){
    const a=rotated[f[0]],b=rotated[f[1]],c=rotated[f[2]];
    const ab=[b[0]-a[0],b[1]-a[1],b[2]-a[2]],ac=[c[0]-a[0],c[1]-a[1],c[2]-a[2]];
    const nx=ab[1]*ac[2]-ab[2]*ac[1],ny=ab[2]*ac[0]-ab[0]*ac[2],nz=ab[0]*ac[1]-ab[1]*ac[0];
    const mag=Math.hypot(nx,ny,nz)||1;
    const light=.70+ .26*Math.abs((nx*.2+ny*.60+nz*.72)/mag);
    all.push({mesh,face:f,depth:(a[2]+b[2]+c[2])/3,light});
   }
  }
  all.sort((a,b)=>a.depth-b.depth);
  for(const {mesh,face,light}of all){
   const p=transformed.get(mesh);
   ctx.beginPath();ctx.moveTo(p[face[0]][0],p[face[0]][1]);
   ctx.lineTo(p[face[1]][0],p[face[1]][1]);ctx.lineTo(p[face[2]][0],p[face[2]][1]);ctx.closePath();
   ctx.fillStyle=shade(mesh.color,light);ctx.fill();
   if(state.wire){
    ctx.lineWidth=.5;ctx.strokeStyle="#d1f8f83d";ctx.stroke();
   }
  }
  for(const m of meshes){
   const pts=transformed.get(m);
   for(const group of m.edgeGroups){
    ctx.strokeStyle=group.color;ctx.lineWidth=group.width||1.4;ctx.beginPath();
    group.indices.forEach((id,i)=>i?ctx.lineTo(pts[id][0],pts[id][1]):ctx.moveTo(pts[id][0],pts[id][1]));ctx.stroke();
   }
  }
  ctx.fillStyle="#b2cfda";ctx.font="11px Segoe UI, Arial";ctx.fillText("3D · schematische Drapierung",12,23);
  ctx.fillStyle="#7a9cab";ctx.font="10px Segoe UI, Arial";ctx.fillText("Ziehen: drehen · Rad: zoomen",12,h-14);
 }
 canvas.addEventListener("pointerdown",e=>{
  state.drag={x:e.clientX,y:e.clientY};
  canvas.setPointerCapture?.(e.pointerId);
 });
 canvas.addEventListener("pointermove",e=>{
  if(!state.drag)return;
  state.yaw+= (e.clientX-state.drag.x)*.008;
  state.pitch=clamp(state.pitch+(e.clientY-state.drag.y)*.006,-1.35,1.35);
  state.drag={x:e.clientX,y:e.clientY};redraw();
 });
 const end=()=>{state.drag=null};
 canvas.addEventListener("pointerup",end);canvas.addEventListener("pointercancel",end);
 canvas.addEventListener("wheel",e=>{
  e.preventDefault();state.zoom=clamp(state.zoom*(e.deltaY<0?1.11:.9),.4,3);redraw();
 },{passive:false});
 return {setModel(type,data){state.model={type,data};state.meshes=modelFor(type,data);redraw()},
  setOptions(options){if("wire"in options)state.wire=!!options.wire;
   if("body"in options)state.body=!!options.body;redraw()},
  reset(){state.yaw=-.55;state.pitch=-.18;state.zoom=1;redraw()},
  draw:redraw,obj(){return exportOBJ(state.meshes)},state};
}
const API={modelFor,createRenderer,exportOBJ};
if(typeof module!=="undefined"&&module.exports)module.exports=API;
root.TextilLabGarment3D=API;
})(typeof globalThis!=="undefined"?globalThis:this);

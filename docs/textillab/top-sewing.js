/* TextilLab v1.5 – garment seam-graph and deformable bodice+two-sleeve preview.
 * Source: actual 2D TextilLab Bodice V2 sampled sewing curves, not generic tubes.
 * Every sewn connection names BOTH partner edges; pairing is computed in mm
 * and welded in the 3D topology, while each panel retains its own flat 2D
 * spring rest lengths. Real easing/warp cannot be asserted without validated
 * processing, grain directions and material measurements.
 */
(function(root){
"use strict";
const PI=Math.PI,clamp=(x,a,b)=>Math.max(a,Math.min(b,x));
const pt=(x,y,z)=>[x,y,z],norm=(a,b)=>Math.hypot(a[0]-b[0],a[1]-b[1]);
const length=p=>p.slice(1).reduce((sum,v,i)=>sum+norm(v,p[i]),0);
const p2=(a,b,t)=>[a[0]*(1-t)+b[0]*t,a[1]*(1-t)+b[1]*t];
function resample(path,count){
 if(!Array.isArray(path)||path.length<2||count<2||!Number.isInteger(count))throw Error("Schnittkante mit mind. 2 Punkten erforderlich.");
 let lens=[],total=0;
 for(let i=1;i<path.length;i++){
  const n=norm(path[i-1],path[i]);lens.push(n);total+=n;
 }
 if(total<1||!Number.isFinite(total))throw Error("Ungültige Nähkante ohne Bogenlänge.");
 const result=[];
 for(let k=0;k<=count;k++){
  let dist=total*k/count,i=0;
  while(i<lens.length-1 && dist>lens[i]){dist-=lens[i];i++;}
  result.push(p2(path[i],path[i+1],lens[i]?clamp(dist/lens[i],0,1):0));
 }
 return result;
}
function bodyEdges(p){
 const a=p.outline;
 if(a.length!==131)throw Error("Bodice-V2 Konstruktionspunkte verändert: Nähkanten nicht freigegeben.");
 return [
  {name:"neck",path:a.slice(0,25),n:14},
  {name:"shoulder",path:a.slice(24,26),n:8},
  {name:"armhole",path:a.slice(25,56),n:20},
  {name:"side",path:a.slice(55,130),n:36},
  {name:"hem",path:a.slice(129,131),n:16},
  {name:"fold",path:[a[130],a[0]],n:40}
 ];
}
function sleeveEdges(p){
 const a=p.outline;
 if(a.length!==89)throw Error("Ärmelkontur verändert: Nähkanten nicht freigegeben.");
 return [
  {name:"capFront",path:a.slice(0,25),n:20},
  {name:"capBack",path:a.slice(24,49),n:20},
  {name:"underBack",path:a.slice(48,69),n:34},
  {name:"cuff",path:a.slice(68,70),n:14},
  {name:"underFront",path:[...a.slice(69),a[0]],n:34}
 ];
}
function area(poly){
 return poly.reduce((sum,p,i)=>{
  const q=poly[(i+1)%poly.length];
  return sum+p[0]*q[1]-q[0]*p[1];
 },0)*.5;
}
function triangles2D(poly) {
 const orientation=Math.sign(area(poly));
 if(!orientation)throw Error("Schnittteil hat Nullfläche.");
 const index=poly.map((_,i)=>i),tris=[];
 const cross=(a,b,c)=>{
  const p=poly[a],q=poly[b],r=poly[c];
  return (q[0]-p[0])*(r[1]-p[1])-(q[1]-p[1])*(r[0]-p[0]);
 };
 const inside=(a,b,c,p)=>{
  const v1=cross(a,b,p)*orientation,
    v2=cross(b,c,p)*orientation,v3=cross(c,a,p)*orientation;
  return v1>=-1e-8&&v2>=-1e-8&&v3>=-1e-8;
 };
 let iterations=0;
 while(index.length>3){
  let found=false;
  for(let i=0;i<index.length;i++){
   const a=index[(i+index.length-1)%index.length],
    b=index[i],c=index[(i+1)%index.length];
   if(cross(a,b,c)*orientation<=1e-6)continue;
   let blocked=false;
   for(const p of index){
    if(p!==a&&p!==b&&p!==c&&inside(a,b,c,p)){blocked=true;break;}
   }
   if(blocked)continue;
   tris.push([a,b,c]);index.splice(i,1);found=true;break;
  }
  if(!found)throw Error("Schnittkontur kann nicht trianguliert werden (Doppelpunkte/Selbstschnitt).");
  if(++iterations>3000)throw Error("Schnittkontur triangulierungslimit erreicht.");
 }
 tris.push(index);
 return tris;
}
function panel(piece,name,kind,sign,m){
 const sections=kind==="sleeve"?sleeveEdges(piece):bodyEdges(piece);
 const uv=[],edgeIds={},edgeLength={};
 for(const group of sections){
  const sampled=resample(group.path,group.n);
  edgeLength[group.name]=length(group.path);
  const start=uv.length;
  for(let j=0;j<group.n;j++)uv.push(sampled[j]);
  edgeIds[group.name]=Array.from({length:group.n+1},(_,j)=>start+j);
 }
 for(const edge of Object.values(edgeIds))
  edge[edge.length-1]%=uv.length;
 if(uv.some(p=>p.some(v=>!Number.isFinite(v))))throw Error("Schnittkontur enthält ungültige Koordinaten.");
 const tris=triangles2D(uv);
 const face=tris.map(t=>[...t]);
 // A unique interior node for each face improves drape resolution without
 // splitting the already perfectly paired sewing-boundary sampling.
 for(const t of tris){
  const c=uv.length;
  const center=[0,0];
  for(const i of t){center[0]+=uv[i][0]/3;center[1]+=uv[i][1]/3;}
  uv.push(center);
  face.push([t[0],t[1],c],[t[1],t[2],c],[t[2],t[0],c]);
 }
 // Remove the coarse spanning triangles: only smaller 3-way wedges remain.
 face.splice(0,tris.length);
 const front=kind==="front",back=kind==="back";
 const width=Math.max(...piece.outline.map(q=>q[0])),
  bust=(m.chest+m.ease)/(2*PI),waist=(m.waist+m.ease)/(2*PI),
  hip=(m.hip+m.ease)/(2*PI);
 const xyz=uv.map(([x,y])=>{
  if(kind==="sleeve"){
   const a=-PI+2*PI*x/width;
   const centerX=sign*(m.shoulder*.48+23+Math.max(0,y-90)*.11);
   const upperR=m.upperArm/(2*PI),wristR=m.wrist/(2*PI);
   const frac=clamp((y-100)/(m.sleeve-100),0,1);
   const radius=(upperR*(1-frac)+wristR*frac)+9;
   // Sleeve underarm apex is level with the REAL bodice armscye base.
   // It must NOT float 140 mm above the garment armhole as before.
   const cap=piece.points.bicepY;
   const worldY=y<=cap?215-200*y/cap:15-(y-cap)*.91;
   return pt(centerX+sign*radius*Math.sin(a),
     worldY,Math.cos(a)*radius);
  }
  const widthRate=clamp(x/width,0,1),theta=front?
   widthRate*PI*.5:PI-widthRate*PI*.5;
  const radius=y<330?bust:y<490?
   bust+(waist-bust)*(y-330)/160:y<680?
   waist+(hip-waist)*(y-490)/190:hip;
  // Near the upper shoulder front/back meet in the shoulder plane,
  // not 15 cm apart across the chest. The volume grows towards the bust.
  const shoulderDrop=clamp((y-piece.points.topY)/
    (piece.points.armY-piece.points.topY),0,1);
  return pt(sign*Math.sin(theta)*(radius+10),260-y,
    Math.cos(theta)*(radius+10)*.88*shoulderDrop);
 });
 return {name,kind,sign,uv,xyz,faces:face,edges:edgeIds,
  edgeLength,source:piece};
}
function profile(values,options={}){
 const mm={};
 for(const [k,fallback]of Object.entries({
  chest:1000,waist:840,hip:1020,shoulder:440,
  upperArm:340,wrist:190,sleeve:620,ease:80})){
  const raw=Number(values?.[k]??fallback);
  if(!Number.isFinite(raw)||raw<=0||raw>3500)throw Error("Körpermaß "+k+" ungültig.");
  const override=Number(options?.[k+"Mm"]??raw);
  if(!Number.isFinite(override)||override<=0||override>3500)
   throw Error("3D-Körpermaß "+k+" ungültig.");
  mm[k]=override;
 }
 if(options.neckMm!=null){
  const n=Number(options.neckMm);
  if(!Number.isFinite(n)||n<180||n>700)throw Error("3D-Halsumfang ungültig.");
  mm.neck=n;
 }else mm.neck=380;
 const rawHeight=Number(options.heightMm??1680);
 if(!Number.isFinite(rawHeight)||rawHeight<850||rawHeight>2400)
  throw Error("Körpergröße der Maßpuppe ist ungültig.");
 mm.height=rawHeight;
 return mm;
}
function seamsFor(panels,fit){
 const get=name=>{
  const p=panels.find(x=>x.name===name);
  if(!p)throw Error("Nähpartner fehlt: "+name);
  return p;
 };
 const pairs=[];
 const pair=(name,a,edgeA,b,edgeB,reverse=false,kind="match")=>{
  const pa=get(a),pb=get(b),ea=pa.edges[edgeA],eb=pb.edges[edgeB];
  if(!ea||!eb||ea.length!==eb.length)throw Error("Nähkanten können nicht paarweise verknüpft werden: "+name);
  const la=pa.edgeLength[edgeA],lb=pb.edgeLength[edgeB],delta=la-lb;
  const tolerance=1.5;
  const plannedEaseMm=kind==="cap"?
    Number(fit.capBalance?.targetEaseMm??0)*(edgeA==="capFront"?.4:.6):0;
  const mode=kind==="cap"&&delta>tolerance?"ease":
    kind==="cap"&&delta< -tolerance?"stretch":"match";
  const status=mode==="match"?Math.abs(delta)<=tolerance?"OK":"MISMATCH":
    mode==="ease"?(Math.abs(delta-plannedEaseMm)<=.6?
      "EINHALTEN_GEPLANT":"EINHALTEN_ERFORDERLICH"):"DEHNEN_ERFORDERLICH";
  pairs.push({id:name,a,edgeA,b,edgeB,mode,status,deltaMm:delta,
    lengthAMm:la,lengthBMm:lb,plannedEaseMm,
    plannedEaseTotalMm:fit.capBalance?.targetEaseMm??null,
    indexPairs:ea.map((v,i)=>[v,eb[reverse?eb.length-1-i:i]]),
    reverse});
 };
 for(const side of ["R","L"]){
  pair("Schulternaht "+side,"front"+side,"shoulder","back"+side,"shoulder");
  pair("Seitennaht "+side,"front"+side,"side","back"+side,"side");
  if(panels.some(p=>p.name==="sleeve"+side)){
    pair("Ärmel vorn "+side,"sleeve"+side,"capFront",
      "front"+side,"armhole",true,"cap");
    pair("Ärmel hinten "+side,"sleeve"+side,"capBack",
      "back"+side,"armhole",false,"cap");
    pair("Ärmel-Unterarm "+side,"sleeve"+side,"underBack",
      "sleeve"+side,"underFront",true);
  }
 }
 pair("Stoffbruch vorne","frontR","fold","frontL","fold");
 pair("Stoffbruch hinten","backR","fold","backL","fold");
 return pairs;
}
function stitchPanels(panels,contracts){
 const offsets=new Map(),vertices=[],uv=[],ids=[],faces=[],springRaw=[];
 for(const p of panels){
  offsets.set(p.name,vertices.length);
  for(let i=0;i<p.xyz.length;i++){
   vertices.push([...p.xyz[i]]);
   uv.push([...p.uv[i]]);
   ids.push(p.name);
  }
  for(const face of p.faces){
   const [a,b,c]=face.map(i=>offsets.get(p.name)+i);
   faces.push([a,b,c]);
   springRaw.push([a,b],[b,c],[c,a]);
  }
 }
 const parent=vertices.map((_,i)=>i);
 function find(i){while(parent[i]!==i){parent[i]=parent[parent[i]];i=parent[i];}return i;}
 function union(a,b){a=find(a);b=find(b);if(a!==b)parent[b]=a;}
 for(const seam of contracts){
  const oa=offsets.get(seam.a),ob=offsets.get(seam.b);
  for(const [a,b]of seam.indexPairs)union(oa+a,ob+b);
 }
 const roots=new Map(),final=[],groups=[];
 for(let i=0;i<vertices.length;i++){
  const root=find(i);
  if(!roots.has(root)){roots.set(root,final.length);final.push([0,0,0]);groups.push([]);}
  const newId=roots.get(root);groups[newId].push(i);
 }
 for(let i=0;i<final.length;i++){
  for(const j of groups[i]){
   for(let k=0;k<3;k++)final[i][k]+=vertices[j][k]/groups[i].length;
  }
 }
 const mapped=vertices.map((_,i)=>roots.get(find(i)));
 const triangles=faces.map(f=>f.map(i=>mapped[i])).filter(f=>new Set(f).size===3);
 const springMap=new Map();
 for(const [a,b]of springRaw){
  const x=mapped[a],y=mapped[b];if(x===y)continue;
  const resting=norm(uv[a],uv[b]);
  if(!(resting>1e-5)||!Number.isFinite(resting))continue;
  const key=x<y?x+":"+y:y+":"+x;
  // Multiple 2D panels may contribute different resting lengths after a
  // stitch weld: preserve all independent fabric constraints, don't average.
  const list=springMap.get(key)||[];
  if(!list.some(v=>Math.abs(v-resting)<.001))list.push(resting);
  springMap.set(key,list);
 }
 const springs=[];
 for(const [key,list]of springMap){
  const [a,b]=key.split(":").map(Number);
  for(const restMm of list)springs.push({a,b,restMm});
 }
 const edgeGroups=[];
 const panelColors={front:"#82d5df",back:"#74bfca",sleeve:"#e9b77f"};
 const colors=["#eacb93","#8ce3d8","#ecaa91"];
 for(const p of panels){
  for(const [edge,indices]of Object.entries(p.edges)){
   const valid=["neck","hem","cuff"].includes(edge);
   if(!valid)continue;
   edgeGroups.push({indices:indices.map(i=>mapped[offsets.get(p.name)+i]),
    color:edge==="neck"?"#eec78a":"#a3e6e3",width:1.6});
  }
 }
 for(const [i,seam]of contracts.entries()){
  if(seam.id.startsWith("Stoffbruch"))continue;
  const p=panels.find(x=>x.name===seam.a);
  edgeGroups.push({indices:p.edges[seam.edgeA].map(idx=>mapped[offsets.get(p.name)+idx]),
    color:colors[i%colors.length],width:2.3});
 }
 const joined=contracts.every(seam=>{
  const a=offsets.get(seam.a),b=offsets.get(seam.b);
  return seam.indexPairs.every(([i,j])=>mapped[a+i]===mapped[b+j]);
 });
 if(!joined)throw Error("3D-Nähpartner wurden nicht zusammengefügt.");
 return {vertices:final,faces:triangles,edgeGroups,springs,
  mapping:mapped,offsets,panelCount:panels.length,contracts,groups,
  stitched:joined};
}
function createBody(mm){
 const models=[];
 function create(name,color,rings,fullBodyOnly=false){
  const mesh={name,color,body:true,fullBodyOnly,vertices:[],faces:[],edgeGroups:[]};
  const N=28,rows=[];
  for(const [y,rx,rz,x=0,z=0]of rings){
   const ring=[];
   for(let i=0;i<=N;i++){
    const t=i*2*PI/N;
    ring.push(mesh.vertices.push(pt(x+rx*Math.cos(t),y,z+rz*Math.sin(t)))-1);
   }
   rows.push(ring);
  }
  for(let i=0;i<rows.length-1;i++)for(let j=0;j<N;j++){
   const a=rows[i][j],b=rows[i][j+1],c=rows[i+1][j],d=rows[i+1][j+1];
   mesh.faces.push([a,b,c],[b,d,c]);
  }
  models.push(mesh);
 }
 const chest=mm.chest/(2*PI),waist=mm.waist/(2*PI),hip=mm.hip/(2*PI);
 create("Maßpuppe · Körper","#9f9b99",[
  [-650,hip*.90,hip*.70],[-430,hip*.98,hip*.84],
  [-220,waist*.98,waist*.83],[10,chest, chest*.82],
  [170,chest*.84,chest*.67],[210,mm.neck/(2*PI)*1.2,mm.neck/(2*PI)]]);
 const scale=mm.height/1680;
 const skullCirc=Math.max(490,Math.min(650,mm.neck*1.5)); // visually estimated until head is measured
 const skull=skullCirc/(2*PI);
 create("Maßpuppe · Kopf (geschätzt)","#b2a59a",[
  [200,mm.neck/(2*PI),mm.neck/(2*PI)*.83],
  [253,skull*.83,skull*.75],
  [315,skull,skull*.94],[380,skull*.78,skull*.76],
  [422,skull*.16,skull*.20]]);
 const upper=mm.upperArm/(2*PI),wrist=mm.wrist/(2*PI);
 for(const side of [-1,1]){
  const x=mm.shoulder*.55;
  create("Maßpuppe · Arm "+side,"#a9a49e",[
   [-410,wrist,wrist*.75,side*(x+65),0],
   [-220,upper*.79,upper*.72,side*(x+45),0],
   [30,upper*.99,upper*.88,side*(x+14),0],
   [210,upper*.85,upper*.83,side*x,0]
  ]);
  const hipSide=mm.hip/(2*PI)*.51;
  create("Maßpuppe · Bein "+side,"#a69b92",[
    [-650*scale,mm.hip*.072,mm.hip*.064,side*hipSide,-3],
    [-900*scale,mm.hip*.050,mm.hip*.048,side*hipSide,12],
    [-1200*scale,mm.hip*.042,mm.hip*.045,side*hipSide,5],
    [-1510*scale,mm.hip*.029,mm.hip*.035,side*hipSide,10],
    [-1540*scale,mm.hip*.027,mm.hip*.032,side*hipSide,12]
  ],true);
 }
 return models;
}
function physics(sewn,mm,material){
 const vertices=sewn.vertices.map(v=>v.slice());
 const preset={
  cotton:{stiffness:.53,bending:.30,drape:.58,clearance:9,color:"#65c5d1"},
  jersey:{stiffness:.27,bending:.13,drape:.93,clearance:7,color:"#4ab7be"},
  denim:{stiffness:.80,bending:.65,drape:.22,clearance:13,color:"#7694be"},
  shell:{stiffness:.62,bending:.40,drape:.48,clearance:11,color:"#c3a4d3"}
 };
 const type=material.preset||"cotton",p=preset[type];
 if(!p)throw Error("Unbekannter Stofftyp.");
 const num=(key,fallback,a,b)=>{
  const v=Number(material[key]??fallback);
  if(!Number.isFinite(v)||v<a||v>b)throw Error("Stoffparameter "+key+" außerhalb "+a+"–"+b);
  return v;
 };
 const stiffness=num("stiffness",p.stiffness,.03,1),
  bending=num("bending",p.bending,.02,1),
  drape=num("drape",p.drape,0,2),
  clearance=num("clearanceMm",p.clearance,2,35),
  iterations=Math.round(num("iterations",22,2,50));
 const blocked=new Set();
 const owner=new Array(vertices.length).fill("front");
 for(let i=0;i<sewn.groups.length;i++){
   const original=sewn.groups[i][0];
   let name="front";
   for(const [panelName,offset]of sewn.offsets){
     if(offset<=original)name=panelName;else break;
   }
   owner[i]=name.startsWith("sleeve")?name:"torso";
 }
 // Keep only a few neckline anchors so gravity makes the rest fall.
 for(const p of sewn.edgeGroups){
  if(p.color!=="#eec78a")continue;
  const step=Math.max(1,Math.floor(p.indices.length/5));
  for(let i=0;i<p.indices.length;i+=step)blocked.add(p.indices[i]);
 }
 const torso=(x,y,z)=>{
  // The head/neck opening is free. A chest-radius collider must not inflate
  // neck and shoulder sewing lines; only the chest and waist are constrained.
  if(y<-580||y>55)return [x,y,z];
  const radius=y>0?mm.chest/(2*PI):
   y>-240?mm.chest/(2*PI)+(mm.waist-mm.chest)/(2*PI)*(-y/240):
   mm.waist/(2*PI)+(mm.hip-mm.waist)/(2*PI)*clamp((-y-240)/280,0,1);
  const rx=radius+clearance,rz=radius*.83+clearance;
  const level=Math.hypot(x/rx,z/rz);
  if(level>=1)return [x,y,z];
  const ratio=1/Math.max(level,1e-7);
  return [x*ratio,y,z*ratio];
 };
 const sleeveCollision=(x,y,z,sign)=>{
   if(y>210||y<-540)return [x,y,z];
   const fraction=clamp((210-y)/550,0,1);
   const centerX=sign*(mm.shoulder*.49+15+fraction*62);
   const r=(mm.upperArm/(2*PI)*(1-fraction)+
      mm.wrist/(2*PI)*fraction)+clearance;
   const dx=x-centerX,dz=z,span=Math.hypot(dx,dz);
   if(span>=r)return [x,y,z];
   const angle=span>1e-6?1/span:0;
   return [centerX+(angle?dx*angle*r:sign*r),y,
     angle?dz*angle*r:0];
 };
 // Build multiple equal-area/edge springs from the flat panel distances.
 const work=sewn.springs;
 let contacts=0;
 for(let i=0;i<iterations;i++){
  for(let j=0;j<vertices.length;j++)if(!blocked.has(j)){
   vertices[j][1]-=drape*.8;
  }
  for(const s of work){
   const a=vertices[s.a],b=vertices[s.b];
   const dx=b[0]-a[0],dy=b[1]-a[1],dz=b[2]-a[2];
   const n=Math.hypot(dx,dy,dz);
   if(n<1e-8)continue;
   const strength=clamp((n-s.restMm)/n*stiffness*.75,-.32,.32);
   const w1=blocked.has(s.a)?0:1,w2=blocked.has(s.b)?0:1,normW=w1+w2;
   if(!normW)continue;
   for(let k=0;k<3;k++){
    const shift=(b[k]-a[k])*strength;
    a[k]+=shift*w1/normW;b[k]-=shift*w2/normW;
   }
  }
  // A small locally averaged bending response prevents randomly buckled
  // triangles; these values are demonstrative, not calibrated textile data.
  if(bending>.02&&i%2===0){
   for(const s of work.filter((_,n)=>n%7===0)){
    const a=vertices[s.a],b=vertices[s.b];if(blocked.has(s.a)||blocked.has(s.b))continue;
    const smooth=bending*.008;
    for(let k=0;k<3;k++){const n=(a[k]+b[k])*.5;a[k]+=smooth*(n-a[k]);b[k]+=smooth*(n-b[k]);}
   }
  }
  for(let j=0;j<vertices.length;j++)if(!blocked.has(j)){
   const a=vertices[j],part=owner[j];
   const b=part==="torso"?torso(...a):
     sleeveCollision(...a,part.endsWith("R")?1:-1);
   if(a[0]!==b[0]||a[2]!==b[2]){
    a[0]=b[0];a[2]=b[2];contacts++;
   }
  }
 }
 const strains=work.map(s=>{
  const a=vertices[s.a],b=vertices[s.b],
   len=Math.hypot(b[0]-a[0],b[1]-a[1],b[2]-a[2]);
  return Math.abs(len/s.restMm-1);
 });
 const rms=Math.sqrt(strains.reduce((a,b)=>a+b*b,0)/Math.max(1,strains.length));
 const max=Math.max(0,...strains);
 return {vertices,rmsPct:rms*100,maxPct:max*100,contacts,color:p.color};
}
function construct(draft,body={},fabric={}){
 const allPieces=draft?.construction?.pieces||draft?.pieces;
 const showSleeves=draft?.profile?.style?.sleeves!==false&&
   draft?.parts?.some(p=>p.name?.includes("Ärmel"))!==false;
 const pieces=showSleeves?allPieces:allPieces?.slice(0,2);
 if(!Array.isArray(pieces)||pieces.length<2)
  throw Error("2D-Grundschnitt für vernähte 3D-Ansicht fehlt.");
 const sourceMeasure=draft?.construction?.measurements||draft?.measurements||{};
 const garmentMm=profile(sourceMeasure);
 const bodyMm=profile(sourceMeasure,body);
 const panels=[];
 for(const side of ["R","L"]){
  const sign=side==="R"?1:-1;
  panels.push(panel(pieces[0],"front"+side,"front",sign,garmentMm));
  panels.push(panel(pieces[1],"back"+side,"back",sign,garmentMm));
  if(pieces.length>=3)panels.push(panel(pieces[2],"sleeve"+side,"sleeve",sign,garmentMm));
 }
 const complete=pieces.length>=3;
 // A sleeveless shirt intentionally leaves its armscyes open; no artificial sleeves.
 const contracts=seamsFor(panels,draft);
 const stitched=stitchPanels(panels,contracts);
 const relaxed=physics(stitched,bodyMm,fabric);
 const cloth={name:"Oberteil · vier Rumpf-Hälften + zwei Ärmel",
  color:relaxed.color,body:false,vertices:relaxed.vertices,
  faces:stitched.faces,edgeGroups:stitched.edgeGroups};
 const unexpected=contracts.filter(c=>c.status!=="OK");
 const metric={
  seamCount:contracts.filter(c=>!c.id.startsWith("Stoffbruch")).length,
  foldPairs:contracts.filter(c=>c.id.startsWith("Stoffbruch")).length,
  seamPairs:contracts.map(({id,a,edgeA,b,edgeB,status,mode,deltaMm,
     lengthAMm,lengthBMm,plannedEaseTotalMm,plannedEaseMm})=>({
     id,a,edgeA,b,edgeB,status,mode,deltaMm,lengthAMm,
     lengthBMm,plannedEaseTotalMm,plannedEaseMm,
     connected:true
  })),
  sewnEdgePairs:contracts.filter(c=>!c.id.startsWith("Stoffbruch")).length,
  missingSeamPartners:0,
  welded:true,centerSeamGapMm:0,
  panelCount:panels.length,
  triangles:cloth.faces.length,vertices:cloth.vertices.length,
  rmsEdgeStrainPct:relaxed.rmsPct,maxEdgeStrainPct:relaxed.maxPct,
  collisionCorrections:relaxed.contacts,
  seamFitVerified:unexpected.length===0,
  pendingProcessing:unexpected.length,
  openEdges:complete?["Halsausschnitt","Saum","Ärmelbündchen"]:
    ["Halsausschnitt","Saum","offene Armlöcher (ärmellos)"],
  unclosedDarts:["Brust-/Schulterabnäher","Taillenabnäher"],
  approximation:true,
  validForProduction:false,
  fitRisk:unexpected.length||relaxed.maxPct>25?"HIGH_STRAIN":"UNVALIDATED"
 };
 if(relaxed.vertices.some(v=>v.some(x=>!Number.isFinite(x))))
  throw Error("Stoffsimulation liefert ungültige Körperkoordinaten.");
 cloth.drapeMetrics=metric;
 const model=createBody(bodyMm);
 return {meshes:[...model,cloth],metrics:metric,body:bodyMm,contracts,
  topology:{panelNames:panels.map(p=>p.name),panels,stitched},fabric};
}
const api={construct,bodyEdges,sleeveEdges,resample,triangles2D,profile,seamsFor};
if(typeof module!=="undefined"&&module.exports)module.exports=api;
root.TextilLabTopSewing=api;
})(typeof globalThis!=="undefined"?globalThis:this);

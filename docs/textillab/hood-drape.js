/* SolidCraft TextilLab / Hood Draping Lab 1.4
 * The actual 2D stitching edges are remeshed to two polygonal panels.
 * Both center seams SHARE vertex indices; they cannot visually drift apart.
 * A bounded position-based mass-spring cloth relaxation is applied with
 * a parametric head collider and measured 2D spring rest lengths.
 *
 * This is an interactive mechanical approximation, NOT validated textile
 * mechanics, drape, pattern fit, or professional garment CAD simulation.
 */
(function(root){"use strict";
const PI=Math.PI,clamp=(v,a,b)=>Math.min(b,Math.max(a,v));
const P=(x,y,z)=>[x,y,z];
const diff=(a,b)=>[a[0]-b[0],a[1]-b[1],a[2]-b[2]];
const dist3=(a,b)=>Math.hypot(a[0]-b[0],a[1]-b[1],a[2]-b[2]);
function length(points){return points.slice(1).reduce((sum,v,i)=>sum+
 Math.hypot(v[0]-points[i][0],v[1]-points[i][1]),0);}
function sample(points,t){
 if(!Array.isArray(points)||points.length<2)throw Error("Hood Drape: Schnittkurve fehlt.");
 t=clamp(t,0,1);
 const total=length(points);if(!(total>1))throw Error("Hood Drape: ungültige Schnittkante.");
 let rem=total*t;
 for(let i=1;i<points.length;i++){
  const a=points[i-1],b=points[i],len=Math.hypot(b[0]-a[0],b[1]-a[1]);
  if(rem<=len||i===points.length-1){
   const ratio=len?clamp(rem/len,0,1):0;
   return [a[0]+(b[0]-a[0])*ratio,a[1]+(b[1]-a[1])*ratio];
  }
  rem-=len;
 }
 return [...points.at(-1)];
}
function blend(a,b,t){return [a[0]*(1-t)+b[0]*t,a[1]*(1-t)+b[1]*t];}
function coons(center,neck,face,u,v){
 // Transfinite interpolation: exactly interpolates center stitch line
 // (v=0), neckline (v=1), face opening (u=1) and rear tip (u=0).
 const c=sample(center,u),n=sample(neck,u),f=sample(face,v);
 const end=blend(center.at(-1),neck.at(-1),v);
 const p=blend(c,n,v);
 return [p[0]+u*(f[0]-end[0]),p[1]+u*(f[1]-end[1])];
}
function modelBody(source={},params={}){
 const mm=(key,fallback,low,high)=>{
  const n=Number(params[key]??fallback);
  if(!Number.isFinite(n)||n<low||n>high)throw Error("3D-Körpermaß "+key+" außerhalb "+low+"–"+high+" mm.");
  return n;
 };
 const profile=source.profile||"women";
 if(!["women","men","child"].includes(profile))throw Error("Unbekanntes Körperprofil.");
 const defaults=profile==="child"?
  {height:1330,shoulder:320,chest:680,waist:570,hip:740,neck:330}:
  profile==="men"?
  {height:1780,shoulder:460,chest:1040,waist:890,hip:1020,neck:400}:
  {height:1680,shoulder:410,chest:940,waist:750,hip:990,neck:350};
 const head=Number(source.headCircMm);
 if(!Number.isFinite(head)||head<360||head>720)throw Error("Kopfumfang der 3D-Puppe fehlt.");
 return {
  profile,headCircMm:head,
  heightMm:mm("heightMm",defaults.height,700,2250),
  shoulderMm:mm("shoulderMm",defaults.shoulder,220,650),
  chestMm:mm("chestMm",defaults.chest,420,1700),
  waistMm:mm("waistMm",defaults.waist,390,1700),
  hipMm:mm("hipMm",defaults.hip,420,1900),
  neckMm:mm("neckMm",defaults.neck,200,590)
 };
}
function clothOptions(data={}){
 const num=(key,fallback,a,b)=>{
  const v=Number(data[key]??fallback);
  if(!Number.isFinite(v)||v<a||v>b)throw Error("Stoffparameter "+key+" außerhalb "+a+"–"+b);
  return v;
 };
 const preset=data.preset||"cotton";
 const presets={
  cotton:{stiffness:.53,bending:.30,drape:.58,clearance:9,color:"#65c5d1"},
  jersey:{stiffness:.27,bending:.13,drape:.93,clearance:7,color:"#4ab7be"},
  denim:{stiffness:.80,bending:.65,drape:.22,clearance:13,color:"#7694be"},
  shell:{stiffness:.62,bending:.40,drape:.48,clearance:11,color:"#c3a4d3"}
 };
 if(!Object.hasOwn(presets,preset))throw Error("Unbekannte Stoffart.");
 const base=presets[preset];
 return {
  preset,stiffness:num("stiffness",base.stiffness,.03,1),
  bending:num("bending",base.bending,.02,1),
  drape:num("drape",base.drape,0,2),
  clearanceMm:num("clearanceMm",base.clearance,2,35),
  color:base.color,
  iterations:Math.round(num("iterations",24,2,60))
 };
}
function createSphere(name,color,center,radii,segments=26,latitudes=14){
 const m={name,color,body:true,vertices:[],faces:[],edgeGroups:[]};
 const rings=[];
 for(let j=0;j<=latitudes;j++){
  const theta=PI*j/latitudes;
  const row=[];
  for(let k=0;k<=segments;k++){
   const phi=2*PI*k/segments;
   row.push(m.vertices.push(P(
    center[0]+radii[0]*Math.sin(theta)*Math.cos(phi),
    center[1]+radii[1]*Math.cos(theta),
    center[2]+radii[2]*Math.sin(theta)*Math.sin(phi)))-1);
  }
  rings.push(row);
 }
 for(let j=0;j<latitudes;j++)
  for(let k=0;k<segments;k++)
   m.faces.push([rings[j][k],rings[j+1][k],rings[j][k+1]],
     [rings[j][k+1],rings[j+1][k],rings[j+1][k+1]]);
 return m;
}
function bodyMeshes(body,shape){
 const r=body.headCircMm/(2*PI);
 const h=shape.heightMm,depth=shape.depthMm;
 // The mannequin is positioned so that its skull clears the 2D top seam,
 // and the neck sits below the sewn neckline. Coordinates are in millimeters.
 const headCenter=P(0,h*.50,0);
 const headRadii=P(r*.94,Math.min(h*.36,r*1.30),r*.94);
 const head=createSphere("Parametrischer Kopf","#baa99a",headCenter,headRadii);
 const neck=createSphere("Hals / Maßpuppe","#b3a393",P(0,-24,-7),
  P(body.neckMm/(2*PI),77,body.neckMm/(2*PI)*.87),20,10);
 const shoulderWidth=body.shoulderMm*.5;
 const torso=createSphere("Schultern / Torso","#a99c91",P(0,-118,-12),
  P(shoulderWidth,105,body.chestMm/(2*PI)*.70),32,16);
 const waist=createSphere("Taille / Hüfte","#9d938a",P(0,-300,-12),
  P(body.waistMm/(2*PI)*1.05,150,body.hipMm/(2*PI)*.60),24,12);
 return {meshes:[head,neck,torso,waist],head:{center:headCenter,radii:headRadii}};
}
function construct(hood,bodyData={},fabricData={}){
 if(!hood?.parts?.[0]?.edges)throw Error("2D-Kapuzenkanten fehlen: 3D-Fertignähte werden nicht erfunden.");
 if(hood.checks?.some(c=>c.status!=="ok"))
  throw Error("Nahtlängen müssen vor der 3D-Anprobe übereinstimmen.");
 const edges=hood.parts[0].edges;
 for(const key of ["center","face","neck"])
  if(!Array.isArray(edges[key])||edges[key].length<2||
    edges[key].some(q=>q.length!==2||!q.every(Number.isFinite)))
    throw Error("Ungültige 2D-Nählinie: "+key);
 const cl=length(edges.center),nl=length(edges.neck),
  fl=length(edges.face);
 const epsilon=.12;
 for(const [a,b]of [[edges.center[0],edges.neck[0]],
  [edges.center.at(-1),edges.face[0]],
  [edges.neck.at(-1),edges.face.at(-1)]]){
  if(Math.hypot(a[0]-b[0],a[1]-b[1])>epsilon)
   throw Error("3D: Kurven-Endpunkte sind nicht verbunden.");
 }
 const parameters=modelBody({profile:hood.profile,headCircMm:hood.values.headCirc},bodyData);
 const material=clothOptions(fabricData);
 const body=bodyMeshes(parameters,hood.model);
 const nu=36,nv=14,frontLength=hood.values.hoodDepth;
 const mesh={name:"Kapuze · 2 Schnittteile, Mittelnaht verschweißt",
  color:material.color,body:false,vertices:[],faces:[],edgeGroups:[]};
 const uv=[],pinned=[],positions=[],sides=[],grid=[[],[]],sharedSeam=[];
 const add=(pointUV,pos,pin,side)=>{
  const id=positions.length;uv.push(pointUV);positions.push(pos);
  pinned.push(pin);sides.push(side);return id;
 };
 const start=coons(edges.center,edges.neck,edges.face,0,0);
 const rear=add(start,P(0,-start[1],start[0]*.72-frontLength*.28),true,0);
 sharedSeam.push(rear);
 for(let i=1;i<=nu;i++){
  const t=i/nu,point=sample(edges.center,t);
  sharedSeam.push(add(point,P(0,-point[1],point[0]*.72-frontLength*.28),true,0));
 }
 for(const [sideIndex,sign]of [-1,1].entries()){
  const field=grid[sideIndex];
  for(let i=0;i<=nu;i++){
   field[i]=[];
   for(let j=0;j<=nv;j++){
    if(i===0){field[i][j]=rear;continue;}
    if(j===0){field[i][j]=sharedSeam[i];continue;}
    const u=i/nu,v=j/nv,pt=coons(edges.center,edges.neck,edges.face,u,v);
    const spread=clamp(Math.sin(v*PI*.5)*Math.pow(u,.56),0,1);
    const headRadius=parameters.headCircMm/(2*PI);
    const sideX=sign*(headRadius*.97+material.clearanceMm)*
      spread*(.96+.06*Math.sin(PI*u));
    const position=P(sideX,-pt[1],pt[0]*.72-frontLength*.28);
    const isNeck=j===nv;
    const anchored=isNeck&&i%2===0;
    field[i][j]=add(pt,position,anchored,sign);
   }
  }
  for(let i=0;i<nu;i++)for(let j=0;j<nv;j++){
   const a=field[i][j],b=field[i+1][j],c=field[i][j+1],d=field[i+1][j+1];
   if(a===c)mesh.faces.push([a,b,d]);
   else {mesh.faces.push([a,b,c]);mesh.faces.push([b,d,c]);}
  }
 }
 mesh.vertices=positions;
 const centerIds=sharedSeam.slice();
 const openings=grid.map(g=>Array.from({length:nv+1},(_,j)=>g[nu][j]));
 const necks=grid.map(g=>Array.from({length:nu+1},(_,i)=>g[i][nv]));
 mesh.edgeGroups=[
  {indices:centerIds,color:"#f4ce89",width:2.4},
  ...openings.map(row=>({indices:row,color:"#86e5dc",width:1.7})),
  ...necks.map(row=>({indices:row,color:"#f2a788",width:1.6}))
 ];
 const rest=positions.map(p=>p.slice());
 const springSet=new Map();
 const addSpring=(a,b,stiffness)=>{
  if(a===b)return;
  const key=a<b?a+":"+b:b+":"+a;
  if(springSet.has(key))return;
  const restMm=Math.hypot(uv[a][0]-uv[b][0],uv[a][1]-uv[b][1]);
  if(!(restMm>1e-8)||!Number.isFinite(restMm))return;
  springSet.set(key,{a,b,restMm,stiffness});
 };
 for(const face of mesh.faces){
  addSpring(face[0],face[1],material.stiffness);
  addSpring(face[1],face[2],material.stiffness);
  addSpring(face[2],face[0],material.stiffness);
 }
 for(const field of grid){
  for(let i=0;i<nu-1;i++)for(let j=0;j<=nv;j++)
   addSpring(field[i][j],field[i+2][j],material.bending*.6);
  for(let i=0;i<=nu;i++)for(let j=0;j<nv-1;j++)
   addSpring(field[i][j],field[i][j+2],material.bending*.6);
 }
 const springs=[...springSet.values()];
 const pos=positions;
 const steps=material.iterations;
 let contacts=0;
 const collision=(id)=>{
  if(pinned[id])return;
  const point=pos[id],center=body.head.center,radii=body.head.radii,
    clearance=material.clearanceMm;
  const scaled=radii.map(v=>v+clearance);
  const dx=(point[0]-center[0])/scaled[0],
    dy=(point[1]-center[1])/scaled[1],
    dz=(point[2]-center[2])/scaled[2],rad=Math.hypot(dx,dy,dz);
  if(rad>=1)return;
  const inv=1/Math.max(rad,1e-6);
  const normal=rad>1e-6?[dx*inv,dy*inv,dz*inv]:[0,1,0];
  point[0]=center[0]+normal[0]*scaled[0];
  point[1]=center[1]+normal[1]*scaled[1];
  point[2]=center[2]+normal[2]*scaled[2];
  contacts++;
 };
 for(let iteration=0;iteration<steps;iteration++){
  const gravity=material.drape*.95;
  for(let id=0;id<pos.length;id++)if(!pinned[id])pos[id][1]-=gravity;
  for(const spring of springs){
   const {a,b,restMm,stiffness}=spring,pa=pos[a],pb=pos[b];
   const delta=diff(pb,pa),len=dist3(pa,pb);
   if(!(len>1e-8))continue;
   const factor=clamp((len-restMm)/len*stiffness,-.55,.55);
   const wa=pinned[a]?0:1,wb=pinned[b]?0:1,weight=wa+wb;
   if(!weight)continue;
   for(let k=0;k<3;k++){
    pa[k]+=delta[k]*factor*wa/weight;
    pb[k]-=delta[k]*factor*wb/weight;
   }
  }
  for(let id=0;id<pos.length;id++)collision(id);
 }
 const distortion=springs.filter(s=>s.stiffness===material.stiffness).map(s=>{
  const len=dist3(pos[s.a],pos[s.b]);
  return Math.abs(len/s.restMm-1);
 });
 const rms=Math.sqrt(distortion.reduce((sum,v)=>sum+v*v,0)/
   Math.max(1,distortion.length));
 const max=Math.max(0,...distortion);
 const faceCount=mesh.faces.length,uniqueCenter=centerIds.every((id,i)=>
   grid[0][i][0]===id&&grid[1][i][0]===id);
 const weldGapMm=Math.max(...centerIds.map((id,i)=>
   dist3(pos[grid[0][i][0]],pos[grid[1][i][0]])));
 const validity=Number.isFinite(rms)&&Number.isFinite(max)&&mesh.vertices.every(p=>p.every(Number.isFinite));
 if(!validity||!uniqueCenter||weldGapMm>1e-6)
  throw Error("Kapuzen-Drapierung ist numerisch fehlerhaft oder nicht vernäht.");
 const metrics={
  joinedCenterSeam:true,centerSeamGapMm:weldGapMm,
  actualSeamMm:cl,neckSeamMm:nl,faceOpeningMm:fl,
  triangles:faceCount,vertexCount:mesh.vertices.length,
  springCount:springs.length,collisionCorrections:contacts,
  rmsEdgeStrainPct:rms*100,maxEdgeStrainPct:max*100,
  approximation:true,validForProduction:false
 };
 mesh.drapeMetrics=metrics;
 return {meshes:[...body.meshes,mesh],mannequin:parameters,material,metrics,
  patternPanelEdges:edges,topology:{centerIds,grid,uv,pinned},
  // A texture-to-mesh mapping and true sewn seam are available for audit.
  seamSegments:{center:[centerIds,centerIds],neck:necks,face:openings}
 };
}
const api={construct,modelBody,clothOptions,coons,sample,length};
if(typeof module!=="undefined"&&module.exports)module.exports=api;
root.TextilLabHoodDrape=api;
})(typeof globalThis!=="undefined"?globalThis:this);

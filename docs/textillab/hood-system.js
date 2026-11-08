/* TextilLab HoodLab v1.3.
 * Openly published numeric Müller & Sohn hood relations are recorded with
 * provenance. Original geometry, Bézier transitions and 3D draping are
 * independent approximations, not licensed reproductions or a verified block.
 * All internal dimensions in millimetres, UI input in centimetres.
 */
(function(root){
"use strict";
const SOURCE="https://www.muellerundsohn.com/allgemein/schnittkonstruktion-kapuze/";
const Contracts=(typeof module!=="undefined"&&module.exports)?require("./seam-contracts.js"):root.TextilLabSeamContracts;
const VARIANTS=Object.freeze({
 close:{name:"Anliegend",minHeight:30,maxHeight:34,extraDepth:0,publication:true},
 relaxed:{name:"Aufgelockert",minHeight:30,maxHeight:36,extraDepth:4.5,publication:true},
 dartless:{name:"Abnäherlos",minHeight:35,maxHeight:35,extraDepth:3,publication:true}
});
const PROFILES=Object.freeze({
 women:{label:"Damen / DOB",sourceVerified:true,group:"DOB",
  example:{headCirc:57,backNeck:10,frontNeck:12,hoodHeight:32,hoodDepth:27}},
 men:{label:"Herren / HAKA",sourceVerified:false,group:"HAKA",
  example:{headCirc:59,backNeck:10.5,frontNeck:12.5,hoodHeight:34,hoodDepth:28}},
 child:{label:"Kinder",sourceVerified:false,group:"Kinder",
  example:{headCirc:52,backNeck:8,frontNeck:9.5,hoodHeight:28,hoodDepth:24,ageYears:8}}
});
const DEFAULTS=Object.freeze({
 headCirc:57,backNeck:10,frontNeck:12,neckWidth:8,
 frontNeckDrop:5,hoodHeight:32,hoodDepth:27,
 dartIntake:3.5,faceOverlap:0,seamAllowance:1,materialDensity:210
});
const SOURCE_RELATIONS=Object.freeze([
 {id:"M01",label:"Anfangshöhe",formula:"Halsbreite + vordere Halsabsenkung",status:"public-number",depends:["neckWidth","frontNeckDrop"]},
 {id:"M02",label:"Anliegende Kapuzenhöhe",formula:"30 bis 34 cm",status:"public-number",depends:["hoodHeight"]},
 {id:"M03",label:"Anliegende Kapuzentiefe",formula:"26 bis 28 cm",status:"public-number",depends:["hoodDepth"]},
 {id:"M04",label:"Konstruktionsversatz",formula:"1,5 cm",status:"public-number"},
 {id:"M05",label:"Hintere Halsansatzstrecke (Referenz)",formula:"Halsloch hinten minus 0,5 cm",status:"public-number",depends:["backNeck"]},
 {id:"M06",label:"Abnäherreserve / Falte",formula:"3 bis 4 cm",status:"public-number",depends:["dartIntake"]},
 {id:"M07",label:"Zusätzliche Versatzstrecken",formula:"jeweils 2 cm",status:"public-number"},
 {id:"M08",label:"Vordere Halsansatzstrecke",formula:"Länge des Vorderhalslochs",status:"public-number",depends:["frontNeck"]},
 {id:"M09",label:"Lockerungszugabe",formula:"4 bis 5 cm an der vertikalen Ausgangsstrecke",status:"public-number",depends:["variant"]},
 {id:"M10",label:"Abnäherlose Kapuze",formula:"Kapuzenhöhe 35 cm, Tiefe ca. 31 cm, Vorderübertritt 2 cm",status:"public-number",depends:["variant"]},
 {id:"M11",label:"Halslochänderung bei abnäherloser Form",formula:"Vordere Mitte +2 cm, Schulter +1 cm, hintere Mitte +0,5 cm",status:"public-number",depends:["variant"]},
 {id:"M12",label:"Abnäherlose Kapuzenrundung",formula:"Referenzabstand 5 bis 6 cm",status:"public-number",depends:["variant"]},
 {id:"M13",label:"Abnäherlose Hilfspunktabstände",formula:"1,5 cm abwärts, 9 cm aufwärts, 4 cm seitwärts",status:"public-number",depends:["variant"]},
 {id:"TL11",label:"Geometrischer Nahtabgleich",formula:"Hood seam arc length = garment neckline arc length",status:"textillab-independent"}
]);
function finite(cm,label,min,max){
 const n=Number(cm);
 if(cm==null||String(cm).trim()===""||!Number.isFinite(n)||n<min||n>max)
  throw Error(label+" muss zwischen "+min+" und "+max+" cm liegen.");
 return n*10;
}
const sub=(a,b)=>[a[0]-b[0],a[1]-b[1]];
const add=(a,b)=>[a[0]+b[0],a[1]+b[1]];
const dist=(a,b)=>Math.hypot(a[0]-b[0],a[1]-b[1]);
const point=(x,y)=>[x,y];
const fmt=n=>Number(n.toFixed(3));
const svgText=x=>String(x).replace(/[&<>"']/g,c=>({"&":"&amp;","<":"&lt;",">":"&gt;",'"':"&quot;","'":"&apos;"}[c]));
function sampleCubic(a,b,c,d,steps=40){
 const pts=[];
 for(let i=0;i<=steps;i++){
  const t=i/steps,u=1-t;
  pts.push([u*u*u*a[0]+3*u*u*t*b[0]+3*u*t*t*c[0]+t*t*t*d[0],
   u*u*u*a[1]+3*u*u*t*b[1]+3*u*t*t*c[1]+t*t*t*d[1]]);
 }
 return pts;
}
const arcLength=arr=>arr.reduce((n,p,i)=>n+(i?dist(p,arr[i-1]):0),0);
const polygonArea=p=>Math.abs(p.reduce((n,a,i)=>{const b=p[(i+1)%p.length];return n+a[0]*b[1]-b[0]*a[1]},0)/2);
function neckSegment(a,target,side){
 const bump=side==="back"?6:8,deltaY=side==="back"?8:-8;
 const curve=dx=>{
  const d=point(a[0]+dx,a[1]+deltaY);
  return sampleCubic(a,point(a[0]+dx*.25,a[1]+bump),
    point(a[0]+dx*.70,d[1]+bump),d);
 };
 let l=0,r=target;
 for(let k=0;k<45;k++){
  const mid=(l+r)/2;
  if(arcLength(curve(mid))>target)r=mid;else l=mid;
 }
 const p=curve((l+r)/2);
 return {points:p,length:arcLength(p),end:p.at(-1)};
}
function parse(values={}){
 const variant=String(values.variant||"close");
 const profile=String(values.profile||"women");
 if(!VARIANTS[variant])throw Error("Nicht unterstützte Kapuzenform.");
 if(!PROFILES[profile])throw Error("Bitte Damen, Herren oder Kinder auswählen.");
 const v={};
 for(const [k,def]of Object.entries(DEFAULTS)){
  const limits=k==="headCirc"?[36,70]:
   (k==="backNeck"||k==="frontNeck")?[5,25]:
   k==="hoodHeight"?[22,45]:
   k==="hoodDepth"?[18,42]:
   k==="neckWidth"?[3,15]:
   k==="frontNeckDrop"?[0,14]:
   k==="dartIntake"?[0,7]:
   k==="faceOverlap"?[0,5]:
   k==="seamAllowance"?[0,3]:
   k==="materialDensity"?[3,500]:
   [0,500];
  v[k]=k==="materialDensity"
    ? Number(values[k]??def)
    : finite(values[k]??PROFILES[profile].example[k]??def,k,...limits);
  if(k==="materialDensity" && (!Number.isFinite(v[k])||v[k]<3||v[k]>500))
    throw Error("Flächengewicht muss zwischen 3 und 500 g/m² liegen.");
 }
 if(profile==="child"&&v.headCirc>590)
   throw Error("Kinderprofil: Kopfumfang überprüfen. Größen-/Altersregeln sind noch nicht fachlich validiert.");
 if(v.backNeck+v.frontNeck>v.hoodDepth*1.4)
   throw Error("Halsansatz ist zu lang für die gewählte Kapuzentiefe.");
 const joinMode=String(values.joinMode||"match");
 if(!["match","stretch","ease"].includes(joinMode))
  throw Error("Ungültige Nahtverarbeitungsart.");
 const allowedEase=finite(values.allowedEase??0.5,"Erlaubte / geplante Einhalteweite",0,4);
 const ageYears=Number(values.ageYears??PROFILES[profile].example.ageYears??8);
 if(profile==="child"&&(!Number.isFinite(ageYears)||ageYears<1||ageYears>14))
  throw Error("Kinderprofil: Alter muss zwischen 1 und 14 Jahren liegen.");
 if(profile==="child"&&values.drawstrings===true)
  throw Error("Kinderprofil: Halskordeln sind aus Sicherheitsgründen gesperrt.");
 const stretchPct=Number(values.stretchPct??0);
 const materialStretchVerified=values.materialStretchVerified===true;
 if(joinMode==="ease"&&allowedEase<=0)
  throw Error("Einhalten benötigt eine geplante positive Einhalteweite.");
 if(joinMode==="stretch"&&(!materialStretchVerified||!Number.isFinite(stretchPct)||
   stretchPct<=0||stretchPct>12))
  throw Error("Dehnen benötigt eine geprüfte Materialdehnung von 0 bis 12 %.");
 return {...v,variant,profile,joinMode,allowedEase,ageYears,stretchPct,materialStretchVerified,
  sourceProfileVerified:profile==="women"&&variant!=="dartless",
  sourceHeightWithinRange:v.hoodHeight>=VARIANTS[variant].minHeight*10&&v.hoodHeight<=VARIANTS[variant].maxHeight*10};
}
function seamCheck(actual,target,mode,max,description){
 const delta=actual-target,eps=.3; // submillimetre tolerance for the sampled curve
 let ok=Math.abs(delta)<=eps;
 if(mode==="stretch"&&!ok)ok=delta<0&&Math.abs(delta)<=max+eps;
 if(mode==="ease"&&!ok)ok=delta>0&&delta<=max+eps;
 return {name:description,actualMm:actual,targetMm:target,deltaMm:delta,mode,
  allowedMm:mode==="match"?eps:max,status:ok?"ok":"fail"};
}
function draft(values={}){
 const v=parse(values),loose=v.variant==="relaxed";
 const height=v.hoodHeight,depth=v.hoodDepth;
 const rearRaw=Math.max(1,v.backNeck-5); // public source reference, not equivalent to a sewn dart
 const rearCorrected=rearRaw+5; // original geometry compensates for exact neckline sewing length
 const ease=v.joinMode==="ease"?v.allowedEase:0;
 const factor=v.joinMode==="stretch"?1/(1+v.stretchPct/100):1;
 const rearTarget=(rearCorrected+ease*.5)*factor;
 const frontTarget=(v.frontNeck+ease*.5)*factor;
 const back=neckSegment([0,0],rearTarget,"back");
 const front=neckSegment(back.end,frontTarget,"front");
 const bottom=[...back.points,...front.points.slice(1)];
 // Loose version: published 4–5 cm base-distance allowance influences opening curve.
 const forehead=point(depth+v.faceOverlap,-height*.73+(loose?45*.15:0));
 const crown=point(depth*.42,-height);
 const mid=point(depth*.09,-height*.96);
 const rear=[0,0];
 const upperRear=sampleCubic(rear,[-depth*.10,-height*.43],[-depth*.08,-height*.88],crown,50);
 const upperFront=sampleCubic(crown,[depth*.72,-height*1.05],
    [depth*1.02,-height*.97],forehead,40);
 const frontEdge=sampleCubic(forehead,[depth*.99,-height*.49+(loose?45*.55:0)],
    [front.end[0]+depth*.13,-height*.19],front.end,35);
 const outer=[...upperRear,...upperFront.slice(1),
   ...frontEdge.slice(1),...bottom.slice(0,-1).reverse()];
 const sewnRear=arcLength(back.points),sewnFront=arcLength(front.points);
 const specs=[
  ["back-neck","Hinteres Halsloch ↔ Kapuze",sewnRear,v.backNeck],
  ["front-neck","Vorderes Halsloch ↔ Kapuze",sewnFront,v.frontNeck]
 ].map(([id,label,aMm,bMm])=>v.joinMode==="ease"?
    {id,label,aMm,bMm,mode:"ease",plannedEaseMm:v.allowedEase*.5,toleranceMm:.3}:
    v.joinMode==="stretch"?
    {id,label,aMm,bMm,mode:"stretch",maxStretchPct:v.stretchPct,
      materialStretchVerified:true,toleranceMm:.3}:
    {id,label,aMm,bMm,mode:"match",toleranceMm:.3});
 const centerLength=arcLength(upperRear)+arcLength(upperFront);
 specs.push({id:"center",label:"Kapuzenmittelnaht (gespiegelte Teile)",
   aMm:centerLength,bMm:centerLength,mode:"match",toleranceMm:.3});
 const seamResult=Contracts.checkAll(specs);
 const checks=seamResult.seams.map(item=>({
  name:item.label,actualMm:item.aMm,targetMm:item.bMm,
  deltaMm:item.deltaMm,mode:item.mode,allowedMm:item.toleranceMm,
  expectedDeltaMm:item.expectedDeltaMm,status:item.ok?"ok":"fail"
 }));
 if(!seamResult.ok)throw Error("Nahtanschlüsse gesperrt: "+
   seamResult.failed.join(", "));
 const cutArea=polygonArea(outer)*2;
 // Strip approximation is not a robust polygon offset around concave corners.
 const seamAdd=arcLength([...outer,outer[0]])*v.seamAllowance+Math.PI*v.seamAllowance*v.seamAllowance;
 const cutWithAllowance=cutArea+2*seamAdd;
 const flags=[];
 if(!v.sourceProfileVerified)flags.push("Für "+PROFILES[v.profile].label+" fehlen veröffentlichte vollständige Konstruktionsregeln; nur experimenteller Formvorschlag.");
 if(!v.sourceHeightWithinRange)flags.push("Kapuzenhöhe außerhalb des veröffentlichten Bereichs der gewählten Variante.");
 if(v.variant==="close"&&(v.hoodDepth<260||v.hoodDepth>280))flags.push("Anliegende Variante: Kapuzentiefe außerhalb der veröffentlichten 26–28 cm.");
 if(v.profile==="child")flags.push("Kinder: Sichtfeld, Halsöffnung und Verzicht auf gefährdende Kordeln prüfen.");
 if(v.variant==="dartless")flags.push("Abnäherlose Variante: Halsloch-Erweiterungen (+2/+1/+0,5 cm) müssen zuerst am Oberteil konstruiert und die neuen Halslinien gemessen werden.");
 flags.push("Alle Bézier-Konturen sind eigenständige Näherungen; Passform und Abnäher sind nicht nach Müller & Sohn geprüft.");
 if(checks.some(c=>c.status==="fail"))flags.push("Nahtprüfung nicht bestanden – vor dem Zuschnitt abgleichen.");
 flags.push("3D-Ansicht ist eine geometrische Drapierung, keine physikalische Stoffsimulation.");
 const points={K1:[0,0],K2:[0,-v.neckWidth-v.frontNeckDrop-(loose?45:0)],
  K3:crown,K4:forehead,K5:back.end,K6:[back.end[0]+v.dartIntake,0],
  K8:front.end,K9:forehead,K10:mid};
 const parts=[{id:"HOOD",name:"Kapuzenseitenteil",count:2,fold:false,
  notes:"Zwei spiegelgleiche Seiten; Mittellinie/Nackennaht markieren",
  outline:outer,cutline:outer,dart:v.dartIntake,
  neckSegments:{back:back.points,front:front.points},
  // These are the ACTUAL sampled drafting curves, not reconstructed guesses
  // by a separate 3D artist. Used as cutting-panel edges by Hood Drape.
  edges:{
    center:[...upperRear,...upperFront.slice(1)],
    face:frontEdge,
    neck:bottom
  }}];
 return {type:"hood",profile:v.profile,variant:v.variant,source:SOURCE,values:v,
  parts,pieces:2,points,checks,flags,rules:SOURCE_RELATIONS,
  productionValidated:false,sewingLinesVerified:seamResult.ok,
  sewnArea:cutArea,cutArea:cutWithAllowance,
  mass:cutWithAllowance/1e6*v.materialDensity,
  model:{type:"hood",heightMm:height,depthMm:depth,
    headCircMm:v.headCirc,baseNeckMm:v.backNeck+v.frontNeck,
    faceOverlapMm:v.faceOverlap},join:{rearRaw,rearCorrection:5},
  sourceStatus:v.sourceProfileVerified?"public DOB numerical relations; geometry approximate":"experimental; reference does not specify this profile"};
}
const poly=p=>p.map(v=>v.map(fmt).join(",")).join(" ");
function render(values={},allowance=null,density=null){
 const opts={...values};
 if(allowance!=null)opts.seamAllowance=Number(allowance)/10; // browser global mm
 if(density!=null)opts.materialDensity=Number(density);
 const d=draft(opts),p=d.parts[0],v=d.values,outline=p.outline;
 const xs=outline.map(a=>a[0]),ys=outline.map(a=>a[1]);
 const dx=50-Math.min(...xs),dy=55-Math.min(...ys);
 const width=Math.max(...xs)-Math.min(...xs)+160,height=Math.max(...ys)-Math.min(...ys)+160;
 const path=(pts,closed=false)=>"M "+pts.map((a,i)=>(i?"L ":"")+fmt(a[0])+" "+fmt(a[1])).join(" ")+(closed?" Z":"");
 const out=['<svg xmlns="http://www.w3.org/2000/svg" data-pattern="hood" data-pieces="2" data-source="muellerundsohn-public-dob" viewBox="0 0 '+fmt(width)+' '+fmt(height)+'" width="'+fmt(width)+'mm" height="'+fmt(height)+'mm">'];
 out.push('<metadata>Eigenständige numerische Interpretation öffentlich genannter Müller-&amp;-Sohn-Relationswerte. Quelle: '+svgText(SOURCE)+'</metadata>');
 out.push('<rect width="100%" height="100%" fill="#14202e"/>');
 out.push('<text x="28" y="22" fill="#edf7fc" font-size="15" font-weight="700">HOODLAB · KAPUZE · '+svgText(VARIANTS[v.variant].name.toUpperCase())+'</text>');
 out.push('<g transform="translate('+fmt(dx)+' '+fmt(dy)+')" data-piece-id="HOOD">');
 out.push('<path d="'+path(outline,true)+'" fill="#28546c" fill-opacity=".36" stroke="#69dae3" stroke-width="1.7"/>');
 // Seam allowance displayed as secondary, deliberately marked as schematic (not cut-ready)
 out.push('<path d="'+path(outline,true)+'" fill="none" stroke="#f4aa72" stroke-width=".8" stroke-dasharray="4 5" opacity=".75"/>');
 const rear=p.neckSegments.back,front=p.neckSegments.front;
 out.push('<path d="'+path(rear)+'" fill="none" stroke="#f5cb82" stroke-width="2.5"/>');
 out.push('<path d="'+path(front)+'" fill="none" stroke="#8be7a8" stroke-width="2.5"/>');
 out.push('<path d="M 0 0 V '+fmt(-v.hoodHeight)+' H '+fmt(v.hoodDepth)+'" stroke="#4b8396" stroke-dasharray="5 5" fill="none"/>');
 out.push('<path d="M '+fmt(-v.hoodDepth*.09)+' '+fmt(-v.hoodHeight*.5)+' H '+fmt(v.hoodDepth*.95)+'" stroke="#426e88" stroke-dasharray="4 5" fill="none"/>');
 for(const [key,pt]of Object.entries(d.points)){
  out.push('<circle cx="'+fmt(pt[0])+'" cy="'+fmt(pt[1])+'" r="2.6" fill="#e5d6a4"/>');
  out.push('<text x="'+fmt(pt[0]+5)+'" y="'+fmt(pt[1]-6)+'" fill="#d9e5e9" font-size="10">'+key+'</text>');
 }
 if(v.dartIntake>0&&v.variant!=="dartless"){
  const a=rear[Math.floor(rear.length*.55)],k=[a[0]+v.dartIntake/2,a[1]-55];
  out.push('<path d="M '+fmt(a[0]-v.dartIntake/2)+' '+fmt(a[1])+' L '+fmt(k[0])+' '+fmt(k[1])+' L '+fmt(a[0]+v.dartIntake/2)+' '+fmt(a[1])+'" stroke="#f6a0b8" stroke-width="1.25" fill="none"/>');
 }
 out.push('<path d="M '+fmt(v.hoodDepth*.47)+' -22 L '+fmt(v.hoodDepth*.47)+' -72" stroke="#d4df94" stroke-width="1.3"/>');
 out.push('<text x="'+fmt(v.hoodDepth*.47+5)+'" y="-45" fill="#d4df94" font-size="10">FDL</text>');
 out.push('</g>');
 const baseY=height-31;
 out.push('<text x="30" y="'+fmt(baseY-33)+'" fill="#b5cbd5" font-size="10">Grün: Vorderhals · Gelb: Rückenhals · Rosa: Abnähermarkierung</text>');
 out.push('<text x="30" y="'+fmt(baseY-16)+'" fill="#dcebf2" font-size="10">H '+fmt(v.hoodHeight/10)+' cm · T '+fmt(v.hoodDepth/10)+' cm · Kopf '+fmt(v.headCirc/10)+' cm · 2× spiegeln</text>');
 out.push('<text x="30" y="'+fmt(baseY)+'" fill="#f1be99" font-size="10">KONZEPT-GRUNDSCHNITT · Nahtzugabe nicht robust als Offset gezeichnet</text>');
 out.push('</svg>');
 return {...d,svg:out.join("\n")};
}
const API={source:SOURCE,profiles:PROFILES,variants:VARIANTS,defaults:DEFAULTS,
 sourceRelations:SOURCE_RELATIONS,parse,seamCheck,draft,render};
if(typeof module!=="undefined"&&module.exports)module.exports=API;
root.TextilLabHoodSystem=API;
})(typeof globalThis!=="undefined"?globalThis:this);

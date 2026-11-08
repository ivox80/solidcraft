/* TextilLab Kapuzenatelier 1.3 – original, parameterized, TWO-PIECE draft.
   The PUBLIC Müller & Sohn article inspires the variant names and dimensions
   (30–34cm high, 26–28cm deep for close-fit), but the precise curves below
   are original TextilLab prototypes: NOT a validated Müller-&-Sohn block.
   All internal geometry in mm. Exposed inputs are in cm.
*/
(function(root){
"use strict";
const Contracts=typeof module!=="undefined"&&module.exports?
  require("./seam-contracts.js"):root.TextilLabSeamContracts;
const SOURCE="https://www.muellerundsohn.com/allgemein/schnittkonstruktion-kapuze/";
const PROFILES=Object.freeze({
  women:Object.freeze({label:"Damen / DOB",head:57,backNeck:10,frontNeck:12,hoodHeight:32,hoodDepth:27,ageYears:null}),
  men:Object.freeze({label:"Herren / HAKA",head:59,backNeck:10.5,frontNeck:12.5,hoodHeight:34,hoodDepth:28,ageYears:null}),
  child:Object.freeze({label:"Kinder",head:52,backNeck:8,frontNeck:9.5,hoodHeight:28,hoodDepth:24,ageYears:8})
});
const STYLES=Object.freeze({
  close:Object.freeze({label:"Knapp anliegend",extraHeight:0,extraDepth:0}),
  relaxed:Object.freeze({label:"Aufgelockert",extraHeight:3,extraDepth:4.5})
});
const fmt=n=>Number(n.toFixed(3)),P=(x,y)=>[x,y],D=(a,b)=>Math.hypot(a[0]-b[0],a[1]-b[1]);
const polyarea=pts=>pts.reduce((sum,p,i)=>{
  const q=pts[(i+1)%pts.length];return sum+(p[0]*q[1]-q[0]*p[1]);
},0)/2;
function bezier(a,b,c,d,steps=56){
  const pts=[a];
  for(let i=1;i<=steps;i++){
    const t=i/steps,u=1-t;
    pts.push(P(
      u*u*u*a[0]+3*u*u*t*b[0]+3*u*t*t*c[0]+t*t*t*d[0],
      u*u*u*a[1]+3*u*u*t*b[1]+3*u*t*t*c[1]+t*t*t*d[1]
    ));
  }
  return pts;
}
function poly(segments){
  const pts=[];
  for(const segment of segments)pts.push(...(pts.length?segment.slice(1):segment));
  return pts;
}
function parameters(raw={},config={}){
  const group=config.profile??raw.profile??"women",style=config.style??raw.style??"close";
  if(!(group in PROFILES))throw Error("Profil wählen: Damen, Herren oder Kinder.");
  if(!(style in STYLES))throw Error("Kapuzenform wählen: knapp anliegend oder aufgelockert.");
  const d=PROFILES[group],s=STYLES[style],cm={};
  for(const field of ["head","backNeck","frontNeck","hoodHeight","hoodDepth"]){
    const n=Number(raw[field]??d[field]);
    if(!Number.isFinite(n)||n<=0)throw Error("Ungültiges Kapuzenmaß: "+field);
    cm[field]=n;
  }
  cm.hoodHeight+=s.extraHeight;
  cm.hoodDepth+=s.extraDepth;
  const age=(raw.ageYears??d.ageYears);
  if(group==="child"&&(!Number.isFinite(Number(age))||Number(age)<1||Number(age)>14))
    throw Error("Kinderprofil erfordert ein Alter von 1 bis 14 Jahren.");
  if(group==="child"&&config.drawstrings===true)
    throw Error("Kinderkapuze: Kordeln im Halsbereich sind gesperrt (DIN EN 14682).");
  if(cm.head<(group==="child"?40:48)||cm.head>(group==="child"?62:72))
    throw Error("Kopfumfang außerhalb des überprüfbaren "+d.label+"-Eingabebereichs.");
  if(cm.backNeck<5||cm.backNeck>20||cm.frontNeck<5||cm.frontNeck>24)
    throw Error("Halb-Halslochlinien müssen aus dem Kleidungsstück gemessen werden.");
  if(cm.hoodHeight<24||cm.hoodHeight>48||cm.hoodDepth<19||cm.hoodDepth>46)
    throw Error("Kapuzenhöhe oder -tiefe außerhalb des Konstruktionsbereichs.");
  if(cm.hoodHeight<cm.head*.44||cm.hoodDepth<cm.head*.36)
    throw Error("Kopfhöhe/-tiefe zu knapp für den eingegebenen Kopfumfang.");
  if(cm.backNeck+cm.frontNeck>cm.hoodDepth*1.22)
    throw Error("Halsloch ist im Verhältnis zur Kapuzentiefe zu lang; Maße prüfen.");
  const allowanceMm=Number(config.allowanceMm??10),density=Number(config.density??210);
  if(!Number.isFinite(allowanceMm)||allowanceMm<0||allowanceMm>30)
    throw Error("Nahtzugabe 0–30 mm wählen.");
  if(!Number.isFinite(density)||density<=0||density>2000)
    throw Error("Stoffgewicht 1–2000 g/m² wählen.");
  const mode=config.neckMode??"match";
  if(!["match","ease","stretch"].includes(mode))throw Error("Unbekannte Nahtverarbeitung.");
  let plannedEaseMm=0,stretchPct=0;
  if(mode==="ease"){
    plannedEaseMm=Number(config.plannedEaseCm??.4)*10;
    if(!Number.isFinite(plannedEaseMm)||plannedEaseMm<=0||plannedEaseMm>30)
      throw Error("Einhalteweite für den Halb-Halsausschnitt muss 0–3 cm betragen.");
  }
  if(mode==="stretch"){
    stretchPct=Number(config.stretchPct??0);
    if(!Number.isFinite(stretchPct)||stretchPct<=0||stretchPct>12||
       config.materialStretchVerified!==true)
      throw Error("Dehnen nur mit dokumentierter Materialdehnung bis 12 % erlaubt.");
  }
  return {group,style,ageYears:group==="child"?Number(age):null,
    mm:Object.fromEntries(Object.entries(cm).map(([k,v])=>[k,v*10])),
    cm,allowanceMm,density,mode,plannedEaseMm,stretchPct,
    materialStretchVerified:config.materialStretchVerified===true,
    sourceStatus:group==="women"?"reference-inspired":
      group==="men"?"HAKA rules not validated":"child rules not validated"};
}
function neckCurve(start,targetMm,riseY,endYOffset){
  // Find the horizontal run so the *actual sampled cubic sewing line*
  // has the required true arc length. No straight-chord approximations.
  const make=run=>bezier(start,P(start[0]+.28*run,start[1]-riseY),
    P(start[0]+.73*run,start[1]-riseY),
    P(start[0]+run,start[1]+endYOffset));
  let lo=1,hi=targetMm;
  for(let i=0;i<56;i++){
    const mid=(lo+hi)*.5;
    if(Contracts.measured(make(mid))>targetMm)hi=mid;else lo=mid;
  }
  const curve=make((lo+hi)*.5),actual=Contracts.measured(curve);
  if(Math.abs(actual-targetMm)>.05)
    throw Error("Halslochkurve lässt sich nicht mit dieser Länge konstruieren.");
  return curve;
}
function reverse(points){return [...points].reverse()}
function prepare(input={},options={}){
  const m=parameters(input,options);
  const {head,hoodHeight:height,hoodDepth:depth,frontNeck,backNeck}=m.mm;
  const extra=m.mode==="ease"?m.plannedEaseMm:0;
  // Required stretch is measured against original A: B/A - 1.
  const factor=m.mode==="stretch"?1/(1+m.stretchPct/100):1;
  const desiredBack=(backNeck+extra*.5)*factor;
  const desiredFront=(frontNeck+extra*.5)*factor;
  const A=P(0,height);
  const neckBack=neckCurve(A,desiredBack,14,-5),S=neckBack.at(-1);
  const neckFront=neckCurve(S,desiredFront,12,-12),F=neckFront.at(-1);
  const apex=P(depth*.54,0),upperBack=poly([
    bezier(A,P(-17,height*.61),P(-8,height*.10),P(depth*.25,8)),
    bezier(P(depth*.25,8),P(depth*.34,-5),P(depth*.47,-2),apex)
  ]);
  const face=poly([
    bezier(apex,P(depth*.84,-2),P(depth*1.03,height*.14),P(depth*.98,height*.34)),
    bezier(P(depth*.98,height*.34),P(depth*1.05,height*.59),
      P(depth*.92,height*.75),F)
  ]);
  const outline=poly([upperBack,face,reverse(neckFront),reverse(neckBack)]);
  outline.pop(); // the final base endpoint is A; polygon closure is implicit
  const centerMm=Contracts.measured(upperBack);
  const seamSpecs=[
    {id:"center-pair",label:"Kapuze: Mittelnaht links ↔ rechts",
      mode:"match",aMm:centerMm,bMm:centerMm,toleranceMm:.25},
    ...[["back-neck","Hinteres Halsloch",Contracts.measured(neckBack),backNeck],
      ["front-neck","Vorderes Halsloch",Contracts.measured(neckFront),frontNeck]]
      .map(([id,label,aMm,bMm])=>{
        if(m.mode==="ease")
          return {id,label,mode:"ease",aMm,bMm,plannedEaseMm:m.plannedEaseMm*.5,toleranceMm:.25};
        if(m.mode==="stretch")
          return {id,label,mode:"stretch",aMm,bMm,maxStretchPct:m.stretchPct,
            materialStretchVerified:true,toleranceMm:.25};
        return {id,label,mode:"match",aMm,bMm,toleranceMm:.25};
      })
  ];
  const report=Contracts.checkAll(seamSpecs);
  const values={rearNeckMm:Contracts.measured(neckBack),
    frontNeckMm:Contracts.measured(neckFront),centerSeamMm:centerMm,
    totalNeckMm:Contracts.measured(neckBack)+Contracts.measured(neckFront),
    garmentHalfNeckMm:backNeck+frontNeck};
  const landmarks={A,S,F,apex,
    backGuide:P(-20,height*.47),faceGuide:P(depth*.98,height*.35),
    headGuide:P(depth*.54,height*.55)};
  const rules=[
    {id:"KH01",name:"Kapuzenhöhe",formula:"Kopfhöhe laut Eingabe + Formzuschlag",mm:height},
    {id:"KH02",name:"Kapuzentiefe",formula:"Kapuzentiefe laut Eingabe + Formzuschlag",mm:depth},
    {id:"KH03",name:"Rückenhalsanschluss",formula:"Bogenlänge = Halsloch hinten/2 ± geplante Verarbeitung",mm:values.rearNeckMm},
    {id:"KH04",name:"Vorderhalsanschluss",formula:"Bogenlänge = Halsloch vorn/2 ± geplante Verarbeitung",mm:values.frontNeckMm},
    {id:"KH05",name:"Mittelnaht",formula:"Zwei spiegelgleiche Kapuzenhälften, gleicher Kurvenverlauf",mm:centerMm}
  ];
  return {profile:m,parts:[{
    name:"Kapuzenseite",id:"KH",count:2,fold:false,
    notes:"2 spiegelgleiche Teile; Ansatznaht nach gemessener Halslochlinie",
    outline,neckBack,neckFront,center:upperBack,face,landmarks,
    grain:P(depth*.48,height*.60)
  }],report,values,rules,ready:report.ok,
   productionValidated:false,designStatus:"Experimenteller Konstruktionsprototyp"};
}
function offset(points,distance){
  if(!distance)return points.map(p=>p.slice());
  const orientation=Math.sign(polyarea(points));
  const lines=points.map((p,i)=>{
    const q=points[(i+1)%points.length],dx=q[0]-p[0],dy=q[1]-p[1],
      len=Math.hypot(dx,dy);
    if(len<1e-7)throw Error("Doppelte Konstruktionspunkte in der Nahtkontur.");
    const nx=orientation*dy/len*distance,ny=-orientation*dx/len*distance;
    return [P(p[0]+nx,p[1]+ny),P(q[0]+nx,q[1]+ny)];
  });
  return lines.map((line,i)=>{
    const prev=lines[(i+lines.length-1)%lines.length];
    const a=prev[0],b=prev[1],c=line[0],d=line[1],
      rx=b[0]-a[0],ry=b[1]-a[1],sx=d[0]-c[0],sy=d[1]-c[1],det=rx*sy-ry*sx;
    if(Math.abs(det)<1e-7)return c;
    const t=((c[0]-a[0])*sy-(c[1]-a[1])*sx)/det,
      hit=P(a[0]+rx*t,a[1]+ry*t);
    return D(hit,points[i])>Math.max(distance*4,22)?
      P((b[0]+c[0])/2,(b[1]+c[1])/2):hit;
  });
}
const path=points=>points.map((p,i)=>(i?"L ":"M ")+fmt(p[0])+" "+fmt(p[1])).join(" ");
const pointsAttr=pts=>pts.map(p=>fmt(p[0])+","+fmt(p[1])).join(" ");
function render(input={},options={}){
  const result=prepare(input,options),{profile,parts,report,values}=result,part=parts[0];
  const cut=offset(part.outline,profile.allowanceMm);
  const sewnArea=Math.abs(polyarea(part.outline))*2,
    cutArea=Math.abs(polyarea(cut))*2,
    mass=cutArea*profile.density/1e6;
  if(!(sewnArea>0&&cutArea>0)||!Number.isFinite(mass))throw Error("Ungültige Konturfläche.");
  const all=[...part.outline,...cut],minx=Math.min(...all.map(p=>p[0])),
    maxx=Math.max(...all.map(p=>p[0])),miny=Math.min(...all.map(p=>p[1])),
    maxy=Math.max(...all.map(p=>p[1]));
  const partWidth=maxx-minx,partHeight=maxy-miny,
    width=partWidth*2+210,height=partHeight+155;
  const shiftx=75-minx,shifty=83-miny,mirrorX=width-75+minx;
  const partsSvg=[];
  partsSvg.push('<svg xmlns="http://www.w3.org/2000/svg" width="'+fmt(width)+'mm" height="'+fmt(height)+'mm" viewBox="0 0 '+fmt(width)+' '+fmt(height)+'" data-hood-system="textillab.hood.prototype.1" data-profile="'+profile.group+'" data-seams-ok="'+report.ok+'" data-production-approved="false">');
  partsSvg.push('<rect width="100%" height="100%" fill="#14202e"/>');
  partsSvg.push('<text x="30" y="27" font-size="16" fill="#e5f2f8" font-weight="700">TEXTILLAB · KAPUZE · '+PROFILES[profile.group].label.toUpperCase()+'</text>');
  partsSvg.push('<text x="30" y="46" font-size="11" fill="#94b4c8">Schnittkonstruktion · TextilLab Experimental · 2× spiegelgleich</text>');
  const groups=[
    {x:shiftx,flip:false},
    {x:mirrorX,flip:true}
  ];
  for(const [index,g] of groups.entries()){
    const trans='translate('+fmt(g.x)+' '+fmt(shifty)+')'+(g.flip?' scale(-1 1)':'');
    partsSvg.push('<g data-piece-id="KH" transform="'+trans+'">');
    partsSvg.push('<path d="'+path(part.outline)+' Z" fill="#25495a" fill-opacity=".72" stroke="#7fe0e5" stroke-width="1.55" stroke-dasharray="4 3"/>');
    if(profile.allowanceMm>0)partsSvg.push('<polygon points="'+pointsAttr(cut)+'" fill="none" stroke="#f1a678" stroke-width="1.3"/>');
    partsSvg.push('<path d="'+path(part.center)+'" fill="none" stroke="#f4bd70" stroke-width="2.1" data-seam="center"/>');
    partsSvg.push('<path d="'+path(part.neckBack)+'" fill="none" stroke="#67c8d6" stroke-width="2.5" data-seam="back-neck"/>');
    partsSvg.push('<path d="'+path(part.neckFront)+'" fill="none" stroke="#c3d782" stroke-width="2.5" data-seam="front-neck"/>');
    partsSvg.push('<path d="M '+fmt(part.landmarks.A[0])+' '+fmt(part.landmarks.A[1])+' H '+fmt(part.landmarks.F[0])+'" fill="none" stroke="#648494" stroke-dasharray="5 6" stroke-width=".7"/>');
    partsSvg.push('<path d="M '+fmt(part.landmarks.A[0])+' '+fmt(part.landmarks.A[1])+' V '+fmt(part.landmarks.apex[1])+' H '+fmt(part.landmarks.F[0])+'" fill="none" stroke="#668b9f" stroke-dasharray="4 6" stroke-width=".75"/>');
    partsSvg.push('<path d="M '+fmt(part.grain[0])+' '+fmt(part.grain[1]-47)+' v 76 m -5 -12 l 5 12 5 -12" fill="none" stroke="#f6d789" stroke-width="1.7"/>');
    for(const [name,p] of Object.entries(part.landmarks)){
      if(!["A","S","F","apex"].includes(name))continue;
      partsSvg.push('<circle cx="'+fmt(p[0])+'" cy="'+fmt(p[1])+'" r="3.1" fill="#f2f4fb"/>');
      if(!g.flip)partsSvg.push('<text x="'+fmt(p[0]+7)+'" y="'+fmt(p[1]-8)+'" font-size="12" fill="#dbebf3">'+name+'</text>');
    }
    partsSvg.push('</g>');
    partsSvg.push('<text x="'+fmt(g.x+(g.flip?-partWidth*.4:partWidth*.4))+'" y="'+fmt(height-60)+'" font-size="12" fill="#c6e8ec" text-anchor="middle">KH-'+(index+1)+' · '+(index?"rechts":"links")+'</text>');
  }
  partsSvg.push('<text x="30" y="'+fmt(height-35)+'" fill="#9cc0d0" font-size="12">KH01 H '+(profile.mm.hoodHeight/10).toFixed(1)+' cm · KH02 T '+(profile.mm.hoodDepth/10).toFixed(1)+' cm · Rückenhals '+(values.rearNeckMm/10).toFixed(1)+' cm · Vorderhals '+(values.frontNeckMm/10).toFixed(1)+' cm</text>');
  partsSvg.push('<text x="30" y="'+fmt(height-14)+'" fill="'+(report.ok?'#b0d4b9':'#fb9898')+'" font-size="11">'+(report.ok?'NAHTLINIEN KONTROLLIERT':'FEHLER: NAHTLÄNGEN ABWEICHEND')+' · NICHT FÜR FERTIGUNG VALIDIERT · Kinder: keine Kordeln</text>');
  partsSvg.push('</svg>');
  return {...result,svg:partsSvg.join("\n"),sewnArea,cutArea,mass,
    pieces:2,parts:parts.map(p=>({
      name:p.name,outline:p.outline,count:p.count,fold:p.fold,notes:p.notes
    })),cutEdgePreviewOnly:true};
}
const api={source:SOURCE,profiles:PROFILES,styles:STYLES,parameters,prepare,render,
  geometry:{bezier,polyarea,offset,neckCurve}};
if(typeof module!=="undefined"&&module.exports)module.exports=api;
root.TextilLabHoodSystem=api;
})(typeof globalThis!=="undefined"?globalThis:this);

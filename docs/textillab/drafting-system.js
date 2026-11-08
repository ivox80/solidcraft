/* TextilLab v1.0 drafting rules and technical construction layers.
 * Independent, explicitly named house-rule system ("TextilLab Experimental
 * Bodice 1"), NOT a licensed Mueller/Sohn or other proprietary block.
 * Values are mm internally. Input measurements and alterations are cm.
 * A drawing and measured seam checks do not imply a validated fit.
 */
(function(root) {
"use strict";
const B = (typeof module !== "undefined" && module.exports) ? require("./bodice-v2.js") : root.TextilLabBodiceV2;
const FITS = Object.freeze({
  fitted:{label:"Körpernah",ease:4},
  regular:{label:"Regular",ease:8},
  comfort:{label:"Bequem",ease:12},
  loose:{label:"Weit",ease:18},
  oversize:{label:"Oversize",ease:24},
  custom:{label:"Benutzerdefiniert",ease:null}
});
const STYLES = Object.freeze({
  bodice:{label:"Grundschnitt",torso:0,hem:0,sleeves:true},
  shirt:{label:"Shirt",torso:0,hem:0,sleeves:true},
  blouse:{label:"Bluse",torso:0,hem:6,sleeves:true},
  jacket:{label:"Jackenbasis",torso:0,hem:15,sleeves:true},
  coat:{label:"Mantelbasis",torso:0,hem:45,sleeves:true},
  sleeveless:{label:"Ärmellos",torso:0,hem:0,sleeves:false}
});
const ZONES = Object.freeze({
  torso:{label:"Brust – Taille",fields:["frontLength","backWaist"]},
  hip:{label:"Taille – Hüfte",fields:["waistToHip","hemBelowWaist"]},
  hem:{label:"Unterhalb Hüfte",fields:["hemBelowWaist"]},
  sleeve:{label:"Ärmel unter Kugel",fields:["sleeve"]}
});
const DEFAULT_LAYERS = Object.freeze({
  guides:true,dimensions:true,points:true,alterations:true,notes:true
});
const fmt = x => Number(x.toFixed(2));
const dist = (a,b) => Math.hypot(a[0]-b[0],a[1]-b[1]);
const pt = (x,y) => [fmt(x),fmt(y)];
const escape = x => String(x).replace(/[&<>"']/g, s=>({
  "&":"&amp;","<":"&lt;",">":"&gt;",'"':"&quot;","'":"&apos;"
}[s]));
const polylen = (p,start,end) => {
  let sum=0;
  for(let i=start+1;i<=end;i++)sum+=dist(p[i-1],p[i]);
  return sum;
};
// Segments below correspond to sampled Bezier parts of BodiceV2.createPiece
// and createSleeve (24 neck, 1 shoulder, 30 armhole, 48 sleeve cap).
function seamCheck(draft) {
  const [front,back,sleeve]=draft.pieces;
  const armFront=polylen(front.outline,25,55);
  const armBack=polylen(back.outline,25,55);
  const sleeveCap=polylen(sleeve.outline,0,48);
  const frontSide=polylen(front.outline,55,129);
  const backSide=polylen(back.outline,55,129);
  const ease=sleeveCap-armFront-armBack;
  return {
    frontArmholeMm:armFront, backArmholeMm:armBack,
    totalArmholeMm:armFront+armBack,sleeveCapMm:sleeveCap,
    sleeveEaseMm:ease,sideDifferenceMm:Math.abs(frontSide-backSide),
    frontSideMm:frontSide,backSideMm:backSide,
    shoulderDartFrontMm:dist(...front.darts.shoulder.legs),
    shoulderDartBackMm:dist(...back.darts.shoulder.legs)
  };
}
function validateOptions(o) {
  if(o.fit && !(o.fit in FITS))throw Error("Unbekannte Passform");
  if(o.style && !(o.style in STYLES))throw Error("Unbekannte Modellvariante");
  for(const [zone,range] of Object.entries(o.alterations||{})) {
    if(!(zone in ZONES))throw Error("Unbekannte Änderungslinie: "+zone);
    const n=Number(range);
    if(!Number.isFinite(n)||n < -12||n > 35)throw Error("Längenänderung "+zone+" muss -12 bis +35 cm betragen.");
  }
  const target=Number(o.targetEaseMm ?? 22);
  if(!Number.isFinite(target)||target<0||target>45)throw Error("Ärmelkugel-Einhalteweite 0 bis 45 mm wählen.");
  return target;
}
function modifyMeasurements(input={},opt={}) {
  const target=validateOptions(opt);
  const style=STYLES[opt.style||"bodice"];
  const fit=FITS[opt.fit||"regular"];
  const raw={chest:100,waist:84,hip:102,shoulder:44,sleeve:62,ease:8,...B.defaults,...input};
  if(fit.ease!==null)raw.ease=fit.ease;
  const changes={torso:style.torso,hip:0,hem:style.hem,sleeve:0,...(opt.alterations||{})};
  for(const [zone,amount] of Object.entries(changes)) {
    const n=Number(amount);
    if(!Number.isFinite(n))throw Error("Nicht-numerische Änderung: "+zone);
    for(const field of ZONES[zone].fields)raw[field]=Number(raw[field]??defaultsCm(field))+n;
  }
  // The source bodice's own validator checks all resulting dimensions.
  B.measureValues(raw);
  return {measurements:raw,target,style,fit,changes};
}
function defaultsCm(field) {
  return (field in B.defaults ? B.defaults[field] :
    ({frontLength:46,backWaist:43,waistToHip:20,hemBelowWaist:29,sleeve:62})[field]);
}
function balancedDraft(m,target) {
  // Bisection adjusts sleeve-cap height (not garment length or biceps width).
  // This is a *mathematical* cap-length match, not a validated pattern system.
  const make = h => B.draft(m,{capHeightMm:h});
  let low=80,high=215,lo=make(low),hi=make(high);
  let loEase=seamCheck(lo).sleeveEaseMm,hiEase=seamCheck(hi).sleeveEaseMm;
  if(target<loEase || target>hiEase){
    const candidate=Math.abs(target-loEase)<Math.abs(target-hiEase)?lo:hi;
    return {draft:candidate,capHeightMm:candidate===lo?low:high,
      reachable:false,achievedEaseMm:seamCheck(candidate).sleeveEaseMm};
  }
  let trial=lo,h=low;
  for(let i=0;i<28;i++) {
    h=(low+high)*.5;
    trial=make(h);
    const diff=seamCheck(trial).sleeveEaseMm-target;
    if(Math.abs(diff)<.06)break;
    if(diff>0) high=h;
    else low=h;
  }
  return {draft:trial,capHeightMm:h,reachable:true,
    achievedEaseMm:seamCheck(trial).sleeveEaseMm};
}
function rules(m,metric,cap,profile) {
  // formula provenance: proprietary construction systems are NOT implied.
  return [
    {id:"R01",title:"Brust-Teilungsweite",formula:"(Br + Mehrweite) / 4",value:(m.chest+m.ease)/4,unit:"mm",depends:["chest","ease"]},
    {id:"R02",title:"Taillen-Teilungsweite",formula:"(Ta + Mehrweite) / 4",value:(m.waist+m.ease)/4,unit:"mm",depends:["waist","ease"]},
    {id:"R03",title:"Hüft-Teilungsweite",formula:"(Hü + Mehrweite) / 4",value:(m.hip+m.ease)/4,unit:"mm",depends:["hip","ease"]},
    {id:"R04",title:"Armlochhöhe",formula:"Armlochtiefe ab Halsreferenz",value:m.armholeDepth,unit:"mm",depends:["armholeDepth"]},
    {id:"R05",title:"Vorderlänge",formula:"Schulterreferenz – Taille",value:m.frontLength,unit:"mm",depends:["frontLength"]},
    {id:"R06",title:"Rückenlänge",formula:"Schulterreferenz – Taille",value:m.backWaist,unit:"mm",depends:["backWaist"]},
    {id:"R07",title:"Ärmelkugel",formula:"Numerisch: Armloch VT + RT + Ziel-Einhalteweite",value:metric.sleeveCapMm,unit:"mm",depends:["upperArm","armholeDepth","targetEaseMm"],
      target:metric.totalArmholeMm+profile.target,matched:cap.reachable},
    {id:"R08",title:"Seitennaht-Balance",formula:"|Nahtlänge VT – Nahtlänge RT|",value:metric.sideDifferenceMm,unit:"mm",depends:["frontLength","backWaist","hemBelowWaist"]}
  ];
}
function namedPoints(d) {
  const [vt,rt,sleeve]=d.pieces;
  const torso=(piece,prefix)=>{
    const l=piece.outline;
    return [
      {id:prefix+"0",label:prefix==="VT"?"MV":"MH",xy:l[0],note:"Mitte oben"},
      {id:prefix+"1",label:"HN",xy:piece.points.shoulderStart,note:"Hals-Schulter"},
      {id:prefix+"2",label:"SP",xy:piece.points.shoulderEnd,note:"Schulterpunkt"},
      {id:prefix+"3",label:"AL",xy:l[55],note:"Armloch unten"},
      {id:prefix+"4",label:prefix==="VT"?"BP":"BR",xy:piece.points.bustPoint,note:"Brustreferenz"},
      {id:prefix+"5",label:"TA",xy:pt(0,piece.points.waistY),note:"Taillenmitte"},
      {id:prefix+"6",label:"HÜ",xy:pt(0,piece.points.hipY),note:"Hüftmitte"},
      {id:prefix+"7",label:"SA",xy:pt(0,piece.points.hemY),note:"Saummitte"}
    ];
  };
  return {
    VT:torso(vt,"VT"),RT:torso(rt,"RT"),
    AR:[
      {id:"AR0",label:"Ä0",xy:pt(sleeve.grain[0],0),note:"Ärmelkugel oben"},
      {id:"AR1",label:"OB",xy:pt(0,sleeve.points.bicepY),note:"Oberarmlinie"},
      {id:"AR2",label:"ELL",xy:pt(sleeve.grain[0],sleeve.points.hemY*.63),note:"Ellenbogenhöhe (Näherung)"},
      {id:"AR3",label:"AS",xy:pt(sleeve.grain[0],sleeve.points.hemY),note:"Ärmelsaum"}
    ]
  };
}
const line=(a,b,color="#426879",dash="5 5",width=".7")=>
  '<path d="M '+fmt(a[0])+' '+fmt(a[1])+' L '+fmt(b[0])+' '+fmt(b[1])+
  '" fill="none" stroke="'+color+'" stroke-width="'+width+'" stroke-dasharray="'+dash+'"/>';
function label(x,y,text,anchor="start",color="#a9c4d3",size=10) {
  return '<text x="'+fmt(x)+'" y="'+fmt(y)+'" fill="'+color+'" font-size="'+size+
    '" text-anchor="'+anchor+'" font-family="Segoe UI,Arial,sans-serif">'+escape(text)+'</text>';
}
function dimension(a,b,offset,id,text,field) {
  // dimension with real numeric length, extension lines and inward arrowheads
  const dx=b[0]-a[0],dy=b[1]-a[1],vertical=Math.abs(dy)>Math.abs(dx);
  const aa=vertical?pt(a[0]+offset,a[1]):pt(a[0],a[1]+offset);
  const bb=vertical?pt(b[0]+offset,b[1]):pt(b[0],b[1]+offset);
  const mid=pt((aa[0]+bb[0])*.5,(aa[1]+bb[1])*.5);
  let result='<g class="tl-dimension" data-rule="'+escape(id)+'" data-measure="'+escape(field)+'">';
  result+=line(a,aa,"#5d8ba3","2 2",".65")+line(b,bb,"#5d8ba3","2 2",".65");
  result+=line(aa,bb,"#abdbe9","","1.15");
  const orientation=vertical?"0":"-90";
  const shift=vertical?pt(10,0):pt(0,-8);
  result+=label(mid[0]+shift[0],mid[1]+shift[1],text,"middle","#e9f2f7",11);
  const arrow=(pos,sign)=>{
    const d=vertical?pt(0,sign*7):pt(sign*7,0),v=vertical?pt(3.2,0):pt(0,3.2);
    const p1=pt(pos[0]+d[0]+v[0],pos[1]+d[1]+v[1]);
    const p2=pt(pos[0]+d[0]-v[0],pos[1]+d[1]-v[1]);
    return '<path d="M '+p1.join(' ')+' L '+pos.join(' ')+' L '+p2.join(' ')+
      '" stroke="#abdbe9" stroke-width="1" fill="none"/>';
  };
  result+=arrow(aa,1)+arrow(bb,-1);
  result+='</g>';
  return result;
}
function layeredDrawing(base,d,profile,config,metric) {
  const L={...DEFAULT_LAYERS,...(config.layers||{})};
  const [vt,rt,sleeve]=d.pieces;
  const fw=Math.max(...vt.outline.map(p=>p[0])),bw=Math.max(...rt.outline.map(p=>p[0]));
  const fx=75,bx=fx+fw+100,ax=bx+bw+100,ay=95;
  const bwMax=Math.max(...sleeve.outline.map(p=>p[0]));
  const positions={VT:p=>pt(fx+p[0],p[1]),RT:p=>pt(bx+bw-p[0],p[1]),AR:p=>pt(ax+p[0],ay+p[1])};
  const boundaries={
    VT:{x0:fx,x1:fx+fw,piece:vt},
    RT:{x0:bx,x1:bx+bw,piece:rt},
    AR:{x0:ax,x1:ax+bwMax,piece:sleeve}
  };
  const parts=[];
  parts.push('<g id="textillab-construction-overlay" data-overlay-version="1.0">');
  if(L.guides) {
    parts.push('<g id="layer-construction-lines">');
    for(const [key,region] of Object.entries(boundaries)) {
      const item=region.piece,ys=key==="AR"?
        [["OB",ay+item.points.bicepY],["ELL",ay+item.points.hemY*.63],["AS",ay+item.points.hemY]]:
        [["BR",item.points.bustY],["TA",item.points.waistY],["HÜ",item.points.hipY],["SA",item.points.hemY]];
      for(const [id,y]of ys) {
        parts.push(line(pt(region.x0-13,y),pt(region.x1+13,y)));
        if(key==="AR")parts.push(label(region.x1+16,y-3,id));
      }
      if(key!=="AR"){
        const center=key==="VT"?region.x0:region.x1;
        parts.push(line(pt(center, item.points.topY-6),pt(center,item.points.hemY+14),"#436e84","3 6"));
        const bp=positions[key](item.points.bustPoint);
        parts.push(line(pt(bp[0],item.points.topY+85),pt(bp[0],item.points.waistY+70),"#3b7183","4 7"));
      }else{
        const center=ax+item.grain[0];
        parts.push(line(pt(center,ay-10),pt(center,ay+item.points.hemY+15),"#436e84","3 6"));
      }
    }
    parts.push('</g>');
  }
  if(L.points){
    parts.push('<g id="layer-construction-points">');
    const all=namedPoints(d);
    for(const [id,points]of Object.entries(all))for(const point of points){
      const [x,y]=positions[id](point.xy);
      parts.push('<circle cx="'+x+'" cy="'+y+'" r="3" fill="#7edbe2" stroke="#14202e" stroke-width=".8"><title>'+
        escape(point.id+" · "+point.note)+'</title></circle>');
      parts.push(label(x+6,y-6,point.label,"start","#c2dfe6",11));
    }
    parts.push('</g>');
  }
  if(L.dimensions){
    parts.push('<g id="layer-dimensions">');
    const f=vt.points;
    parts.push(dimension(positions.VT(pt(0,f.topY)),positions.VT(pt(0,f.waistY)),-40,"VL",
      "VL "+(profile.measurements.frontLength).toFixed(1)+" cm","frontLength"));
    const b=rt.points;
    parts.push(dimension(positions.RT(pt(0,b.topY)),positions.RT(pt(0,b.waistY)),40,"RL",
      "RL "+(profile.measurements.backWaist).toFixed(1)+" cm","backWaist"));
    parts.push(dimension(positions.VT(f.shoulderStart),positions.VT(f.shoulderEnd),-40,"S",
      "Sch "+(profile.measurements.shoulder).toFixed(1)+" cm","shoulder"));
    const y=vt.points.bustY;
    parts.push(dimension(positions.VT(pt(0,y)),positions.VT(pt(f.bustPoint[0],y)),45,"BPA",
      "BP/2 "+(profile.measurements.bustSpan/2).toFixed(1)+" cm","bustSpan"));
    parts.push(dimension(positions.AR(pt(sleeve.grain[0],0)),
      positions.AR(pt(sleeve.grain[0],sleeve.points.hemY)),35,"ÄL",
      "ÄL "+profile.measurements.sleeve.toFixed(1)+" cm","sleeve"));
    parts.push(dimension(positions.AR(pt(0,sleeve.points.bicepY)),
      positions.AR(pt(bwMax,sleeve.points.bicepY)),22,"OU",
      "OB "+profile.measurements.upperArm.toFixed(1)+" cm","upperArm"));
    parts.push('</g>');
  }
  if(L.alterations) {
    parts.push('<g id="layer-alteration-lines">');
    // Each pair of bars brackets an intentional alteration interval
    const torsoY=(vt.points.bustY+vt.points.waistY)*.5;
    const hipY=(vt.points.waistY+vt.points.hipY)*.5;
    const hemY=(vt.points.hipY+vt.points.hemY)*.5;
    const elbowY=ay+sleeve.points.hemY*.64;
    for(const [zone,y]of [["torso",torsoY],["hip",hipY],["hem",hemY]]) {
      for(const region of [boundaries.VT,boundaries.RT])
        parts.push(line(pt(region.x0+8,y),pt(region.x1-8,y),"#d18ff2","6 4","1.15"));
      parts.push(label(boundaries.RT.x1+20,y-7,"K-"+zone,"start","#e7b4ff",11));
    }
    parts.push(line(pt(ax+5,elbowY),pt(ax+bwMax-5,elbowY),"#d18ff2","6 4","1.15"));
    parts.push(label(ax+bwMax+10,elbowY-7,"K-Ä","start","#e7b4ff",11));
    parts.push('</g>');
  }
  if(L.notes){
    parts.push('<g id="layer-rule-notes">');
    const note="Armloch "+fmt(metric.totalArmholeMm)+" mm · Ärmelkugel "+
      fmt(metric.sleeveCapMm)+" mm · Einhalteweite "+fmt(metric.sleeveEaseMm)+" mm";
    parts.push(label(ax,ay+sleeve.points.hemY+38,note,"start","#a8c8d9",11));
    const state=metric.sleeveEaseMm<0?"ACHTUNG: negative Einhalteweite":
      metric.sideDifferenceMm>5?"Seitennahtbalance überprüfen":"Kurvenlängen numerisch geprüft";
    parts.push(label(ax,ay+sleeve.points.hemY+54,state,"start",
      metric.sleeveEaseMm<0?"#ffa799":"#8fd4c2",10));
    parts.push('</g>');
  }
  parts.push('</g>');
  return base.replace("</svg>",parts.join("")+"</svg>");
}
function generate(measurements={},options={}) {
  const profile=modifyMeasurements(measurements,options);
  const balanced=balancedDraft(profile.measurements,profile.target);
  const d=balanced.draft;
  const metric=seamCheck(d);
  const onlyBody=!profile.style.sleeves;
  const result=B.render(profile.measurements,Number(options.allowance??10),
    Number(options.density??210), {capHeightMm:balanced.capHeightMm,sleeves:!onlyBody});
  result.svg=layeredDrawing(result.svg,d,profile,options,metric);
  const m=B.measureValues(profile.measurements);
  result.rules=rules(m,metric,balanced,profile);
  result.measures=metric;
  result.profile=profile;
  result.capBalance={targetEaseMm:profile.target,actualEaseMm:metric.sleeveEaseMm,
    capHeightMm:balanced.capHeightMm,reachable:balanced.reachable};
  result.namedPoints=namedPoints(d);
  result.version="TextilLab 1.0 experimental";
  return result;
}
const API={fits:FITS,styles:STYLES,zones:ZONES,layers:DEFAULT_LAYERS,
  generate,modifyMeasurements,balancedDraft,seamCheck,rules,namedPoints,dimension};
if(typeof module!=="undefined"&&module.exports)module.exports=API;
root.TextilLabDraftingSystem=API;
})(typeof globalThis!=="undefined"?globalThis:this);

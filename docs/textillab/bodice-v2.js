/* TextilLab Bodice V2 -- front (VT) and back (RT) drafting concept.
 * Units: user inputs in cm, geometry in mm.
 * Technical illustration only. NOT a fitted or production-approved block.
 */
(function(root) {
"use strict";
const DEFAULTS={
  frontLength:46, backWaist:43, bustHeight:27, bustSpan:19,
  neck:38, armholeDepth:23, waistToHip:20, hemBelowWaist:29,
  shoulderSlope:3, upperArm:34, wrist:19
};
const EXTENDED=[
 ["frontLength","Vorderlänge (bis Taille)"],
 ["backWaist","Rückenlänge (bis Taille)"],
 ["bustHeight","Brusthöhe"],
 ["bustSpan","Brustpunktabstand"],
 ["neck","Halsumfang"],
 ["armholeDepth","Armlochtiefe"],
 ["waistToHip","Taille–Hüfte"],
 ["hemBelowWaist","Taille–Saum"],
 ["shoulderSlope","Schulterabfall"],
 ["upperArm","Oberarmumfang"],["wrist","Handgelenkumfang"]
];
const P=(x,y)=>[x,y];
const distance=(a,b)=>Math.hypot(a[0]-b[0],a[1]-b[1]);
const area=points=>points.reduce((sum,p,i)=>{
  const q=points[(i+1)%points.length];
  return sum+p[0]*q[1]-q[0]*p[1];
},0)*.5;
const pointLine=(p1,p2,t)=>P(p1[0]*(1-t)+p2[0]*t,p1[1]*(1-t)+p2[1]*t);
const fmt=n=>Number(n.toFixed(3));
function measureValues(input={}) {
  const defaults={chest:100,waist:84,hip:102,shoulder:44,ease:8,...DEFAULTS};
  const result={};
  for(const [name,v] of Object.entries(defaults)){
    const n=Number(input[name]??v);
    if(!Number.isFinite(n)||n<=0||n>300) throw Error("Ungültiges Körpermaß: "+name+" (cm).");
    result[name]=n*10;
  }
  if(result.shoulder>result.chest*.8 || result.neck>result.shoulder*1.25)
    throw Error("Körpermaße prüfen: Schulter-, Hals- und Brustumfang passen nicht zusammen.");
  if(result.bustHeight>=result.frontLength-40)
    throw Error("Brusthöhe muss deutlich oberhalb der Taille liegen.");
  if(result.waistToHip>=result.hemBelowWaist)
    throw Error("Die Hüftlinie muss oberhalb des Saums liegen.");
  if(result.armholeDepth>result.frontLength*.83)
    throw Error("Die Armlochtiefe ist im Verhältnis zur Vorderlänge zu groß.");
  if(result.upperArm<=result.wrist || result.upperArm>result.chest*.8)
    throw Error("Oberarmumfang und Handgelenkumfang überprüfen.");
  if(result.bustSpan>=result.chest*.75)
    throw Error("Brustpunktabstand im Verhältnis zum Brustumfang prüfen.");
  return result;
}
class Shape {
  constructor(){this.points=[];this.commands=[];this.current=null;}
  move(x,y) {this.current=P(x,y);this.points.push(this.current);this.commands.push("M "+fmt(x)+" "+fmt(y));return this;}
  line(x,y){
    const to=P(x,y);
    if(distance(to,this.current)>1e-8){this.points.push(to);this.commands.push("L "+fmt(x)+" "+fmt(y));}
    this.current=to;return this;
  }
  cubic(c1,c2,end,segments=18){
    const start=this.current;
    for(let i=1;i<=segments;i++){
      const t=i/segments,u=1-t;
      this.points.push(P(
        u*u*u*start[0]+3*u*u*t*c1[0]+3*u*t*t*c2[0]+t*t*t*end[0],
        u*u*u*start[1]+3*u*u*t*c1[1]+3*u*t*t*c2[1]+t*t*t*end[1]
      ));
    }
    this.commands.push("C "+c1.map(fmt).join(" ")+" "+c2.map(fmt).join(" ")+" "+end.map(fmt).join(" "));
    this.current=end;return this;
  }
  close(){this.commands.push("Z");return this;}
  path(){return this.commands.join(" ");}
}
function convexLikeOffset(points,mm){
  if(mm===0)return points.map(p=>[...p]);
  // Miter limit for local preview; not a production seam-allowance algorithm.
  const sign=Math.sign(area(points)),n=points.length,edges=[];
  if(!sign)throw Error("Kontur ohne Fläche.");
  for(let i=0;i<n;i++){
    const a=points[i],b=points[(i+1)%n];
    const dx=b[0]-a[0],dy=b[1]-a[1],len=Math.hypot(dx,dy);
    if(len<1e-7)throw Error("Nullkante in Schnittkontur.");
    const normal=P(sign*dy/len*mm,-sign*dx/len*mm);
    edges.push([P(a[0]+normal[0],a[1]+normal[1]),P(b[0]+normal[0],b[1]+normal[1])]);
  }
  const result=[];
  for(let i=0;i<n;i++){
    const prev=edges[(i+n-1)%n],cur=edges[i],
      d=P(prev[1][0]-prev[0][0],prev[1][1]-prev[0][1]),
      e=P(cur[1][0]-cur[0][0],cur[1][1]-cur[0][1]);
    const determinant=d[0]*e[1]-d[1]*e[0];
    let hit=cur[0];
    if(Math.abs(determinant)>1e-8){
      const q=P(cur[0][0]-prev[0][0],cur[0][1]-prev[0][1]);
      const t=(q[0]*e[1]-q[1]*e[0])/determinant;
      hit=P(prev[0][0]+t*d[0],prev[0][1]+t*d[1]);
      if(distance(hit,points[i])>Math.max(mm*5,30))
        hit=P((prev[1][0]+cur[0][0])/2,(prev[1][1]+cur[0][1])/2);
    }
    result.push(hit);
  }
  return result;
}
function dart(x,y,half,up,down){
  return [P(x,y-up),P(x+half,y),P(x,y+down),P(x-half,y)];
}
function createPiece(side,m){
  const front=side==="front";
  const waistY=475;
  const topY=waistY-(front?m.frontLength:m.backWaist);
  const bustY=waistY-m.frontLength+m.bustHeight;
  const armY=topY+m.armholeDepth;
  const hipY=waistY+m.waistToHip;
  const hemY=waistY+m.hemBelowWaist;
  const chestW=(m.chest+m.ease)/4*(front?1.008:.992);
  const waistDart=front?Math.max(20,m.chest*.029):Math.max(16,m.chest*.025);
  const targetWaist=(m.waist+m.ease)/4;
  const waistSide=targetWaist+waistDart/2;
  const hipSide=(m.hip+m.ease)/4;
  const neckW=m.neck/5.75;
  const neckD=front?m.neck/5.4+13:m.neck/17+7;
  const shoulderEndX=Math.max(neckW+90,Math.min(chestW-17,m.shoulder/2));
  const slope=m.shoulderSlope;
  const shoulderStart=P(neckW,topY);
  const shoulderEnd=P(shoulderEndX,topY+slope);
  const underarm=P(chestW,armY);
  const burst=P(chestW,bustY);
  const waistSideP=P(waistSide,waistY);
  const hipSideP=P(hipSide,hipY);
  const shape=new Shape();
  shape.move(0,topY+neckD)
    .cubic(P(neckW*.28,topY+neckD),P(neckW*.66,topY+neckD*.03),shoulderStart,24)
    .line(...shoulderEnd)
    .cubic(
      P(shoulderEndX+8,topY+slope+armY*.12),
      P(chestW-47,armY-12),underarm,30
    )
    .cubic(P(chestW+1,armY+24),P(chestW+2,bustY-15),burst,12)
    .cubic(P(chestW-7,bustY+72),P(waistSide+8,waistY-60),waistSideP,24)
    .cubic(P(waistSide-6,waistY+63),P(hipSide-6,hipY-55),hipSideP,24)
    .cubic(P(hipSide,hipY+20),P(hipSide,hemY-35),P(hipSide,hemY),14)
    .line(0,hemY)
    .close();
  const bustPoint=P(Math.min(chestW*.53,m.bustSpan/2),bustY);
  const waistDartX=front?bustPoint[0]:Math.min(chestW*.54,m.bustSpan*.56);
  const waistDartShape=dart(waistDartX,waistY,waistDart/2,front?145:165,Math.min(170,(hipY-waistY)*.8));
  const shoulderDartX=neckW+(shoulderEndX-neckW)*(front?.51:.57);
  const shoulderIntake=front?Math.max(19,m.chest*.033):Math.max(10,m.chest*.014);
  const shoulderY=x=>topY+(x-neckW)/(shoulderEndX-neckW)*slope;
  const shoulderDart={
    legs:[P(shoulderDartX-shoulderIntake/2,shoulderY(shoulderDartX-shoulderIntake/2)),
          P(shoulderDartX+shoulderIntake/2,shoulderY(shoulderDartX+shoulderIntake/2))],
    apex:front?P(bustPoint[0],bustPoint[1]-12):P(shoulderDartX-15,topY+165)
  };
  return {
    name:front?"VT – Vorderteil":"RT – Rückenteil",
    outline:shape.points,
    svgPath:shape.path(),
    count:1,fold:true,notes:front?
      "Mitte vorn im Stoffbruch, Schulter-/Brustabnäher, Taillenabnäher":
      "Mitte hinten im Stoffbruch, Schulter- und Taillenabnäher",
    side, points:{topY,bustY,armY,waistY,hipY,hemY,bustPoint,shoulderStart,shoulderEnd},
    darts:{waist:waistDartShape,shoulder:shoulderDart},
    grain:P(chestW*.62,0),
    sewnWaistQuarter:waistSide-waistDart,
    bustQuarter:chestW,
    hipQuarter:hipSide
  };
}

function createSleeve(m) {
  const bicepWidth=m.upperArm+m.ease*.25;
  const cuffWidth=m.wrist+Math.max(45,m.ease*.22);
  const capHeight=Math.min(Math.max(m.armholeDepth*.60,105),190);
  const length=m.sleeve;
  if(length<=capHeight+110)throw Error("Die Armlänge ist für die Ärmelkugel zu kurz.");
  const center=bicepWidth*.5;
  const shape=new Shape();
  // Two asymmetric cap curves approximate front and back armscyes.
  shape.move(0,capHeight)
    .cubic(P(bicepWidth*.10,capHeight*.38),P(center*.50,capHeight*.08),P(center,0),24)
    .cubic(P(center+bicepWidth*.23,-1),P(bicepWidth*.93,capHeight*.50),P(bicepWidth,capHeight),24)
    .cubic(P(bicepWidth*.94,capHeight+120),P(center+cuffWidth*.55,length-85),P(center+cuffWidth*.5,length),20)
    .line(center-cuffWidth*.5,length)
    .cubic(P(center-cuffWidth*.55,length-85),P(bicepWidth*.06,capHeight+120),P(0,capHeight),20)
    .close();
  return {
    name:"Ärmel – langer Ärmel",count:2,fold:false,
    notes:"2 Ärmel; Armkugellänge/Einhalteweite am Probeteil prüfen",
    outline:shape.points,svgPath:shape.path(),side:"sleeve",
    grain:P(center,0),points:{bicepY:capHeight,hemY:length},
    notches:[P(bicepWidth*.21,capHeight*.27),P(bicepWidth*.81,capHeight*.31)]
  };
}
function draft(measurements={}){
  const m=measureValues(measurements);
  const front=createPiece("front",m),back=createPiece("back",m);
  return {pieces:[front,back,createSleeve(m)],measurements:m,kind:"top",construction:"bodice-v2"};
}
function esc(s){return String(s).replace(/[&<>"]/g,x=>({"&":"&amp;","<":"&lt;",">":"&gt;",'"':"&quot;"}[x]));}
function svgPoly(points,color,extra=""){
  return '<polygon points="'+points.map(p=>fmt(p[0])+","+fmt(p[1])).join(" ")+'" fill="none" stroke="'+color+'" '+extra+'/>';
}
function techPiece(piece,dx,mirror,allowance){
  const cx=mirror?Math.max(...piece.outline.map(p=>p[0])):0;
  const tx=mirror?dx+cx:dx;
  const transform='translate('+fmt(tx)+' 0)'+(mirror?' scale(-1 1)':'');
  const out=[];
  const contour=convexLikeOffset(piece.outline,allowance);
  out.push('<g transform="'+transform+'">');
  out.push('<path d="'+piece.svgPath+'" fill="#264554" fill-opacity="0.22" stroke="#75d3d6" stroke-dasharray="4 2" stroke-width="1.4" stroke-linejoin="round"/>');
  if(allowance>0)out.push(svgPoly(contour,"#efa16f",'stroke-width="1.6" stroke-linejoin="round"'));
  const D=piece.darts.waist;
  out.push(svgPoly(D,"#f29a87",'stroke-width="1.3" stroke-linejoin="round"'));
  const sd=piece.darts.shoulder;
  for(const leg of sd.legs)out.push('<path d="M '+fmt(leg[0])+' '+fmt(leg[1])+' L '+fmt(sd.apex[0])+' '+fmt(sd.apex[1])+'" fill="none" stroke="#f29a87" stroke-width="1.4"/>');
  const g=piece.grain[0],top=piece.points.bustY+60,bottom=piece.points.hemY-65;
  out.push('<path d="M '+fmt(g)+' '+fmt(top)+' V '+fmt(bottom)+'" stroke="#68c6d5" stroke-width="1.5" fill="none"/>');
  out.push('<path d="M '+fmt(g-6)+' '+fmt(bottom-13)+' L '+fmt(g)+' '+fmt(bottom)+' L '+fmt(g+6)+' '+fmt(bottom-13)+'" fill="none" stroke="#68c6d5" stroke-width="2"/>');
  out.push('<circle cx="'+fmt(piece.points.bustPoint[0])+'" cy="'+fmt(piece.points.bustPoint[1])+'" r="2.6" fill="#ef9f74"/>');
  out.push('</g>');
  const labelX=mirror?dx+cx*.5:dx+cx*.5+110;
  out.push('<text x="'+fmt(labelX)+'" y="'+fmt(piece.points.hemY-35)+'" text-anchor="middle" font-size="24" fill="#dcebf3" font-weight="700">'+(mirror?'RT':'VT')+'</text>');
  out.push('<text x="'+fmt(labelX+34)+'" y="'+fmt(piece.points.waistY+120)+'" font-size="13" fill="#68c6d5" transform="rotate(-90 '+fmt(labelX+34)+' '+fmt(piece.points.waistY+120)+')">FDL</text>');
  return out.join("");
}

function techSleeve(piece,dx,dy,allowance) {
  const out=[],contour=convexLikeOffset(piece.outline,allowance);
  out.push('<g transform="translate('+fmt(dx)+' '+fmt(dy)+')">');
  out.push('<path d="'+piece.svgPath+'" fill="#264554" fill-opacity=".22" stroke="#75d3d6" stroke-dasharray="4 2" stroke-width="1.4" stroke-linejoin="round"/>');
  if(allowance>0)out.push(svgPoly(contour,"#efa16f",'stroke-width="1.6" stroke-linejoin="round"'));
  const center=piece.grain[0],cap=piece.points.bicepY,hem=piece.points.hemY;
  out.push('<path d="M '+fmt(center)+' '+fmt(cap+80)+' V '+fmt(hem-50)+'" stroke="#68c6d5" stroke-width="1.5"/>');
  out.push('<path d="M '+fmt(center-6)+' '+fmt(hem-63)+' L '+fmt(center)+' '+fmt(hem-50)+' L '+fmt(center+6)+' '+fmt(hem-63)+'" stroke="#68c6d5" stroke-width="1.5" fill="none"/>');
  out.push('<path d="M -12 '+fmt(cap)+' H '+fmt(Math.max(...piece.outline.map(p=>p[0]))+12)+'" stroke="#53768c" stroke-dasharray="4 5" stroke-width=".65" opacity=".6"/>');
  for(const [index,pt]of piece.notches.entries()){
    out.push('<circle cx="'+fmt(pt[0])+'" cy="'+fmt(pt[1])+'" r="'+(index===0?3.4:2.6)+'" fill="#ef9f74"/>');
  }
  out.push('<text x="'+fmt(center)+'" y="'+fmt(hem-22)+'" text-anchor="middle" font-size="22" font-weight="700" fill="#dcebf3">ÄR</text>');
  out.push('<text x="'+fmt(center+24)+'" y="'+fmt((cap+hem)/2)+'" text-anchor="middle" font-size="12" fill="#68c6d5" transform="rotate(-90 '+fmt(center+24)+' '+fmt((cap+hem)/2)+')">FDL</text>');
  out.push('</g>');
  return out.join("");
}
function render(measurements={},allowance=10,density=210){
  const m=measureValues(measurements);
  allowance=Number(allowance);density=Number(density);
  if(!Number.isFinite(allowance)||allowance<0||allowance>80)throw Error("Nahtzugabe: 0–80 mm");
  if(!Number.isFinite(density)||density<1||density>5000)throw Error("Flächengewicht: 1–5000 g/m²");
  const d=draft(measurements),front=d.pieces[0],back=d.pieces[1],sleeve=d.pieces[2];
  const frontDX=75,frontWidth=Math.max(...front.outline.map(p=>p[0]));
  const backDX=frontDX+frontWidth+100,backWidth=Math.max(...back.outline.map(p=>p[0]));
  const sleeveDX=backDX+backWidth+100,sleeveDY=95;
  const sleeveWidth=Math.max(...sleeve.outline.map(p=>p[0]));
  const bodyRight=backDX+backWidth;
  const width=sleeveDX+sleeveWidth+95;
  const maxY=Math.max(front.points.hemY,back.points.hemY,sleeve.points.hemY+sleeveDY);
  const height=maxY+90;
  let areaSewn=0,areaCut=0;
  for(const part of d.pieces){
    areaSewn+=Math.abs(area(part.outline))*part.count;
    areaCut+=Math.abs(area(convexLikeOffset(part.outline,allowance)))*part.count;
  }
  const mass=areaCut*density/1e6;
  const count=d.pieces.reduce((n,p)=>n+p.count,0);
  const out=['<svg xmlns="http://www.w3.org/2000/svg" width="'+fmt(width)+'mm" height="'+fmt(height)+'mm" viewBox="0 0 '+fmt(width)+' '+fmt(height)+'" data-bodice-version="2" data-pieces="'+count+'" data-sleeves="2" data-sewn-area="'+fmt(areaSewn)+'" data-cut-area="'+fmt(areaCut)+'" data-mass="'+fmt(mass)+'">'];
  out.push('<rect width="100%" height="100%" fill="#14202e"/>');
  const lines=[["Brust",front.points.bustY],["Taille",front.points.waistY],["Hüfte",front.points.hipY],["Saum",front.points.hemY]];
  for(const [name,y]of lines){
    out.push('<path d="M 20 '+fmt(y)+' H '+fmt(bodyRight+25)+'" stroke="#53768c" stroke-width=".65" stroke-dasharray="4 5" opacity=".62"/>');
    out.push('<text x="'+fmt(bodyRight+29)+'" y="'+fmt(y-6)+'" font-size="12" fill="#a3b9c9">'+name+'</text>');
  }
  out.push(techPiece(front,frontDX,false,allowance));
  out.push(techPiece(back,backDX,true,allowance));
  out.push(techSleeve(sleeve,sleeveDX,sleeveDY,allowance));
  out.push('<text x="'+fmt(frontDX)+'" y="30" font-size="17" font-weight="700" fill="#e4eff7">TEXTILLAB · OBERTEIL · VT / RT / ÄRMEL</text>');
  out.push('<text x="'+fmt(frontDX)+'" y="51" font-size="12" fill="#9bafc1">Türkis: Nahtlinie · Orange: Zuschnitt · FDL / Abnäher / Referenzlinien</text>');
  out.push('<text x="'+fmt(sleeveDX)+'" y="'+fmt(sleeveDY-35)+'" font-size="14" font-weight="600" fill="#dcebf3">ÄR · 2× zuschneiden</text>');
  out.push('<text x="'+fmt(frontDX)+'" y="'+fmt(height-24)+'" font-size="11" fill="#9bafc1">KONZEPT-GRUNDSCHNITT · Armkugellänge und Abnäher vor Fertigung prüfen</text>');
  out.push('</svg>');
  return {
    svg:out.join("\n"),pieces:count,sewnArea:areaSewn,cutArea:areaCut,mass,
    parts:d.pieces.map(({name,outline,count,fold,notes})=>({name,outline,count,fold,notes})),
    construction:d
  };
}
const api={defaults:DEFAULTS,extraFields:EXTENDED,draft,render,measureValues};
if(typeof module!=="undefined"&&module.exports)module.exports=api;
root.TextilLabBodiceV2=api;
})(typeof globalThis!=="undefined"?globalThis:this);

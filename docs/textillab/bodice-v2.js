/* TextilLab Bodice V2 -- front (VT) and back (RT) drafting concept.
 * Units: user inputs in cm, geometry in mm.
 * Technical illustration only. NOT a fitted or production-approved block.
 */
(function(root) {
"use strict";
const DEFAULTS={
  frontLength:46, backWaist:43, bustHeight:27, bustSpan:19,
  neck:38, armholeDepth:23, waistToHip:20, hemBelowWaist:29,
  shoulderSlope:3
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
 ["shoulderSlope","Schulterabfall"]
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
function draft(measurements={}){
  const m=measureValues(measurements);
  const front=createPiece("front",m),back=createPiece("back",m);
  return {pieces:[front,back],measurements:m,kind:"top",construction:"bodice-v2"};
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
  out.push('<path d="'+piece.svgPath+'" fill="#cb6953" fill-opacity="0.04" stroke="#a85145" stroke-width="1.8" stroke-linejoin="round"/>');
  if(allowance>0)out.push(svgPoly(contour,"#e5a16c",'stroke-width=".85" stroke-dasharray="5 3" stroke-linejoin="round"'));
  const D=piece.darts.waist;
  out.push(svgPoly(D,"#b9594d",'stroke-width="1.3" stroke-linejoin="round"'));
  const sd=piece.darts.shoulder;
  for(const leg of sd.legs)out.push('<path d="M '+fmt(leg[0])+' '+fmt(leg[1])+' L '+fmt(sd.apex[0])+' '+fmt(sd.apex[1])+'" fill="none" stroke="#b9594d" stroke-width="1.4"/>');
  const g=piece.grain[0],top=piece.points.bustY+60,bottom=piece.points.hemY-65;
  out.push('<path d="M '+fmt(g)+' '+fmt(top)+' V '+fmt(bottom)+'" stroke="#4d8c9d" stroke-width="1.5" fill="none"/>');
  out.push('<path d="M '+fmt(g-6)+' '+fmt(bottom-13)+' L '+fmt(g)+' '+fmt(bottom)+' L '+fmt(g+6)+' '+fmt(bottom-13)+'" fill="none" stroke="#4d8c9d" stroke-width="2"/>');
  out.push('<circle cx="'+fmt(piece.points.bustPoint[0])+'" cy="'+fmt(piece.points.bustPoint[1])+'" r="2.6" fill="#8d4037"/>');
  out.push('</g>');
  const labelX=mirror?dx+cx*.5:dx+cx*.5+110;
  out.push('<text x="'+fmt(labelX)+'" y="'+fmt(piece.points.hemY-35)+'" text-anchor="middle" font-size="24" fill="#304a53" font-weight="700">'+(mirror?'RT':'VT')+'</text>');
  out.push('<text x="'+fmt(labelX+34)+'" y="'+fmt(piece.points.waistY+120)+'" font-size="13" fill="#4d8c9d" transform="rotate(-90 '+fmt(labelX+34)+' '+fmt(piece.points.waistY+120)+')">FDL</text>');
  return out.join("");
}
function render(measurements={},allowance=10,density=210){
  const m=measureValues(measurements);
  allowance=Number(allowance);density=Number(density);
  if(!Number.isFinite(allowance)||allowance<0||allowance>80)throw Error("Nahtzugabe: 0–80 mm");
  if(!Number.isFinite(density)||density<1||density>5000)throw Error("Flächengewicht: 1–5000 g/m²");
  const d=draft(measurements),front=d.pieces[0],back=d.pieces[1];
  const frontDX=75,backDX=515,backWidth=Math.max(...back.outline.map(p=>p[0]));
  const maxY=Math.max(front.points.hemY,back.points.hemY);
  const width=backDX+backWidth+115,height=maxY+80;
  let areaSewn=0,areaCut=0;
  const out=['<svg xmlns="http://www.w3.org/2000/svg" width="'+fmt(width)+'mm" height="'+fmt(height)+'mm" viewBox="0 0 '+fmt(width)+' '+fmt(height)+'" data-bodice-version="2"'];
  for(const piece of d.pieces){
    areaSewn+=Math.abs(area(piece.outline));
    areaCut+=Math.abs(area(convexLikeOffset(piece.outline,allowance)));
  }
  const mass=areaCut*density/1e6;
  out[0]+=' data-pieces="2" data-sewn-area="'+fmt(areaSewn)+'" data-cut-area="'+fmt(areaCut)+'" data-mass="'+fmt(mass)+'">';
  out.push('<rect width="100%" height="100%" fill="#d9c4a5"/>');
  const lines=[
    ["Brust",front.points.bustY],
    ["Taille",front.points.waistY],
    ["Hüfte",front.points.hipY],
    ["Saum",front.points.hemY]
  ];
  for(const [name,y] of lines){
    out.push('<path d="M 20 '+fmt(y)+' H '+fmt(width-70)+'" stroke="#776b5c" stroke-width=".65" opacity=".48"/>');
    out.push('<text x="'+fmt(width-60)+'" y="'+fmt(y-5)+'" font-size="13" fill="#52616c">'+name+'</text>');
  }
  out.push(techPiece(front,frontDX,false,allowance));
  out.push(techPiece(back,backDX,true,allowance));
  out.push('<text x="75" y="29" font-size="15" font-weight="700" fill="#3b454e">TEXTILLAB · VT / RT GRUNDSCHNITT v2</text>');
  out.push('<text x="75" y="49" font-size="11" fill="#62584e">Abnäher, Hilfslinien &amp; Fadenlauf · Maße in mm · KONZEPT-GRUNDSCHNITT · keine freigegebene Fertigungsvorlage</text>');
  out.push('</svg>');
  return {
    svg:out.join("\n"),pieces:2,sewnArea:areaSewn,cutArea:areaCut,mass,
    parts:d.pieces.map(({name,outline,count,fold,notes})=>({name,outline,count,fold,notes})),
    construction:d
  };
}
const api={defaults:DEFAULTS,extraFields:EXTENDED,draft,render,measureValues};
if(typeof module!=="undefined"&&module.exports)module.exports=api;
root.TextilLabBodiceV2=api;
})(typeof globalThis!=="undefined"?globalThis:this);

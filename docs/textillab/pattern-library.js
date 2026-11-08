/* TextilLab v0.6 | Parametric clothing starter blocks.
 * These are geometric drafting CONCEPTS, not fitted, production-ready patterns.
 * Geometry in mm; input measurements in cm in the browser UI.
 */
(function(root) {
"use strict";
const p=(x,y)=>[x,y], PI=Math.PI;
const BodiceV2=(typeof module!=="undefined"&&module.exports)?require("./bodice-v2.js"):root.TextilLabBodiceV2;
const PantsSystem=(typeof module!=="undefined"&&module.exports)?require("./pants-system.js"):root.TextilLabPantsSystem;
const D={
 chest:100,waist:84,hip:102,back:65,shoulder:44,sleeve:62,
 inseam:80,head:57,hand:22,handLength:19, ease:8
};
const TYPES=[
 ["top","Oberteil / T-Shirt"],["jacket","Jacke"],["trousers","Hose"],
 ["glove","Handschuh"],["beanie","Mütze"],["hat","Hut"],["skirt","Rock"]
];
const clamp=(n,lo,hi)=>Math.max(lo,Math.min(hi,n));
function values(m) {
 const v={};
 for(const [key,def] of Object.entries(D)) {
   let n=Number(m?.[key] ?? def);
   if(!Number.isFinite(n)||n<=0 || n>300) throw Error("Ungültiges Körpermaß: "+key);
   v[key]=n*10;
 }
 if(v.shoulder > v.chest*0.8 || v.waist>v.hip*1.6)
   throw Error("Bitte Körpermaße überprüfen (Schulter-/Taillenverhältnis).");
 return v;
}
function quad(out,a,b,c,count=6) {
 for(let i=1;i<=count;i++) {
   const t=i/count,u=1-t;
   out.push(p(u*u*a[0]+2*u*t*b[0]+t*t*c[0],u*u*a[1]+2*u*t*b[1]+t*t*c[1]));
 }
}
function circle(cx,cy,r,n=40) {
 return Array.from({length:n},(_,i)=>p(cx+Math.cos(i*2*PI/n)*r,cy+Math.sin(i*2*PI/n)*r));
}
function piece(name,outline,count=1,fold=false,notes="",holes=[]) {
 return {name,outline,count,fold,notes,holes};
}
function bodice(v,front,coat=false) {
 const width=(v.chest+v.ease*(coat?1.7:1))/4;
 const waist=(v.waist+v.ease*(coat?1.7:1))/4;
 const length=v.back*(coat?1.16:1);
 const neckW=v.chest*.077;
 const neckD=front?v.chest*.092:v.chest*.029;
 const armD=v.chest*.225+(coat?22:0);
 const shoulder=clamp(v.shoulder/2-neckW,52,170);
 const a=[p(0,neckD)];
 quad(a,p(0,neckD),p(neckW*.55,neckD*.45),p(neckW,0),5);
 a.push(p(neckW+shoulder,21));
 quad(a,p(neckW+shoulder,21),p(width+20,armD*.45),p(width,armD),8);
 a.push(p(waist,length),p(0,length));
 return a;
}
function sleeve(v,coat=false) {
 const arm=(v.chest+v.ease)/3.25+(coat?32:0);
 const length=v.sleeve;
 const cap=clamp(v.chest*.115,105,170);
 const wrist=v.hand*1.5+(coat?30:0);
 const a=[p(0,cap)];
 quad(a,p(0,cap),p(arm*.12,cap*.12),p(arm*.5,0),6);
 quad(a,p(arm*.5,0),p(arm*.86,cap*.12),p(arm,cap),6);
 a.push(p(arm*.5+wrist*.5,length),p(arm*.5-wrist*.5,length));
 return a;
}
function gloveOutline(v) {
 const w=v.hand*.55,h=v.handLength;
 const a=[p(0,h),p(0,h*.31)];
 const tips=[.29,.115,.06,.19];
 for(let i=0;i<4;i++) {
   const x0=i*w/4,x1=(i+1)*w/4,yt=h*tips[i],valley=h*.40;
   a.push(p(x0+2,yt+5));
   quad(a,p(x0+2,yt+5),p((x0+x1)*.5,yt-12),p(x1-2,yt+5),5);
   if(i<3) a.push(p(x1-1,valley));
 }
 a.push(p(w,h));
 return a;
}
function beanie(v) {
 const circumference=v.head+v.ease*.1;
 const width=circumference/4;
 const height=clamp(v.head*.32,165,260);
 const a=[p(0,height),p(width,height),p(width*.93,height*.60)];
 quad(a,p(width*.93,height*.60),p(width*.85,height*.1),p(width*.5,0),8);
 quad(a,p(width*.5,0),p(width*.13,height*.1),p(width*.07,height*.60),8);
 return [
   piece("Mützen-Segment",a,4,false,"4 identische Keile"),
   piece("Bündchen", [p(0,0),p(circumference/2,0),p(circumference/2,75),p(0,75)],2,false,"Bündchen 2× oder im Bruch")
 ];
}
function hat(v) {
 const circ=v.head+v.ease*.12;
 const side=circ/6;
 const radius=circ/(2*PI);
 const wedge=[p(0,135),p(side,135),p(side*.88,48)];
 quad(wedge,p(side*.88,48),p(side*.65,-10),p(side*.5,0),4);
 quad(wedge,p(side*.5,0),p(side*.35,-10),p(side*.12,48),4);
 const top=circle(radius,radius,radius,48);
 const outer=circle(radius+70,radius+70,radius+70,64);
 const inner=circle(radius+70,radius+70,radius,64);
 return [
   piece("Kronensegment",wedge,6,false,"6 Segmente; Form nur Konzept"),
   piece("Hutdeckel",top,1,false,"Kreisform; Kronenumfang prüfen"),
   piece("Krempe",outer,1,false,"Ringform mit Innenausschnitt",[inner])
 ];
}
function skirt(v) {
 const waist=(v.waist+v.ease)/4;
 const hip=(v.hip+v.ease)/4;
 const length=v.back*.87;
 const panel=[p(0,0),p(waist,0),p(hip,205),p(hip+35,length),p(0,length)];
 return [
   piece("Rock vorne",panel,1,true,"Mitte im Stoffbruch"),
   piece("Rock hinten",panel,1,true,"Mitte im Stoffbruch; Abnäher fehlen"),
   piece("Bund", [p(0,0),p((v.waist+v.ease)/2,0),p((v.waist+v.ease)/2,90),p(0,90)],2,false,"Bundlänge an Probestück anpassen")
 ];
}
function draft(type,measurements) {
 if(type==="trousers") {
   const generated=PantsSystem.draft(measurements);
   return {type,pieces:generated.pieces,measurements:generated.measurements};
 }
 const v=values(measurements);
 if(type==="top") {
   const result=BodiceV2.draft(measurements);
   return {type,pieces:result.pieces,measurements:v};
 }
 let pieces;
 switch(type) {
 case "top":
   pieces=[
     piece("Vorderteil",bodice(v,true),1,true,"Mitte im Stoffbruch; Hals-/Armloch nur Grundform"),
     piece("Rückenteil",bodice(v,false),1,true,"Mitte im Stoffbruch"),
     piece("Ärmel",sleeve(v),2,false,"Ärmelkugel ist Näherung; Einhalteweite prüfen")
   ];break;
 case "jacket":
   pieces=[
     piece("Jacke vorne",bodice(v,true,true),2,false,"Zwei Vorderteile; Verschlussbeleg noch offen"),
     piece("Jacke hinten",bodice(v,false,true),1,true,"Mitte im Stoffbruch"),
     piece("Jackenärmel",sleeve(v,true),2,false,"Ein-/Zweiteilung später"),
     piece("Kragen", [p(0,0),p(v.chest*.38,0),p(v.chest*.38,95),p(0,95)],1,false,"Platzhalter für späteren geformten Kragen")
   ];break;
 case "glove":
   pieces=[
     piece("Handfläche",gloveOutline(v),2,false,"Ein Teil je Hand; Fingerlängen schätzen"),
     piece("Handrücken",gloveOutline(v),2,false,"Fingerzwischenstücke fehlen"),
     piece("Daumen", [p(0,45),p(v.hand*.12,0),p(v.hand*.24,45),p(v.hand*.21,v.handLength*.53),p(v.hand*.03,v.handLength*.53)],2,false,"Daumeneinsatz; keine vollständige Handschuhkonstruktion")
   ];break;
 case "beanie":pieces=beanie(v);break;
 case "hat":pieces=hat(v);break;
 case "skirt":pieces=skirt(v);break;
 default:throw Error("Unbekannter Grundschnitt");
 }
 return {type,pieces,measurements:v};
}
function area(poly){return poly.reduce((s,a,i)=>{const b=poly[(i+1)%poly.length];return s+a[0]*b[1]-b[0]*a[1]},0)*.5;}
function offset(poly,mm) {
 if(mm===0) return poly.map(q=>[...q]);
 const sign=Math.sign(area(poly)),lines=[];
 for(let i=0;i<poly.length;i++) {
   const a=poly[i],b=poly[(i+1)%poly.length],dx=b[0]-a[0],dy=b[1]-a[1],len=Math.hypot(dx,dy);
   if(len<1e-7) throw Error("Nullkante im Schnittteil");
   const nx=sign*dy/len*mm,ny=-sign*dx/len*mm;
   lines.push([[a[0]+nx,a[1]+ny],[b[0]+nx,b[1]+ny]]);
 }
 const res=[];
 for(let i=0;i<lines.length;i++) {
   const prev=lines[(i+lines.length-1)%lines.length],cur=lines[i];
   const r=[prev[1][0]-prev[0][0],prev[1][1]-prev[0][1]];
   const s=[cur[1][0]-cur[0][0],cur[1][1]-cur[0][1]];
   const denominator=r[0]*s[1]-r[1]*s[0];
   if(Math.abs(denominator)<1e-8) {res.push(cur[0]);continue;}
   const q=[cur[0][0]-prev[0][0],cur[0][1]-prev[0][1]];
   const t=(q[0]*s[1]-q[1]*s[0])/denominator;
   const candidate=[prev[0][0]+t*r[0],prev[0][1]+t*r[1]];
   const distance=Math.hypot(candidate[0]-poly[i][0],candidate[1]-poly[i][1]);
   if(distance>Math.max(5*mm,30)) {
     res.push([(cur[0][0]+prev[1][0])*.5,(cur[0][1]+prev[1][1])*.5]);
   }else res.push(candidate);
 }
 return res;
}
function render(type,measurements,allowance=10,density=210) {
 if(type==="top") return BodiceV2.render(measurements,allowance,density);
 if(type==="trousers") return PantsSystem.render(measurements,allowance,density);
 const d=draft(type,measurements);
 const seam=Number(allowance),weight=Number(density);
 if(!Number.isFinite(seam)||seam<0||seam>80) throw Error("Nahtzugabe 0–80 mm wählen.");
 if(!Number.isFinite(weight)||weight<1||weight>5000) throw Error("Ungültiges Stoffgewicht.");
 const paths=[];
 let totalSewn=0,totalCut=0, x=18, y=22, rowH=0,maxW=0;
 const pieces=d.pieces.map(item=>{
   const outer=offset(item.outline,seam);
   const holes=item.holes||[];
   const data=[...item.outline,...outer,...holes.flat()];
   const xs=data.map(pt=>pt[0]),ys=data.map(pt=>pt[1]);
   const minx=Math.min(...xs),maxx=Math.max(...xs),miny=Math.min(...ys),maxy=Math.max(...ys);
   const w=maxx-minx,h=maxy-miny;
   if(x+w>1450 && x>18){x=18;y+=rowH+48;rowH=0}
   const ox=x-minx,oy=y-miny;
   const pts=arr=>arr.map(a=>a.map(v=>v.toFixed(3)).join(",")).join(" ");
   let shape='<g transform="translate('+ox.toFixed(3)+' '+oy.toFixed(3)+')">';
   shape+='<polygon points="'+pts(outer)+'" fill="#203b4b" fill-opacity=".65" stroke="#efa16f" stroke-width="1.05"/>';
   shape+='<polygon points="'+pts(item.outline)+'" fill="none" stroke="#72d7db" stroke-dasharray="4 2" stroke-width=".9"/>';
   if(holes.length)for(const hole of holes){
     shape+='<polygon points="'+pts(hole)+'" fill="#14202e" stroke="#efa16f" stroke-width=".8"/>';
   }
   const title=item.name+' · '+item.count+'×'+(item.fold?' · Stoffbruch':'');
   const escapeHTML=s=>String(s).replace(/[&<>"]/g, a=>({"&":"&amp;","<":"&lt;",">":"&gt;",'"':"&quot;"}[a]));
   shape+='<text x="'+minx.toFixed(2)+'" y="'+(miny-6).toFixed(2)+'" font-size="13" fill="#d4e5f3">'+escapeHTML(title)+'</text>';
   shape+='</g>';paths.push(shape);
   const sewn=Math.abs(area(item.outline))-holes.reduce((sum,pt)=>sum+Math.abs(area(pt)),0);
   const cut=Math.abs(area(outer))-holes.reduce((sum,pt)=>sum+Math.abs(area(pt)),0);
   totalSewn+=sewn*item.count;totalCut+=cut*item.count;
   x+=w+46;rowH=Math.max(rowH,h+28);maxW=Math.max(maxW,x);
   return {...item,sewnArea:sewn,cutArea:cut};
 });
 const width=Math.max(600,Math.min(1550,maxW)),height=y+rowH+35;
 const mass=totalCut*weight/1e6;
 const svg='<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 '+width.toFixed(2)+' '+height.toFixed(2)+'" width="'+width.toFixed(2)+'mm" height="'+height.toFixed(2)+'mm" data-pieces="'+d.pieces.reduce((sum,p)=>sum+p.count,0)+'" data-sewn-area="'+totalSewn.toFixed(3)+'" data-cut-area="'+totalCut.toFixed(3)+'" data-mass="'+mass.toFixed(3)+'">'+
 '<rect width="100%" height="100%" fill="#14202e"/>'+paths.join('')+
 '<text x="18" y="'+(height-11)+'" font-size="12" fill="#edb36d">KONZEPT-GRUNDSCHNITT: Nicht ohne Anprobe/Validierung zuschneiden</text></svg>';
 return {svg,pieces:d.pieces.reduce((sum,p)=>sum+p.count,0),sewnArea:totalSewn,cutArea:totalCut,mass,parts:pieces};
}
const api={defaults:D,types:TYPES,draft,render};
if(typeof module!=="undefined"&&module.exports)module.exports=api;
root.TextilLabPatternLibrary=api;
})(typeof globalThis!=="undefined"?globalThis:this);

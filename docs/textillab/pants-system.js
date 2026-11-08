/* TextilLab v1.1 – independent pants construction rules and technical diagram.
 * Original experimental drafting geometry: NOT Müller & Sohn or a reproduction
 * of any printed book. Values in cm at input; geometry in mm.
 * Pattern must be fitted and seam allowances validated before cutting.
 */
(function(root) {
  "use strict";
  const DEFAULTS = Object.freeze({
    waist:84, hip:102, inseam:80, outseam:108,
    crotchDepth:28, waistToHip:20, waistToKnee:60,
    thigh:64, knee:48, hem:44, hipEase:4, waistEase:2,
    dartIntake:2, crotchExtension:0
  });
  const FIELDS = [
    ["outseam","Seitenlänge"],["crotchDepth","Leibhöhe / Sitztiefe"],
    ["waistToHip","Taille–Hüfte"],["waistToKnee","Taille–Knie"],
    ["thigh","Oberschenkelumfang"],["knee","Knieumfang"],
    ["hem","Saumumfang je Bein"],["hipEase","Hüft-Zugabe"],
    ["waistEase","Taillen-Zugabe"],["dartIntake","Hinterer Abnäher"],
    ["crotchExtension","Zusätzliche Schrittausladung"]
  ];
  const RULESET = Object.freeze({
    id:"textillab.pants.experimental.1",
    label:"TextilLab Hose – eigenständiger Konstruktionsprototyp",
    status:"experimental", reference:"TextilLab original formulas, 2026",
    documentation:"docs/TEXTILLAB_PANTS_RULES_V11_DE.md"
  });
  const fmt = n => Number(n.toFixed(3));
  const point = (x,y) => [x,y];
  const length = (a,b) => Math.hypot(a[0]-b[0],a[1]-b[1]);
  const area = poly => poly.reduce((n,p,i)=>{
    const q=poly[(i+1)%poly.length];return n+(p[0]*q[1]-q[0]*p[1])/2;
  },0);
  const pathLength = (pts,start,end) => {
    let n=0;for(let i=start+1;i<=end;i++)n+=length(pts[i-1],pts[i]);return n;
  };
  function inputs(values={}) {
    const m={};
    for (const [key,v] of Object.entries(DEFAULTS)) {
      const n=Number(values[key]??v);
      if(!Number.isFinite(n))throw Error("Ungültiges Hosenmaß: "+key);
      m[key]=n*10;
    }
    for(const key of ["waist","hip","inseam","outseam","crotchDepth","waistToHip","waistToKnee","thigh","knee","hem"]){
      if(m[key]<100 || m[key]>2000)throw Error("Hosenmaß "+key+" außerhalb des erlaubten Bereichs (10–200 cm).");
    }
    for(const key of ["hipEase","waistEase","dartIntake"]){
      if(m[key]<0 || m[key]>130)throw Error("Zugabe "+key+" muss 0–13 cm sein.");
    }
    if(m.crotchExtension< -20 || m.crotchExtension>70)throw Error("Schrittausladung -2 bis 7 cm einstellen.");
    const yRise=m.outseam-m.inseam;
    if(yRise<190 || yRise>450)throw Error("Seitenlänge minus Schrittlänge ergibt unrealistische Leibhöhe.");
    if(Math.abs(yRise-m.crotchDepth)>50)throw Error("Leibhöhe passt nicht zur Seiten-/Schrittlänge (max. 5 cm Abweichung).");
    if(m.waistToHip>=yRise-35 || m.waistToKnee<=yRise+95 || m.waistToKnee>=m.outseam-110)
      throw Error("Hüft-, Schritt-, Knie- und Saumlinien haben eine falsche Reihenfolge.");
    if(m.hip<m.waist*.67 || m.waist>m.hip*1.35)throw Error("Verhältnis Taille/Hüfte überprüfen.");
    if(m.knee<m.hem*.70 || m.knee>m.thigh*1.2)throw Error("Knie-/Saum-/Oberschenkelumfang überprüfen.");
    return m;
  }
  class Shape {
    constructor(){this.outline=[];this.ops=[];}
    start(p){this.outline.push(p);this.ops.push("M "+fmt(p[0])+" "+fmt(p[1]));return this;}
    line(p){const a=this.outline.at(-1);if(length(a,p)>1e-8)this.outline.push(p);this.ops.push("L "+fmt(p[0])+" "+fmt(p[1]));return this;}
    cubic(b,c,d,steps=24){
      const a=this.outline.at(-1);
      for(let i=1;i<=steps;i++){
        const t=i/steps,u=1-t;
        this.outline.push(point(
          u*u*u*a[0]+3*u*u*t*b[0]+3*u*t*t*c[0]+t*t*t*d[0],
          u*u*u*a[1]+3*u*u*t*b[1]+3*u*t*t*c[1]+t*t*t*d[1]
        ));
      }
      this.ops.push("C "+[...b,...c,...d].map(fmt).join(" "));
      return this;
    }
    close(){this.ops.push("Z");return this;}
    get path(){return this.ops.join(" ");}
  }
  function panel(m,back=false){
    const frontFactor=back?1.02:.98;
    const hipW=(m.hip+m.hipEase)/4*frontFactor;
    const waistQ=(m.waist+m.waistEase)/4;
    const dart=back?m.dartIntake:0;
    const draftedWaistWidth=waistQ*(back?1.02:.98)+dart;
    const waistHalf=draftedWaistWidth/2;
    const hipHalf=hipW/2;
    const rise=m.outseam-m.inseam,kneeY=m.waistToKnee,hipY=m.waistToHip,hemY=m.outseam;
    const ext=(m.hip+m.hipEase)/20+(back?30:0)+m.crotchExtension;
    const creaseX=0;
    const kneeW=m.knee/2+(back?10:-10);
    const hemW=m.hem/2+(back?10:-10);
    const ks=kneeW/2, hs=hemW/2;
    const centerW=point(-waistHalf,back?-26:0),sideW=point(waistHalf,back?5:8);
    const sideHip=point(hipHalf,hipY),centerHip=point(-hipHalf,hipY);
    const sideRise=point(hipHalf+(back?15:12),rise),innerRise=point(-hipHalf-ext,rise);
    const sideKnee=point(ks,kneeY),innerKnee=point(-ks,kneeY);
    const sideHem=point(hs,hemY),innerHem=point(-hs,hemY);
    const poly=new Shape().start(centerW).line(sideW);
    const seamStart=poly.outline.length-1;
    poly.cubic(point(sideW[0]+9,60),point(sideHip[0]-4,hipY-35),sideHip,18)
        .cubic(point(sideHip[0]+3,hipY+35),point(sideRise[0]+5,rise-40),sideRise,16)
        .cubic(point(sideRise[0]-10,rise+105),point(sideKnee[0]+15,kneeY-90),sideKnee,24)
        .line(sideHem);
    const seamEnd=poly.outline.length-1;
    poly.line(innerHem);
    const inseamStart=poly.outline.length-1;
    poly.line(innerKnee)
        .cubic(point(innerKnee[0]-6,kneeY-95),
               point(innerRise[0]+(back?57:35),rise+105),innerRise,24);
    const inseamEnd=poly.outline.length-1;
    poly.cubic(
      point(innerRise[0]+(back?18:21),rise-4),
      point(centerHip[0]-(back?35:17),hipY+53),centerHip,32
    ).cubic(
      point(centerHip[0]+(back?0:3),hipY-45),
      point(centerW[0]-(back?15:0),55),centerW,18
    ).close();
    // Last Bezier point equals first point. Remove duplicate from polygon sampling.
    poly.outline.pop();
    const grain=point(creaseX,0);
    const dartX=(centerW[0]+sideW[0])/2+25;
    const darts=back?[
      point(dartX-dart/2,-8),point(dartX,110),point(dartX+dart/2,-8)
    ]:[];
    const marks={
      WC:centerW, WS:sideW, HS:sideHip, HC:centerHip,
      CR:innerRise, RS:sideRise, KS:sideKnee, KI:innerKnee,
      AS:sideHem, AI:innerHem, GR:grain
    };
    const sewnWaist=pathLength(poly.outline,0,seamStart)-dart;
    return {
      name:back?"Hinterhose":"Vorderhose",id:back?"HH":"VH",
      count:2,fold:false,notes:back?
        "Hintere Schrittkurve und Abnäher – Passform anprobieren":
        "Vordere Schrittkurve, Knie- und Bügelfaltenlinie",
      outline:poly.outline,svgPath:poly.path,darts,marks,
      grainline:point(0,(hipY+hemY)/2),levels:{hip:hipY,rise,knee:kneeY,hem:hemY},
      seams:{
        side:pathLength(poly.outline,seamStart,seamEnd),
        inseam:pathLength(poly.outline,inseamStart,inseamEnd),
        rise:pathLength(poly.outline,inseamEnd,poly.outline.length-1)+length(poly.outline.at(-1),centerW),
        waist:sewnWaist
      }
    };
  }
  function ruleRecords(m,panels){
    const front=panels[0],back=panels[1],rise=m.outseam-m.inseam;
    return [
      {id:"H01",title:"Hüftviertel",formula:"(Hüftumfang + Hüftzugabe) / 4",value:(m.hip+m.hipEase)/4,unit:"mm",depends:["hip","hipEase"],status:"experimental"},
      {id:"H02",title:"Taillenviertel",formula:"(Taillenumfang + Taillenzugabe) / 4",value:(m.waist+m.waistEase)/4,unit:"mm",depends:["waist","waistEase"],status:"experimental"},
      {id:"H03",title:"Schritthöhe",formula:"Seitenlänge − Schrittlänge",value:rise,unit:"mm",depends:["outseam","inseam"],status:"experimental"},
      {id:"H04",title:"Vordere Schrittausladung",formula:"(Hüftumfang + Hüftzugabe) / 20 + Änderung",value:(m.hip+m.hipEase)/20+m.crotchExtension,unit:"mm",depends:["hip","hipEase","crotchExtension"],status:"experimental"},
      {id:"H05",title:"Hintere Schrittausladung",formula:"Vordere Ausladung + 30 mm (Prototyp)",value:(m.hip+m.hipEase)/20+m.crotchExtension+30,unit:"mm",depends:["hip","crotchExtension"],status:"experimental"},
      {id:"H06",title:"Knieverteilung vorne",formula:"Knieumfang / 2 − 10 mm",value:m.knee/2-10,unit:"mm",depends:["knee"],status:"experimental"},
      {id:"H07",title:"Knieverteilung hinten",formula:"Knieumfang / 2 + 10 mm",value:m.knee/2+10,unit:"mm",depends:["knee"],status:"experimental"},
      {id:"H08",title:"Seitennähte |Δ|",formula:"|L(VH) − L(HH)|",value:Math.abs(front.seams.side-back.seams.side),unit:"mm",depends:["outseam","hip","waist"],status:"measured"},
      {id:"H09",title:"Innenbeinnähte |Δ|",formula:"|L(VH) − L(HH)|",value:Math.abs(front.seams.inseam-back.seams.inseam),unit:"mm",depends:["inseam","hip","knee"],status:"measured"},
      {id:"H10",title:"Hinterer Abnäher",formula:"separate Taillen-Formungsreserve",value:m.dartIntake,unit:"mm",depends:["dartIntake"],status:"experimental"}
    ];
  }
  function draft(values={}){
    const m=inputs(values),front=panel(m,false),back=panel(m,true);
    const pieces=[front,back];
    const checks={
      sideDifferenceMm:Math.abs(front.seams.side-back.seams.side),
      inseamDifferenceMm:Math.abs(front.seams.inseam-back.seams.inseam),
      frontCrotchMm:front.seams.rise,backCrotchMm:back.seams.rise,
      frontWaistMm:front.seams.waist,backWaistMm:back.seams.waist,
      thighEaseNotVerified:true
    };
    const warnings=[];
    if(checks.sideDifferenceMm>5)warnings.push("Seitennähte weichen mehr als 5 mm ab – Formung prüfen");
    if(checks.inseamDifferenceMm>8)warnings.push("Innenbeinnähte weichen mehr als 8 mm ab – Einhalteverteilung prüfen");
    warnings.push("Leibhöhe, Sitzkurve und Abnäher sind noch nicht an einem Probeteil geprüft");
    return {pieces,checks,rules:ruleRecords(m,pieces),measurements:m,
      bodyValues:{...DEFAULTS,...values},warnings,system:RULESET};
  }
  function offset(poly,mm) {
    if(!mm)return poly.map(p=>[...p]);
    const sign=Math.sign(area(poly)),lines=[];
    for(let i=0;i<poly.length;i++){
      const a=poly[i],b=poly[(i+1)%poly.length],dx=b[0]-a[0],dy=b[1]-a[1];
      const len=Math.hypot(dx,dy);
      if(len<1e-8)throw Error("Doppelte Konturpunkte verhindern Nahtzugabe");
      const nx=sign*dy/len*mm,ny=-sign*dx/len*mm;
      lines.push([[a[0]+nx,a[1]+ny],[b[0]+nx,b[1]+ny]]);
    }
    return lines.map((cur,i)=>{
      const prev=lines[(i+lines.length-1)%lines.length],r=point(prev[1][0]-prev[0][0],prev[1][1]-prev[0][1]),
        s=point(cur[1][0]-cur[0][0],cur[1][1]-cur[0][1]),d=r[0]*s[1]-r[1]*s[0];
      if(Math.abs(d)<1e-8)return cur[0];
      const q=point(cur[0][0]-prev[0][0],cur[0][1]-prev[0][1]),
        t=(q[0]*s[1]-q[1]*s[0])/d,candidate=point(prev[0][0]+r[0]*t,prev[0][1]+r[1]*t);
      if(length(candidate,poly[i])>Math.max(5*mm,30))
        return point((prev[1][0]+cur[0][0])/2,(prev[1][1]+cur[0][1])/2);
      return candidate;
    });
  }
  const ptString=poly=>poly.map(p=>fmt(p[0])+","+fmt(p[1])).join(" ");
  function render(values={},allowance=10,density=210,options={}){
    const d=draft(values),mm=Number(allowance),g=Number(density);
    if(!Number.isFinite(mm)||mm<0||mm>80)throw Error("Nahtzugabe muss 0–80 mm betragen.");
    if(!Number.isFinite(g)||g<1||g>5000)throw Error("Flächengewicht muss 1–5000 g/m² betragen.");
    const layers={guides:true,labels:true,dimensions:true,points:true,darts:true,...(options.layers||{})};
    const spans=d.pieces.map(part=>{
      const o=offset(part.outline,mm),xs=part.outline.concat(o).map(p=>p[0]),
        ys=part.outline.concat(o).map(p=>p[1]);
      return {part,o,minx:Math.min(...xs),maxx:Math.max(...xs),miny:Math.min(...ys),maxy:Math.max(...ys)};
    });
    const width=spans.reduce((n,e)=>n+e.maxx-e.minx+130,70)+70;
    const height=Math.max(...spans.map(s=>s.maxy-s.miny))+175;
    let x=70,sewn=0,cut=0;
    const lines=['<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 '+fmt(width)+' '+fmt(height)+'" width="'+fmt(width)+'mm" height="'+fmt(height)+'mm" data-pants-system="'+RULESET.id+'" data-pieces="4">','<rect width="100%" height="100%" fill="#14202e"/>'];
    lines.push('<text x="35" y="24" fill="#d9e6ef" font-size="16" font-weight="700">TEXTILLAB · HOSEN-GRUNDSCHNITT · VH / HH</text>');
    lines.push('<text x="35" y="43" fill="#9db4c5" font-size="11">Türkis: Naht · Orange: Zuschnitt · Gelb: Fadenlauf · Rosa: Abnäher</text>');
    for(const s of spans){
      const p=s.part,ox=x-s.minx,oy=65-s.miny;
      lines.push('<g transform="translate('+fmt(ox)+' '+fmt(oy)+')">');
      lines.push('<path d="'+p.svgPath+'" fill="#264554" fill-opacity=".36" stroke="#7cdae1" stroke-width="1.7" stroke-dasharray="4 3" stroke-linejoin="round"/>');
      if(mm>0)lines.push('<polygon points="'+ptString(s.o)+'" fill="none" stroke="#f4a670" stroke-width="1.15" stroke-linejoin="round"/>');
      const extremes=[Math.min(...p.outline.map(v=>v[0])),Math.max(...p.outline.map(v=>v[0]))];
      if(layers.guides){
        const titles={hip:"Hü",rise:"Sch",knee:"Kn",hem:"Sa"};
        for(const [name,y]of Object.entries(p.levels)){
          lines.push('<path d="M '+fmt(extremes[0]-15)+' '+fmt(y)+' H '+fmt(extremes[1]+15)+'" stroke="#4b7892" stroke-dasharray="5 5" stroke-width=".8"/>');
          if(layers.labels)lines.push('<text x="'+fmt(extremes[1]+17)+'" y="'+fmt(y-5)+'" fill="#a8bed0" font-size="10">'+titles[name]+'</text>');
        }
      }
      const gy0=p.levels.rise+48,gy1=p.levels.hem-40;
      lines.push('<path d="M 0 '+fmt(gy0)+' V '+fmt(gy1)+'" stroke="#d4d37c" stroke-width="1.3" stroke-dasharray="10 5"/>');
      lines.push('<path d="M -5 '+fmt(gy1-10)+' L 0 '+fmt(gy1)+' L 5 '+fmt(gy1-10)+'" stroke="#d4d37c" fill="none"/>');
      if(layers.labels)lines.push('<text x="9" y="'+fmt((gy0+gy1)/2)+'" fill="#d4d37c" font-size="11">FDL</text>');
      if(layers.darts && p.darts.length){
        lines.push('<polyline points="'+ptString(p.darts)+'" fill="none" stroke="#fa9fa3" stroke-width="1.6"/>');
      }
      if(layers.points){
        for(const [id,pos]of Object.entries(p.marks)){
          lines.push('<circle cx="'+fmt(pos[0])+'" cy="'+fmt(pos[1])+'" r="2.5" fill="#80d9e0"/>');
          lines.push('<text x="'+fmt(pos[0]+5)+'" y="'+fmt(pos[1]-5)+'" fill="#bedce2" font-size="9">'+id+'</text>');
        }
      }
      if(layers.dimensions){
        const referenceY=p.levels.knee;
        lines.push('<path d="M '+fmt(extremes[0])+' '+fmt(referenceY+24)+' H '+fmt(extremes[1])+'" stroke="#a2dbe5" stroke-width=".9"/>');
        lines.push('<text x="'+fmt(extremes[1]-5)+'" y="'+fmt(referenceY+19)+'" text-anchor="end" fill="#e0f2f6" font-size="11">Kn '+(d.bodyValues.knee/2).toFixed(1)+' cm / Schnittpaar</text>');
        const dx=extremes[1]+34,y0=0,y1=p.levels.hem;
        lines.push('<path d="M '+fmt(dx)+' '+fmt(y0)+' V '+fmt(y1)+'" stroke="#a2dbe5" stroke-width=".9"/>');
        lines.push('<text x="'+fmt(dx+8)+'" y="'+fmt((y0+y1)/2)+'" fill="#e0f2f6" font-size="11">SL '+d.bodyValues.outseam.toFixed(1)+' cm</text>');
      }
      lines.push('</g>');
      if(layers.labels)lines.push('<text x="'+fmt(x+(s.maxx-s.minx)/2)+'" y="59" text-anchor="middle" fill="#e0edf7" font-size="13" font-weight="700">'+p.id+' · '+p.name+' · 2×</text>');
      sewn+=Math.abs(area(p.outline))*p.count;
      cut+=Math.abs(area(s.o))*p.count;
      x+=s.maxx-s.minx+130;
    }
    lines.push('<text x="35" y="'+fmt(height-20)+'" fill="#a4bacb" font-size="10">KONZEPT-GRUNDSCHNITT · Ausführungsformeln sind noch nicht Müller &amp; Sohn-validiert</text>');
    lines.push('</svg>');
    const mass=cut*g/1e6;
    let svg=lines.join("\n");
    svg=svg.replace('data-pieces="4"','data-pieces="4" data-sewn-area="'+fmt(sewn)+'" data-cut-area="'+fmt(cut)+'" data-mass="'+fmt(mass)+'"');
    return {...d,svg,sewnArea:sewn,cutArea:cut,mass,
      pieces:4,parts:d.pieces};
  }
  const api={defaults:DEFAULTS,fields:FIELDS,ruleset:RULESET,inputs,draft,render};
  if(typeof module!=="undefined"&&module.exports)module.exports=api;
  root.TextilLabPantsSystem=api;
})(typeof globalThis!=="undefined"?globalThis:this);

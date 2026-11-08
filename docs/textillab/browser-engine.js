/* TextilLab browser-only geometry preview.
 * This is a compact JavaScript port of the developable-mesh algorithm used
 * for public static demos; the native TextilLab Studio uses the Rust core.
 * Units are mm after applying the OBJ-unit conversion.
 */
(function (root) {
  "use strict";
  const key = (a, b) => Math.min(a, b) + ":" + Math.max(a, b);
  const sub = (a, b) => [a[0] - b[0], a[1] - b[1]];
  const dot = (a, b) => a[0] * b[0] + a[1] * b[1];
  const cross = (a, b) => a[0] * b[1] - a[1] * b[0];
  const norm = v => Math.hypot(v[0], v[1]);
  const dist = (a, b) => Math.hypot(a[0]-b[0],a[1]-b[1],a[2]-b[2]);
  const area = points => points.reduce((sum, p, i) => {
    const q = points[(i + 1) % points.length];
    return sum + (p[0] * q[1] - p[1] * q[0]) / 2;
  }, 0);

  function validate(vertices, faces) {
    if (!faces.length) throw Error("OBJ enthält keine Flächen.");
    if (faces.length > 10000) throw Error("Demo erlaubt maximal 10.000 Dreiecke.");
    if (!vertices.every(v => v.length===3 && v.every(Number.isFinite))) throw Error("Ungültige Koordinaten.");
    const incidence = new Map();
    for (let fi=0; fi<faces.length; fi++) {
      const f=faces[fi];
      if (f.length!==3 || new Set(f).size!==3 || f.some(i=> !Number.isInteger(i) || i < 0 || i>=vertices.length))
        throw Error("Ungültiges Dreieck #" + (fi+1));
      const a=vertices[f[0]], b=vertices[f[1]], c=vertices[f[2]];
      const ab=b.map((x,i)=>x-a[i]), ac=c.map((x,i)=>x-a[i]);
      const cp=[ab[1]*ac[2]-ab[2]*ac[1],ab[2]*ac[0]-ab[0]*ac[2],ab[0]*ac[1]-ab[1]*ac[0]];
      if (Math.hypot(...cp) <= 1e-10) throw Error("Entartetes Dreieck #" + (fi+1));
      for (let j=0;j<3;j++) {
        const e=key(f[j],f[(j+1)%3]);
        if (!incidence.has(e)) incidence.set(e,[]);
        incidence.get(e).push(fi);
        if (incidence.get(e).length>2) throw Error("Nicht-mannigfaltige Netzkante.");
      }
    }
    return incidence;
  }

  function flatten(vertices, faces, seams) {
    const incidence=validate(vertices,faces);
    const cutSet = new Set([...seams].map(pair => {
      const [a,b]=pair.split(":").map(n=>Number(n)-1);
      const k=key(a,b);
      if (!incidence.has(k)) throw Error("Schnittkante " + pair + " existiert nicht.");
      return k;
    }));
    const visited=new Set();
    const patches=[];
    for (let root=0;root<faces.length;root++) {
      if (visited.has(root)) continue;
      const first=faces[root], coords=new Map();
      const a=vertices[first[0]], b=vertices[first[1]], c=vertices[first[2]];
      const ab=dist(a,b),ac=dist(a,c),bc=dist(b,c);
      const x=(ac*ac+ab*ab-bc*bc)/(2*ab);
      coords.set(first[0],[0,0]);coords.set(first[1],[ab,0]);
      coords.set(first[2],[x,Math.sqrt(Math.max(0,ac*ac-x*x))]);
      const queue=[root], faceIDs=[];visited.add(root);
      while(queue.length) {
        const fi=queue.shift(), f=faces[fi];faceIDs.push(fi);
        for(let j=0;j<3;j++) {
          const ia=f[j],ib=f[(j+1)%3],e=key(ia,ib);
          if(cutSet.has(e)) continue;
          const adjacent=incidence.get(e);
          if(adjacent.length!==2) continue;
          const next=adjacent[0]===fi?adjacent[1]:adjacent[0];
          const ic=faces[next].find(v=>v!==ia&&v!==ib);
          const pa=coords.get(ia),pb=coords.get(ib), d=sub(pb,pa),len=norm(d);
          const da=dist(vertices[ia],vertices[ic]), db=dist(vertices[ib],vertices[ic]);
          const projection=(da*da+len*len-db*db)/(2*len);
          const h2=da*da-projection*projection;
          if(h2 < -1e-6) throw Error("Unstimmige Dreiecks-Kantenlängen.");
          const height=Math.sqrt(Math.max(0,h2));
          const opposite=f[(j+2)%3];
          const sign=cross(d,sub(coords.get(opposite),pa))>0?-1:1;
          const candidate=[
            pa[0] + d[0]*projection/len + sign*height*(-d[1]/len),
            pa[1] + d[1]*projection/len + sign*height*( d[0]/len)
          ];
          if(coords.has(ic)) {
            const diff=norm(sub(coords.get(ic),candidate));
            if(diff > Math.max(1e-5,da*1e-7))
              throw Error("Diese Fläche kann nicht dehnungsfrei abgewickelt werden. Weitere Schnittkanten setzen.");
          } else coords.set(ic,candidate);
          if(!visited.has(next)){visited.add(next);queue.push(next);}
        }
      }
      patches.push({faceIDs,coords});
    }
    return patches;
  }

  function boundary(faces, patch) {
    const count=new Map(),adj=new Map();
    for(const fi of patch.faceIDs) {
      const f=faces[fi];
      for(let j=0;j<3;j++) {
        const e=key(f[j],f[(j+1)%3]);
        count.set(e,(count.get(e)||0)+1);
      }
    }
    for(const [edge,c] of count) {
      if(c>2) throw Error("Mehrfachkante im Schnittteil.");
      if(c!==1) continue;
      const [a,b]=edge.split(":").map(Number);
      if(!adj.has(a)) adj.set(a,[]);
      if(!adj.has(b)) adj.set(b,[]);
      adj.get(a).push(b);adj.get(b).push(a);
    }
    if(adj.size<3 || [...adj.values()].some(list=>list.length!==2))
      throw Error("Schnittteil hat keine einfache Außenkontur.");
    const start=Math.min(...adj.keys()),indices=[],visited=new Set();
    let current=start,previous=null;
    while(true) {
      if(visited.has(current)) throw Error("Ungültige Schnittteilkontur.");
      indices.push(current);visited.add(current);
      const list=adj.get(current);
      const next=list[0]===previous?list[1]:list[0];
      if(next===start) break;
      previous=current; current=next;
      if(indices.length>adj.size) throw Error("Offene Kontur.");
    }
    if(indices.length!==adj.size) throw Error("Mehrere Randkurven oder Löcher werden noch nicht unterstützt.");
    const points=indices.map(i=>patch.coords.get(i));
    if(Math.abs(area(points))<1e-8) throw Error("Degenerierte Stoffkontur.");
    return points;
  }

  function offsetPolygon(outline,mm) {
    if(!mm) return outline.map(p=>[...p]);
    const orientation=Math.sign(area(outline)),lines=[];
    for(let i=0;i<outline.length;i++) {
      const a=outline[i],b=outline[(i+1)%outline.length],c=outline[(i+2)%outline.length];
      const d=sub(b,a),other=sub(c,b);
      if(cross(d,other)*orientation < -1e-8)
        throw Error("Nahtzugabe für konkave Schnittteile noch nicht verfügbar.");
      const len=norm(d);
      if(len<=1e-9) throw Error("Nullkante in Schnittkontur.");
      const n=[orientation*d[1]/len*mm,-orientation*d[0]/len*mm];
      lines.push([[a[0]+n[0],a[1]+n[1]],[b[0]+n[0],b[1]+n[1]]]);
    }
    const out=[];
    for(let i=0;i<lines.length;i++) {
      const [p,q]=lines[(i+lines.length-1)%lines.length], [a,b]=lines[i];
      const r=sub(q,p),s=sub(b,a),den=cross(r,s);
      if(Math.abs(den)<1e-10) {
        // Collinear neighboring edges: use this edge's outward offset.
        out.push(a);
        continue;
      }
      const t=cross(sub(a,p),s)/den;
      const hit=[p[0]+t*r[0],p[1]+t*r[1]];
      if(!hit.every(Number.isFinite)) throw Error("Nahtzugabe ist numerisch instabil.");
      out.push(hit);
    }
    return out;
  }

  function generate(mesh, opts) {
    const factor={mm:1,cm:10,m:1000}[opts.unit];
    if(!factor) throw Error("Unbekannte Längeneinheit.");
    const allowance=Number(opts.allowance),density=Number(opts.density);
    if(!Number.isFinite(allowance)||allowance<0||allowance>200)
      throw Error("Nahtzugabe muss zwischen 0 und 200 mm liegen.");
    if(!Number.isFinite(density)||density<1||density>5000)
      throw Error("Flächengewicht muss zwischen 1 und 5000 g/m² liegen.");
    const vertices=mesh.vertices.map(v=>v.map(n=>n*factor));
    const patches=flatten(vertices,mesh.faces,opts.seams);
    let sewn=0,cut=0,xoffset=10,maxh=50;
    const groups=[];
    for(let i=0;i<patches.length;i++) {
      const contour=boundary(mesh.faces,patches[i]);
      const cutter=offsetPolygon(contour,allowance);
      const xs=contour.concat(cutter).map(p=>p[0]),ys=contour.concat(cutter).map(p=>p[1]);
      const minx=Math.min(...xs),maxx=Math.max(...xs),miny=Math.min(...ys),maxy=Math.max(...ys);
      const pts=a=>a.map(p=>p.map(x=>x.toFixed(5)).join(",")).join(" ");
      sewn+=Math.abs(area(contour));cut+=Math.abs(area(cutter));
      groups.push('<g transform="translate('+ (xoffset-minx).toFixed(5)+' '+(19-miny).toFixed(5)+')">'+
        '<text x="'+minx.toFixed(3)+'" y="'+(miny-6).toFixed(3)+'" fill="#c8d7e8" font-size="5">Teil '+(i+1)+'</text>'+
        '<polygon fill="none" stroke="#f5a173" stroke-width="0.65" points="'+pts(cutter)+'"/>'+
        '<polygon fill="none" stroke="#75cfd3" stroke-width="0.65" stroke-dasharray="3 2" points="'+pts(contour)+'"/></g>');
      xoffset+=maxx-minx+24; maxh=Math.max(maxh,maxy-miny+40);
    }
    const mass=cut*density/1e6,width=Math.max(50,xoffset);
    const svg='<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 '+width.toFixed(3)+' '+maxh.toFixed(3)+'" width="'+width.toFixed(3)+'mm" height="'+maxh.toFixed(3)+'mm" data-pieces="'+patches.length+'" data-sewn-area="'+sewn.toFixed(3)+'" data-cut-area="'+cut.toFixed(3)+'" data-mass="'+mass.toFixed(3)+'"><rect width="100%" height="100%" fill="#14202e"/>'+groups.join('')+'</svg>';
    return {svg,pieces:patches.length,sewnArea:sewn,cutArea:cut,mass};
  }

  const api={generate,flatten,boundary,offsetPolygon};
  if(typeof module!=="undefined" && module.exports) module.exports=api;
  root.TextilLabBrowserEngine=api;
})(typeof globalThis==="undefined" ? this : globalThis);

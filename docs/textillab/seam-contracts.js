/* TextilLab reusable sewing seam contracts: *sewing line* length only,
   NEVER cut-edge length, and no unapproved automatic stretching.
   This module does not certify fit, grading, seam allowances or child safety.
*/
(function(root){
  "use strict";
  const dist=(a,b)=>Math.hypot(a[0]-b[0],a[1]-b[1]);
  function measured(points) {
    if(!Array.isArray(points)||points.length<2)throw Error("Nahtlinie benötigt mindestens 2 Punkte.");
    let length=0;
    for(let i=1;i<points.length;i++){
      const a=points[i-1],b=points[i];
      if(!Array.isArray(a)||!Array.isArray(b)||
        ![...a,...b].every(Number.isFinite))throw Error("Ungültige Nahtkoordinaten.");
      length+=dist(a,b);
    }
    if(!Number.isFinite(length)||length<=0)throw Error("Nahtlänge muss positiv sein.");
    return length;
  }
  function check(spec) {
    const a=Number(spec.aMm),b=Number(spec.bMm);
    const tolerance=Number(spec.toleranceMm??1);
    const mode=spec.mode??"match";
    if(!Number.isFinite(a)||!Number.isFinite(b)||a<=0||b<=0||
       !Number.isFinite(tolerance)||tolerance<0||tolerance>10)
      throw Error("Nahtvergleich: positive Nahtlängen und Toleranz 0–10 mm erforderlich.");
    let expected=0,status,reason;
    if(mode==="match"){
      status=Math.abs(a-b)<=tolerance;
      reason="Nahtlinien müssen innerhalb der Maßtoleranz gleich lang sein.";
    }else if(mode==="ease"){
      const planned=Number(spec.plannedEaseMm);
      if(!Number.isFinite(planned)||planned<=0||planned>100)
        throw Error("Einhalten nur mit expliziter positiver Einhalteweite.");
      expected=planned;
      status=a>=b && Math.abs(a-b-planned)<=tolerance;
      reason="Die längere Naht A wird um den vorgegebenen Betrag auf B eingehalten.";
    }else if(mode==="stretch"){
      const maxPct=Number(spec.maxStretchPct);
      if(!Number.isFinite(maxPct)||maxPct<=0||maxPct>100||
         spec.materialStretchVerified!==true)
        throw Error("Dehnen erfordert einen geprüften Materialwert und eine Maximaldehnung.");
      const pct=(b-a)/a*100;
      status=a<b && pct<=maxPct+1e-8; // FP tolerance for exact percent boundary
      reason="Naht A wird auf B gedehnt; erforderliche Dehnung "+pct.toFixed(1)+" %.";
    }else{
      throw Error("Nicht unterstützte Nahtverarbeitung: "+mode);
    }
    const diff=a-b;
    return Object.freeze({
      id:spec.id||"seam",label:spec.label||"Nahtpaar",mode,aMm:a,bMm:b,
      deltaMm:diff,expectedDeltaMm:expected,toleranceMm:tolerance,
      ok:status,status:status?"OK":"FEHLER",reason
    });
  }
  function checkAll(specs){
    if(!Array.isArray(specs)||specs.length===0)throw Error("Keine Nähverbindungen definiert.");
    const seams=specs.map(check);
    return Object.freeze({
      seams,ok:seams.every(s=>s.ok),
      failed:seams.filter(s=>!s.ok).map(s=>s.id),
      maximumMismatchMm:Math.max(...seams.map(s=>Math.abs(s.deltaMm-s.expectedDeltaMm)))
    });
  }
  const api={measured,check,checkAll};
  if(typeof module!=="undefined"&&module.exports)module.exports=api;
  root.TextilLabSeamContracts=api;
})(typeof globalThis!=="undefined"?globalThis:this);

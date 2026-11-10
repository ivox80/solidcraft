"use strict";
const {test}=require("node:test");
const assert=require("node:assert/strict");
const fs=require("node:fs");
const path=require("node:path");
const bodice=require("./bodice-v2.js");
const rules=require("./drafting-system.js");
const tailor=require("./top-sewing.js");
const three=require("./garment-3d.js");
const defaults=()=>rules.generate();

test("sleeve cap is balanced independently front and rear, underarm sewing lines remain equal",()=>{
 for(const ease of [0,12,22,35,45]){
  const d=rules.generate({}, {targetEaseMm:ease});
  assert.ok(d.capBalance.reachable,"Ease "+ease);
  const measured=d.measures;
  const sleeve=d.construction.pieces[2].outline;
  const front=tailor.resample(sleeve.slice(0,25),28);
  const back=tailor.resample(sleeve.slice(24,49),28);
  const arc=points=>points.slice(1).reduce((sum,p,i)=>
    sum+Math.hypot(p[0]-points[i][0],p[1]-points[i][1]),0);
  assert.ok(Math.abs(arc(front)-measured.frontArmholeMm-ease*.4)<.6);
  assert.ok(Math.abs(arc(back)-measured.backArmholeMm-ease*.6)<.6);
  const underA=arc(tailor.resample(sleeve.slice(48,69),36));
  const underB=arc(tailor.resample([...sleeve.slice(69),sleeve[0]],36));
  assert.ok(Math.abs(underA-underB)<.1);
  assert.equal(d.construction.pieces[2].points.hemY,d.construction.measurements.sleeve);
 }
});
test("each real top seam has TWO named pattern edges sharing actual 3D vertices",()=>{
 const output=tailor.construct(defaults());
 const {stitched}=output.topology;
 assert.equal(output.metrics.panelCount,6);
 assert.equal(output.metrics.sewnEdgePairs,10);
 assert.equal(output.metrics.foldPairs,2);
 assert.equal(output.metrics.missingSeamPartners,0);
 assert.equal(output.metrics.welded,true);
 assert.equal(output.metrics.centerSeamGapMm,0);
 assert.equal(output.metrics.validForProduction,false);
 assert.equal(output.metrics.seamFitVerified,false); // planned ease is not a verified material
 for(const seam of output.contracts){
  const a=stitched.offsets.get(seam.a),b=stitched.offsets.get(seam.b);
  assert.ok(Number.isInteger(a)&&Number.isInteger(b),seam.id);
  for(const [v,w]of seam.indexPairs){
   assert.equal(stitched.mapping[a+v],stitched.mapping[b+w],seam.id);
  }
  assert.ok(seam.lengthAMm>0&&seam.lengthBMm>0);
 }
 for(const name of ["Schulternaht R","Seitennaht L","Ärmel vorn R","Ärmel hinten L",
    "Ärmel-Unterarm R","Stoffbruch vorne"]){
  assert.ok(output.contracts.some(s=>s.id===name),name);
 }
 for(const v of output.meshes.at(-1).vertices)assert.ok(v.every(Number.isFinite));
 assert.ok(output.meshes.at(-1).faces.length>1700);
});
test("sleeve easing is distributed 40/60 and flagged for physical processing",()=>{
 const output=tailor.construct(defaults());
 const cap=output.contracts.filter(c=>c.id.startsWith("Ärmel vorn")||c.id.startsWith("Ärmel hinten"));
 assert.equal(cap.length,4);
 for(const seam of cap){
  assert.equal(seam.mode,"ease");
  assert.equal(seam.status,"EINHALTEN_GEPLANT");
  assert.ok(Math.abs(seam.deltaMm-seam.plannedEaseMm)<.6);
 }
 assert.equal(output.metrics.pendingProcessing,4);
 assert.ok(output.metrics.unclosedDarts.length>0);
 assert.deepEqual(output.metrics.openEdges,["Halsausschnitt","Saum","Ärmelbündchen"]);
});
test("sleeveless choice produces an actually open armhole and NO fictional sleeve",()=>{
 const output=tailor.construct(rules.generate({}, {style:"sleeveless"}));
 assert.equal(output.metrics.panelCount,4);
 assert.equal(output.metrics.sewnEdgePairs,4);
 assert.equal(output.metrics.foldPairs,2);
 assert.equal(output.contracts.filter(s=>s.id.startsWith("Ärmel")).length,0);
 assert.ok(output.metrics.openEdges.some(s=>s.includes("Armlöcher")));
});
test("changing cloth properties produces different gravity-based mesh displacements",()=>{
 const input=defaults();
 const soft=tailor.construct(input,{}, {preset:"jersey",iterations:16});
 const hard=tailor.construct(input,{}, {preset:"denim",iterations:16});
 const noGravity=tailor.construct(input,{}, {preset:"cotton",iterations:16,drape:0});
 const gravity=tailor.construct(input,{}, {preset:"cotton",iterations:16,drape:1.8});
 assert.notDeepEqual(soft.meshes.at(-1).vertices,hard.meshes.at(-1).vertices);
 assert.notDeepEqual(noGravity.meshes.at(-1).vertices,gravity.meshes.at(-1).vertices);
 assert.ok(soft.metrics.rmsEdgeStrainPct>0);
 assert.ok(soft.metrics.approximation);
 assert.equal(soft.metrics.validForProduction,false);
});
test("independent mannequin measurements and actual pattern inputs change the mesh",()=>{
 const original=tailor.construct(defaults());
 const adjusted=tailor.construct(defaults(),{
  chestMm:1100,shoulderMm:460,waistMm:920,hipMm:1080,heightMm:1790
 });
 assert.equal(adjusted.body.height,1790);
 assert.equal(adjusted.body.shoulder,460);
 assert.notDeepEqual(original.meshes[0].vertices,adjusted.meshes[0].vertices);
 const wider=tailor.construct(rules.generate({chest:110,waist:90,hip:108}));
 assert.notDeepEqual(original.meshes.at(-1).vertices,wider.meshes.at(-1).vertices);
 assert.ok(original.meshes.some(x=>x.name.includes("Kopf")));
 assert.ok(original.meshes.some(x=>x.fullBodyOnly&&x.name.includes("Bein")));
});
test("missing pattern pieces are rejected rather than being replaced by tubes",()=>{
 assert.throws(()=>tailor.construct({}),/2D-Grundschnitt/);
 const bad=structuredClone(defaults());
 bad.construction.pieces[0].outline.pop();
 assert.throws(()=>tailor.construct(bad),/Konstruktionspunkte/);
 assert.throws(()=>tailor.construct(defaults(),{heightMm:400}),/Körpergröße/);
 assert.throws(()=>tailor.construct(defaults(),{}, {preset:"fiction"}),/Stofftyp/);
});
test("3D viewer loads real panel mesh, preserves user controls and OBJ output",()=>{
 const context=new Proxy({createLinearGradient:()=>({addColorStop(){}})},
  {get(o,k){return k in o?o[k]:()=>{}}});
 const canvas={getContext:()=>context,
  getBoundingClientRect:()=>({width:600,height:540}),
  addEventListener(){},setPointerCapture(){}};
 const renderer=three.createRenderer(canvas);
 assert.throws(()=>renderer.setModel("top",{}),/2D-Schnittteile/);
 const report=renderer.setModel("top",{topDraft:defaults(),fabric:{preset:"cotton"}});
 assert.equal(report.sewnEdgePairs,10);
 assert.equal(renderer.state.report.centerSeamGapMm,0);
 renderer.setOptions({body:true,fullBody:true,wire:true});
 assert.equal(renderer.state.fullBody,true);
 assert.equal(renderer.state.wire,true);
 assert.ok(renderer.obj().includes("o Oberteil"));
 assert.ok(renderer.obj().includes("f "));
});
test("the actual UI exposes paired-seam inspector and editable cloth controls",()=>{
 const index=fs.readFileSync(path.join(__dirname,"index.html"),"utf8");
 const app=fs.readFileSync(path.join(__dirname,"app.js"),"utf8");
 assert.ok(index.includes("./top-sewing.js"));
 assert.ok(index.indexOf("./top-sewing.js")<index.indexOf("./garment-3d.js"));
 for(const id of ["topSewingReport","topAvatarLink","fitProfile","fitChest","fabricPreset","fabricDrape"]){
  assert.ok(index.includes('id="'+id+'"'),id);
 }
 assert.ok(app.includes('garmentRenderer.setModel("top"'));
 assert.ok(app.includes("showTopSeamReport"));
 const ids=[...index.matchAll(/\bid="([^"]+)"/g)].map(x=>x[1]);
 assert.equal(ids.length,new Set(ids).size,"duplicate HTML control ID");
});

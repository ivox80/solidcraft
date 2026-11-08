"use strict";
const {test}=require("node:test");
const assert=require("node:assert/strict");
const fs=require("node:fs");
const path=require("node:path");
const hood=require("./hood-system.js");
const contracts=require("./seam-contracts.js");
const viewer=require("./garment-3d.js");

test("each body profile is separately parameterized and marked as a draft",()=>{
 const a=hood.draft(),b=hood.draft({profile:"men"}),c=hood.draft({profile:"child"});
 assert.equal(a.values.headCirc,570);
 assert.equal(b.values.headCirc,590);
 assert.equal(c.values.headCirc,520);
 assert.equal(a.productionValidated,false);
 for(const result of [a,b,c]){
   assert.ok(result.sewingLinesVerified);
   assert.ok(result.checks.every(s=>s.status==="ok"));
   assert.ok(result.parts.length===1 && result.pieces===2);
   assert.ok(result.parts[0].outline.length>100);
   assert.ok(viewer.modelFor("hood",result.model).length>0);
 }
});
test("join must match by actual sewing-line arc length",()=>{
 const d=hood.draft();
 assert.ok(Math.abs(d.checks[0].actualMm-d.values.backNeck)<.25);
 assert.ok(Math.abs(d.checks[1].actualMm-d.values.frontNeck)<.25);
 assert.ok(Math.abs(d.checks[2].deltaMm)<.25);
 const changed=hood.draft({backNeck:11.5,frontNeck:14});
 assert.ok(changed.checks.every(s=>s.status==="ok"));
 assert.notDeepEqual(changed.parts[0].outline,d.parts[0].outline);
});
test("deliberate easing changes hood neckline length by exactly the specified amount",()=>{
 const d=hood.draft({joinMode:"ease",allowedEase:0.6});
 assert.ok(d.sewingLinesVerified);
 assert.ok(Math.abs(d.checks[0].deltaMm-3)<.25);
 assert.ok(Math.abs(d.checks[1].deltaMm-3)<.25);
 assert.ok(d.checks.slice(0,2).every(s=>s.mode==="ease"));
 assert.throws(()=>hood.draft({joinMode:"ease",allowedEase:0}),/Einhalten/);
});
test("stretch is blocked without actual verified material characteristics",()=>{
 assert.throws(()=>hood.draft({joinMode:"stretch",stretchPct:4}),/geprüfte Materialdehnung/);
 const d=hood.draft({joinMode:"stretch",stretchPct:4,materialStretchVerified:true});
 assert.ok(d.checks.every(x=>x.status==="ok"));
 for(const s of d.checks.slice(0,2)){
   assert.equal(s.mode,"stretch");
   assert.ok(Math.abs((s.targetMm/s.actualMm-1)*100-4)<.001);
 }
 assert.throws(()=>hood.draft({joinMode:"stretch",stretchPct:15,materialStretchVerified:true}),/geprüfte Materialdehnung/);
});
test("child profile rejects unsafe age or neck drawstrings",()=>{
 assert.throws(()=>hood.draft({profile:"child",ageYears:0}),/Kinderprofil: Alter/);
 assert.throws(()=>hood.draft({profile:"child",drawstrings:true}),/Halskordeln/);
 assert.throws(()=>hood.draft({profile:"child",headCirc:65}),/Kinderprofil/);
 assert.equal(hood.draft({profile:"child",ageYears:12}).values.ageYears,12);
});
test("generic seam contract prevents false OK on opposing seam mismatches",()=>{
 assert.equal(contracts.check({aMm:109,bMm:100,mode:"match"}).ok,false);
 assert.equal(contracts.check({aMm:109,bMm:100,mode:"ease",plannedEaseMm:9}).ok,true);
 assert.equal(contracts.check({aMm:91,bMm:100,mode:"ease",plannedEaseMm:9}).ok,false);
 assert.throws(()=>contracts.check({aMm:91,bMm:100,mode:"stretch",maxStretchPct:10}),/geprüften Materialwert/);
 assert.equal(contracts.check({aMm:100,bMm:120,mode:"stretch",maxStretchPct:5,materialStretchVerified:true}).ok,false);
 const result=contracts.checkAll([{id:"a",aMm:100,bMm:100},{id:"b",aMm:140,bMm:100}]);
 assert.equal(result.ok,false);
 assert.deepEqual(result.failed,["b"]);
});
test("3D and 2D hood UI remain present and safe input controls are wired",()=>{
 const html=fs.readFileSync(path.join(__dirname,"index.html"),"utf8");
 const app=fs.readFileSync(path.join(__dirname,"app.js"),"utf8");
 for(const id of ["hoodAgeYears","hoodStretchPct","hoodVerifiedMaterial",
  "hoodChildAgeRow","hoodRuleReport","garmentCanvas","hoodProfile","hoodVariant"]){
  assert.ok(html.includes('id="'+id+'"'),id);
 }
 assert.ok(html.includes("./garment-3d.js"));
 assert.ok(html.indexOf("seam-contracts.js")<html.indexOf("hood-system.js"));
 for(const id of ["hoodStretchPct","hoodAgeYears","hoodVerifiedMaterial"])
  assert.ok(app.includes(id),id);
});

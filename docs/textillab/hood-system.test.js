"use strict";
const {test}=require("node:test");
const assert=require("node:assert/strict");
const hood=require("./hood-system.js");
const seam=require("./seam-contracts.js");

test("profiled hood pieces have exact measured rear/front neck joining lengths",()=>{
 for(const group of ["women","men","child"]){
   const result=hood.render({}, {profile:group});
   assert.equal(result.pieces,2,group);
   assert.equal(result.parts.length,1,group);
   assert.equal(result.parts[0].count,2,group);
   assert.ok(result.ready && result.report.ok,group);
   assert.ok(result.productionValidated===false);
   assert.equal(result.report.seams.length,3);
   assert.ok(result.report.seams.every(x=>x.ok),group);
   assert.ok(result.report.seams.every(x=>Math.abs(x.deltaMm)<.05),group);
   assert.ok(result.svg.includes('data-production-approved="false"'),group);
   assert.ok(result.cutArea>result.sewnArea && result.mass>0,group);
   assert.ok(!result.svg.includes("NaN")&&!result.svg.includes("Infinity"));
 }
});
test("close vs relaxed hood and body measures change real cubic outlines",()=>{
 const fitted=hood.prepare({},{profile:"women",style:"close"});
 const relaxed=hood.prepare({},{profile:"women",style:"relaxed"});
 const wider=hood.prepare({head:60,hoodDepth:30,backNeck:11,frontNeck:13});
 assert.ok(relaxed.profile.mm.hoodHeight>fitted.profile.mm.hoodHeight);
 assert.ok(relaxed.profile.mm.hoodDepth>fitted.profile.mm.hoodDepth);
 assert.notDeepEqual(relaxed.parts[0].outline,fitted.parts[0].outline);
 assert.notDeepEqual(wider.parts[0].outline,fitted.parts[0].outline);
 assert.ok(wider.values.rearNeckMm>fitted.values.rearNeckMm);
 assert.ok(wider.values.frontNeckMm>fitted.values.frontNeckMm);
});
test("ease is only allowed with a deliberate measured extra length",()=>{
 const result=hood.prepare({},{neckMode:"ease",plannedEaseCm:.6});
 assert.equal(result.report.ok,true);
 assert.ok(Math.abs(result.report.seams[1].deltaMm-3)<.1);
 assert.ok(Math.abs(result.report.seams[2].deltaMm-3)<.1);
 assert.equal(result.report.seams[1].mode,"ease");
 assert.throws(()=>hood.render({},{neckMode:"ease",plannedEaseCm:0}),/Einhalteweite/);
});
test("stretch requires explicit verified material capability; correct percentage",()=>{
 const result=hood.prepare({},{neckMode:"stretch",stretchPct:4,materialStretchVerified:true});
 assert.ok(result.report.ok,JSON.stringify(result.report.seams));
 for(const s of result.report.seams.slice(1)) {
   assert.equal(s.mode,"stretch");
   assert.ok((s.bMm/s.aMm-1)*100 <4.001);
 }
 assert.throws(()=>hood.render({},{neckMode:"stretch",stretchPct:4}),/dokumentierter Materialdehnung/);
 const tooMuch=seam.check({aMm:100,bMm:118,mode:"stretch",maxStretchPct:4,materialStretchVerified:true});
 assert.equal(tooMuch.ok,false);
});
test("generic seam contracts fail closed on nonmatching or unauthorised processing",()=>{
 const unmatched=seam.check({id:"bad",aMm:120,bMm:111,mode:"match",toleranceMm:1});
 assert.equal(unmatched.ok,false);
 const easing=seam.check({aMm:120,bMm:111,mode:"ease",plannedEaseMm:9,toleranceMm:.2});
 assert.equal(easing.ok,true);
 const wrong=seam.check({aMm:111,bMm:120,mode:"ease",plannedEaseMm:9});
 assert.equal(wrong.ok,false);
 const all=seam.checkAll([
   {aMm:100,bMm:100},{aMm:100,bMm:92}
 ]);
 assert.equal(all.ok,false);
 assert.deepEqual(all.failed,["seam"]);
 assert.throws(()=>seam.check({aMm:100,bMm:120,mode:"stretch",maxStretchPct:25}),/geprüften Materialwert/);
});
test("children require age, forbid drawstrings, and do not inherit adult profile",()=>{
 assert.notDeepEqual(hood.profiles.child,hood.profiles.women);
 assert.throws(()=>hood.render({ageYears:0},{profile:"child"}),/Kinderprofil/);
 assert.throws(()=>hood.render({},{profile:"child",drawstrings:true}),/Kordeln/);
 assert.throws(()=>hood.render({head:68},{profile:"child"}),/Kopfumfang/);
 assert.throws(()=>hood.render({},{profile:"unknown"}),/Profil/);
 assert.equal(hood.render({ageYears:12},{profile:"child"}).profile.ageYears,12);
});
test("invalid target neckline and seam parameters must be rejected",()=>{
 assert.throws(()=>hood.render({backNeck:"abc"}),/Ungültiges Kapuzenmaß/);
 assert.throws(()=>hood.render({frontNeck:32}),/Halb-Halsloch/);
 assert.throws(()=>hood.render({hoodDepth:18}),/Kapuzenhöhe oder -tiefe/);
 assert.throws(()=>hood.render({}, {allowanceMm:-1}),/Nahtzugabe/);
 assert.throws(()=>hood.render({}, {density:0}),/Stoffgewicht/);
});

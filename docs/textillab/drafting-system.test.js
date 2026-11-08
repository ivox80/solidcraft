"use strict";
const {test}=require("node:test");
const assert=require("node:assert/strict");
const drafting=require("./drafting-system.js");
const base=require("./bodice-v2.js");
const L={...drafting.layers};

test("v0.8: named points, construction lines, dimensional labels and alteration bars are in technical drawing",()=>{
  const out=drafting.generate();
  assert.match(out.svg,/id="layer-construction-lines"/);
  assert.match(out.svg,/id="layer-construction-points"/);
  assert.match(out.svg,/id="layer-dimensions"/);
  assert.match(out.svg,/id="layer-alteration-lines"/);
  assert.match(out.svg,/data-measure="frontLength"/);
  assert.match(out.svg,/data-measure="upperArm"/);
  for(const label of ["MV","MH","SP","AL","BP","FDL","K-torso","K-hip","K-hem"])assert.ok(out.svg.includes(label),label);
  assert.equal(out.namedPoints.VT.length,8);
  assert.equal(out.rules.length,8);
  assert.ok(!out.svg.includes("NaN")&&!out.svg.includes("Infinity"));
});
test("v0.8: construction layers can be switched off individually",()=>{
  const no=drafting.generate({},{layers:{guides:false,dimensions:false,points:false,alterations:false,notes:false}});
  assert.ok(!no.svg.includes('id="layer-construction-lines"'));
  assert.ok(!no.svg.includes('id="layer-dimensions"'));
  assert.ok(!no.svg.includes('id="layer-alteration-lines"'));
  assert.ok(!no.svg.includes('id="layer-rule-notes"'));
  assert.match(no.svg,/data-pieces="4"/);
  assert.deepEqual(drafting.layers,L);
});
test("v0.9: armhole vs cap lengths are measured and matched numerically",()=>{
  const out=drafting.generate();
  assert.ok(out.capBalance.reachable);
  assert.ok(Math.abs(out.capBalance.actualEaseMm-22)<0.1);
  assert.ok(out.measures.sideDifferenceMm<1);
  assert.ok(Math.abs(out.measures.totalArmholeMm-
    out.measures.frontArmholeMm-out.measures.backArmholeMm)<1e-7);
  const more=drafting.generate({},{targetEaseMm:34});
  assert.ok(Math.abs(more.measures.sleeveEaseMm-34)<.1);
  assert.ok(more.capBalance.capHeightMm>out.capBalance.capHeightMm);
  assert.ok(out.rules.find(r=>r.id==="R07").matched);
});
test("v0.9: rules are named and explain their source dimensions",()=>{
  const a=drafting.generate();
  const b=drafting.generate({chest:115});
  assert.equal(a.rules.find(r=>r.id==="R01").formula,"(Br + Mehrweite) / 4");
  assert.ok(b.rules.find(r=>r.id==="R01").value > a.rules.find(r=>r.id==="R01").value);
});
test("v1.0: four alteration zones regenerate corresponding body geometry",()=>{
  const input={};
  const baseOut=drafting.generate(input);
  const torso=drafting.generate(input,{alterations:{torso:3}});
  const hip=drafting.generate(input,{alterations:{hip:2}});
  const hem=drafting.generate(input,{alterations:{hem:5}});
  const arm=drafting.generate(input,{alterations:{sleeve:4}});
  assert.equal(torso.profile.measurements.frontLength,49);
  assert.equal(torso.profile.measurements.backWaist,46);
  assert.equal(hip.profile.measurements.waistToHip,22);
  assert.equal(hip.profile.measurements.hemBelowWaist,31);
  assert.equal(hem.profile.measurements.hemBelowWaist,34);
  assert.equal(arm.profile.measurements.sleeve,66);
  for(const altered of [torso,hip,hem,arm])
    assert.notEqual(altered.svg,baseOut.svg);
  assert.ok(arm.cutArea>baseOut.cutArea);
});
test("v1.0: fit profiles and style variations affect geometry and cut-piece count",()=>{
  const fitted=drafting.generate({},{fit:"fitted"});
  const coat=drafting.generate({},{fit:"loose",style:"coat"});
  const sleeveless=drafting.generate({},{style:"sleeveless"});
  assert.equal(fitted.profile.measurements.ease,4);
  assert.equal(coat.profile.measurements.ease,18);
  assert.equal(coat.profile.measurements.hemBelowWaist,74);
  assert.ok(coat.cutArea>fitted.cutArea);
  assert.equal(sleeveless.pieces,2);
  assert.equal(sleeveless.parts.length,2);
  assert.ok(sleeveless.cutArea<fitted.cutArea);
  assert.match(sleeveless.svg,/data-sleeves="0"/);
  const custom=drafting.generate({ease:14},{fit:"custom"});
  assert.equal(custom.profile.measurements.ease,14);
});
test("v1.0: invalid settings fail without creating unsafe output",()=>{
  assert.throws(()=>drafting.generate({},{alterations:{torso:44}}),/Längenänderung/);
  assert.throws(()=>drafting.generate({},{targetEaseMm:55}),/Einhalteweite/);
  assert.throws(()=>drafting.generate({},{fit:"missing"}),/Passform/);
  assert.throws(()=>drafting.generate({},{style:"missing"}),/Modellvariante/);
  assert.throws(()=>drafting.generate({frontLength:35},{alterations:{torso:-10}}));
});
test("legacy pattern interface remains usable",()=>{
  const old=base.render();
  assert.equal(old.parts.length,3);
  assert.equal(old.pieces,4);
  assert.equal(base.render({},10,210,{sleeves:false}).pieces,2);
});

test("SVG dimension numbers show the measured sketch span, not full-body input values",()=>{
  const bodice=base.draft();
  const vt=bodice.pieces[0],sleeve=bodice.pieces[2];
  const actualShoulder=Math.hypot(
    vt.points.shoulderStart[0]-vt.points.shoulderEnd[0],
    vt.points.shoulderStart[1]-vt.points.shoulderEnd[1]
  )/10;
  const sleeveSpan=(Math.max(...sleeve.outline.map(p=>p[0])))/10;
  const out=drafting.generate();
  assert.ok(out.svg.includes("SL "+actualShoulder.toFixed(1)+" cm"));
  assert.ok(out.svg.includes("ÄB "+sleeveSpan.toFixed(1)+" cm"));
  assert.ok(!out.svg.includes("Sch 44.0 cm"));
});

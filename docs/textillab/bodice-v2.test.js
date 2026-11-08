"use strict";
const { test } = require("node:test");
const assert = require("node:assert/strict");
const bodice = require("./bodice-v2.js");
const patterns = require("./pattern-library.js");

test("VT/RT are separately drafted with smooth curves and darts", () => {
  const draft = bodice.draft();
  assert.equal(draft.pieces.length, 3);
  const [vt, rt, sleeve] = draft.pieces;
  assert.ok(vt.name.includes("VT"));
  assert.ok(rt.name.includes("RT"));
  assert.ok(vt.svgPath.split(" C ").length >= 6);
  assert.ok(rt.svgPath.split(" C ").length >= 6);
  assert.ok(vt.outline.length > 100);
  assert.ok(rt.outline.length > 100);
  assert.equal(vt.darts.waist.length, 4);
  assert.equal(rt.darts.shoulder.legs.length, 2);
  assert.notDeepEqual(vt.outline, rt.outline);
  assert.equal(sleeve.count, 2);
  assert.ok(sleeve.name.includes("Ärmel"));
  assert.ok(sleeve.svgPath.split(" C ").length >= 5);
  assert.ok(sleeve.outline.length > 70);
});
test("SVG has reference construction lines, FDL and labels", () => {
  const a = bodice.render();
  assert.match(a.svg, /data-bodice-version="2"/);
  for (const label of ["VT", "RT", "Brust", "Taille", "Hüfte", "Saum", "FDL"]) {
    assert.ok(a.svg.includes(label), label);
  }
  assert.equal(a.pieces, 4);
  assert.equal(a.parts.length, 3);
  assert.ok(a.cutArea > a.sewnArea);
  assert.match(a.svg, /data-sleeves="2"/);
  assert.match(a.svg, /<rect[^>]+fill="#14202e"/);
  assert.ok(!a.svg.includes("#d9c4a5"));
  assert.ok(a.svg.includes("ÄR"));
  assert.ok(a.mass > 0);
  assert.ok(!a.svg.includes("NaN") && !a.svg.includes("Infinity"));
});
test("each body size parameter affects recomputed drawing", () => {
  const reference = bodice.render();
  const changes = {
    chest:112, waist:92, hip:114, shoulder:47,
    frontLength:49, backWaist:46, bustHeight:29, bustSpan:20,
    neck:41, armholeDepth:25, waistToHip:22, hemBelowWaist:33,
    shoulderSlope:4, sleeve:69, upperArm:38, wrist:21,
  };
  for (const [key, value] of Object.entries(changes)) {
    const changed = bodice.render({[key]: value});
    assert.notEqual(changed.svg, reference.svg, key + " must regenerate geometry");
  }
});
test("larger allowance and higher density increase mass", () => {
  const base = bodice.render({}, 0, 210);
  const withSeam = bodice.render({}, 10, 210);
  const doubled = bodice.render({}, 10, 420);
  assert.ok(withSeam.cutArea > base.cutArea);
  assert.ok(Math.abs(doubled.mass/withSeam.mass-2) < 1e-9);
});
test("invalid anatomically inconsistent measures are rejected", () => {
  assert.throws(()=>bodice.render({frontLength:40,bustHeight:39}),/Brusthöhe/);
  assert.throws(()=>bodice.render({waistToHip:29,hemBelowWaist:25}),/Hüftlinie/);
  assert.throws(()=>bodice.render({shoulder:125}),/Schulter/);
  assert.throws(()=>bodice.render({},-1),/Nahtzugabe/);
});
test("main clothing catalog routes top to v2 while retaining other categories", () => {
  const front = patterns.draft("top");
  assert.equal(front.pieces.length, 3);
  assert.ok(front.pieces[0].name.includes("VT"));
  assert.equal(patterns.render("top").pieces, 4);
  assert.ok(patterns.render("jacket").pieces > 2);
  assert.ok(patterns.render("trousers").pieces > 2);
});

test("sleeve panel responds to sleeve length, upper-arm and wrist circumference", () => {
  const a = bodice.draft();
  const b = bodice.draft({sleeve:70, upperArm:39, wrist:21});
  assert.notDeepEqual(a.pieces[2].outline, b.pieces[2].outline);
  assert.ok(bodice.render({sleeve:70}).cutArea > bodice.render({sleeve:62}).cutArea);
});

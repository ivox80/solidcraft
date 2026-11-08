"use strict";
const { test } = require("node:test");
const assert = require("node:assert/strict");
const bodice = require("./bodice-v2.js");
const patterns = require("./pattern-library.js");

test("VT/RT are separately drafted with smooth curves and darts", () => {
  const draft = bodice.draft();
  assert.equal(draft.pieces.length, 2);
  const [vt, rt] = draft.pieces;
  assert.ok(vt.name.includes("VT"));
  assert.ok(rt.name.includes("RT"));
  assert.ok(vt.svgPath.split(" C ").length >= 6);
  assert.ok(rt.svgPath.split(" C ").length >= 6);
  assert.ok(vt.outline.length > 100);
  assert.ok(rt.outline.length > 100);
  assert.equal(vt.darts.waist.length, 4);
  assert.equal(rt.darts.shoulder.legs.length, 2);
  assert.notDeepEqual(vt.outline, rt.outline);
});
test("SVG has reference construction lines, FDL and labels", () => {
  const a = bodice.render();
  assert.match(a.svg, /data-bodice-version="2"/);
  for (const label of ["VT", "RT", "Brust", "Taille", "Hüfte", "Saum", "FDL"]) {
    assert.ok(a.svg.includes(label), label);
  }
  assert.equal(a.pieces, 2);
  assert.equal(a.parts.length, 2);
  assert.ok(a.cutArea > a.sewnArea);
  assert.ok(a.mass > 0);
  assert.ok(!a.svg.includes("NaN") && !a.svg.includes("Infinity"));
});
test("each body size parameter affects recomputed drawing", () => {
  const reference = bodice.render();
  const changes = {
    chest:112, waist:92, hip:114, shoulder:47,
    frontLength:49, backWaist:46, bustHeight:29, bustSpan:20,
    neck:41, armholeDepth:25, waistToHip:22, hemBelowWaist:33,
    shoulderSlope:4,
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
  assert.equal(front.pieces.length, 2);
  assert.ok(front.pieces[0].name.includes("VT"));
  assert.equal(patterns.render("top").pieces, 2);
  assert.ok(patterns.render("jacket").pieces > 2);
  assert.ok(patterns.render("trousers").pieces > 2);
});

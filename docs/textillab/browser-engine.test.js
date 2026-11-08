const {test} = require("node:test");
const assert = require("node:assert/strict");
const engine = require("./browser-engine.js");
const quad = {
  vertices:[[0,0,0],[100,0,0],[100,60,0],[0,60,0]],
  faces:[[0,1,2],[0,2,3]]
};
const options = {seams:new Set(),allowance:10,density:200,unit:"mm"};
test("a flat rectangle unfolds with exact area, offset and mass", () => {
  const result = engine.generate(quad,options);
  assert.equal(result.pieces,1);
  assert.ok(Math.abs(result.sewnArea-6000)<1e-5);
  assert.ok(Math.abs(result.cutArea-9600)<1e-5);
  assert.ok(Math.abs(result.mass-1.92)<1e-7);
  assert.match(result.svg,/<svg/);
});
test("cutting diagonal makes two independent panels", () => {
  const out = engine.generate(quad,{...options,seams:new Set(["1:3"])});
  assert.equal(out.pieces,2);
  assert.match(out.svg,/Teil 2/);
  assert.ok(out.cutArea>9600);
});
test("3D hinge unfolds without stretching edge lengths", () => {
  const bent = structuredClone(quad);
  bent.vertices[3][2] = 25;
  assert.equal(engine.generate(bent, options).pieces,1);
});
test("unit conversion cm to mm scales area 100x", () => {
  const a = engine.generate(quad,{...options,allowance:0,unit:"mm"});
  const b = engine.generate(quad,{...options,allowance:0,unit:"cm"});
  assert.ok(Math.abs(b.sewnArea/a.sewnArea-100)<1e-8);
});
test("reject invalid inputs", () => {
  assert.throws(()=>engine.generate(quad,{...options,density:-1}));
  assert.throws(()=>engine.generate(quad,{...options,allowance:-5}));
  assert.throws(()=>engine.generate(quad,{...options,seams:new Set(["2:4"])}));
  const bad = structuredClone(quad);
  bad.faces.push([0,2,1]);
  assert.throws(()=>engine.generate(bad,options));
});

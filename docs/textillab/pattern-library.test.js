"use strict";
const {test}=require("node:test");
const assert=require("node:assert/strict");
const lib=require("./pattern-library.js");

test("every garment category produces labeled panels, meaningful SVG and plausible area",()=>{
  assert.equal(lib.types.length,8);
  for(const [type] of lib.types) {
    const a=lib.render(type,{},10,210);
    assert.ok(a.parts.length>=(type==="hood"?1:2),type);
    assert.ok(a.pieces>=2,type);
    assert.match(a.svg,/data-pieces="\d+"/);
    assert.match(a.svg,/KONZEPT-GRUNDSCHNITT/);
    assert.ok(a.sewnArea>1000,type);
    assert.ok(a.cutArea>=a.sewnArea,type);
    assert.ok(a.mass>0,type);
    assert.ok(!a.svg.includes("NaN")&&!a.svg.includes("Infinity"),type);
  }
});
test("templates react to relevant measurements",()=>{
  const combinations=[
    ["top",{chest:100},{chest:115}],
    ["jacket",{chest:100},{chest:115}],
    ["hood",{hoodDepth:27},{hoodDepth:31}],
    ["trousers",{hip:102},{hip:120}],
    ["glove",{hand:22,handLength:19},{hand:25,handLength:22}],
    ["beanie",{head:57},{head:62}],
    ["hat",{head:57},{head:62}],
    ["skirt",{hip:102},{hip:120}],
  ];
  for(const [type,small,large] of combinations){
    const a=lib.render(type,small,10,210),b=lib.render(type,large,10,210);
    assert.ok(b.cutArea>a.cutArea,type+" must scale with body measurements");
  }
});
test("larger allowance and cloth density change material estimate",()=>{
  const a=lib.render("top",{},0,210);
  const b=lib.render("top",{},10,210);
  const c=lib.render("top",{},10,420);
  assert.ok(b.cutArea>a.cutArea);
  assert.ok(Math.abs(c.mass/b.mass-2)<1e-9);
});
test("body sizes and type are validated",()=>{
  assert.throws(()=>lib.render("top",{chest:-5}),/Körpermaß/);
  assert.throws(()=>lib.render("top",{shoulder:100}),/Körpermaße/);
  assert.throws(()=>lib.render("unknown",{}),/Unbekannter/);
  assert.throws(()=>lib.render("hat",{},-4),/Nahtzugabe/);
});
test("garment parts mark fold, count and separate lining opportunities",()=>{
  const top=lib.draft("top");
  assert.ok(top.pieces.some(p=>p.fold));
  const jacket=lib.draft("jacket");
  assert.ok(jacket.pieces.some(p=>p.name==="Kragen"));
  const glove=lib.draft("glove");
  assert.ok(glove.pieces.some(p=>p.name==="Daumen"));
  const hat=lib.draft("hat");
  assert.ok(hat.pieces.some(p=>p.holes.length===1));
});

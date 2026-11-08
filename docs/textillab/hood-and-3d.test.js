"use strict";
const {test}=require("node:test");
const assert=require("node:assert/strict");
const fs=require("node:fs");
const path=require("node:path");
const hood=require("./hood-system.js");
const gallery=require("./garment-3d.js");
const garments=require("./pattern-library.js");

test("published Müller & Sohn measurements are attributed, separate from invented curves",()=>{
 assert.ok(hood.source.startsWith("https://www.muellerundsohn.com/"));
 const vars=Object.keys(hood.variants);
 assert.deepEqual(vars,["close","relaxed","dartless"]);
 const rules=hood.sourceRelations;
 assert.ok(rules.length>=10);
 for(const v of ["30 bis 34","26 bis 28","1,5","0,5","3 bis 4","4 bis 5"]){
  assert.ok(rules.some(x=>x.formula.includes(v)),v);
 }
 assert.ok(rules.some(x=>x.status==="textillab-independent"));
});
test("2D hood has one mirrored pattern piece, separate measured neckline segments",()=>{
 const d=hood.render();
 assert.equal(d.type,"hood");
 assert.equal(d.pieces,2);
 assert.equal(d.parts.length,1);
 assert.equal(d.parts[0].count,2);
 assert.ok(d.parts[0].outline.length>160);
 assert.ok(d.checks.every(c=>c.status==="ok"));
 assert.ok(Math.abs(d.checks[0].actualMm-100)<.001);
 assert.ok(Math.abs(d.checks[1].actualMm-120)<.001);
 assert.match(d.svg,/data-pieces="2"/);
 assert.match(d.svg,/data-piece-id="HOOD"/);
 assert.match(d.svg,/K1/);
 assert.match(d.svg,/FDL/);
 assert.match(d.svg,/KONZEPT-GRUNDSCHNITT/);
 assert.ok(!d.svg.includes("NaN")&&!d.svg.includes("Infinity"));
});
test("changing dimensions regenerates hood geometry and 3D model specification",()=>{
 const base=hood.render();
 for(const [key,value]of Object.entries({backNeck:11,frontNeck:13,
  hoodHeight:34,hoodDepth:28,faceOverlap:2,neckWidth:9})){
  const a=hood.render({[key]:value});
  assert.notEqual(a.svg,base.svg,key);
 }
 const tall=hood.render({hoodHeight:34});
 assert.notEqual(tall.model.heightMm,base.model.heightMm);
 const changedNeck=hood.render({frontNeck:15});
 assert.notDeepEqual(gallery.modelFor("hood",changedNeck.model)[1].vertices,
  gallery.modelFor("hood",base.model)[1].vertices);
 const wide=hood.render({headCirc:61});
 assert.notEqual(wide.model.headCircMm,base.model.headCircMm);
 assert.notDeepEqual(gallery.modelFor("hood",wide.model)[1].vertices,
  gallery.modelFor("hood",base.model)[1].vertices);
});
test("seam matching is exact; unplanned ease/stretch is rejected",()=>{
 const d=hood.draft();
 assert.ok(d.checks.slice(0,2).every(c=>Math.abs(c.deltaMm)<.3));
 assert.equal(hood.seamCheck(105,100,"match",0,"test").status,"fail");
 assert.equal(hood.seamCheck(105,100,"ease",5,"test").status,"ok");
 assert.equal(hood.seamCheck(95,100,"stretch",5,"test").status,"ok");
 assert.equal(hood.seamCheck(105,100,"stretch",5,"test").status,"fail");
 assert.equal(hood.seamCheck(95,100,"ease",5,"test").status,"fail");
 const d2=hood.draft({backNeck:12,frontNeck:14,joinMode:"match"});
 assert.ok(d2.checks.every(c=>c.status==="ok"));
 assert.equal(d2.join.rearCorrection,5);
});
test("men and children cannot be mislabelled as verified DOB methods",()=>{
 for(const profile of ["men","child"]){
  const d=hood.render({profile});
  assert.equal(d.values.sourceProfileVerified,false);
  assert.ok(d.flags.some(s=>s.includes("fehlen")));
 }
 const d=hood.render({profile:"child",headCirc:51,backNeck:8,frontNeck:9});
 assert.equal(d.profile,"child");
 assert.ok(d.flags.some(s=>s.includes("Kordeln")));
 assert.throws(()=>hood.render({profile:"child",headCirc:65}),/Kinderprofil/);
 assert.equal(hood.render({profile:"women"}).values.sourceProfileVerified,true);
});
test("invalid body measures and unsupported modes fail closed",()=>{
 assert.throws(()=>hood.render({hoodHeight:"abc"}),/hoodHeight/);
 assert.throws(()=>hood.render({variant:"x"}),/Kapuzenform/);
 assert.throws(()=>hood.render({profile:"unknown"}),/Damen/);
 assert.throws(()=>hood.render({joinMode:"unknown"}),/Nahtverarbeitungsart/);
 assert.throws(()=>hood.render({headCirc:20}),/headCirc/);
 assert.throws(()=>hood.render({allowedEase:-1}),/Erlaubte/);
});
test("seam allowance and density alter estimate without falsifying sewn outline",()=>{
 const no=hood.render({},0,210),a=hood.render({},10,210),double=hood.render({},10,420);
 assert.equal(a.sewnArea,no.sewnArea);
 assert.ok(a.cutArea>no.cutArea);
 assert.ok(Math.abs(double.mass/a.mass-2)<1e-9);
 assert.ok(a.mass>15&&a.mass<100);
});
test("all garment presets now have actual finite geometry for the 3D viewer",()=>{
 for(const type of ["top","jacket","hood","trousers","skirt","beanie","hat","glove"]){
  const meshes=gallery.modelFor(type,hood.draft().model);
  assert.ok(meshes.length>=1,type);
  assert.ok(meshes.some(m=>m.faces.length>0),type);
  for(const mesh of meshes)for(const vertex of mesh.vertices)
   assert.ok(vertex.every(Number.isFinite),type);
 }
 const hoodModel=gallery.modelFor("hood",hood.draft().model);
 const obj=gallery.exportOBJ(hoodModel);
 assert.match(obj,/^# TextilLab/m);
 assert.match(obj,/^o Kapuze_3D/m);
 assert.match(obj,/^v -?\d/m);
 assert.match(obj,/^f \d/m);
 assert.ok(!obj.includes("NaN"));
});
test("3D renderer draws an actual triangle mesh and responds to orbit and zoom",()=>{
 const calls={paths:0,fills:0};
 const ctx=new Proxy({
  createLinearGradient(){return{addColorStop(){}}},
  beginPath(){calls.paths++},
  fill(){calls.fills++}
 },{get(target,k){return k in target?target[k]:(typeof k==="string"?()=>{}:undefined)}});
 const handlers={};
 const canvas={
  getContext(){return ctx},getBoundingClientRect(){return{width:450,height:420}},
  addEventListener(key,fn){handlers[key]=fn},setPointerCapture(){}
 };
 const viewer=gallery.createRenderer(canvas);
 viewer.setModel("hood",hood.draft().model);
 assert.ok(calls.fills>300);
 const before=viewer.state.yaw;
 handlers.pointerdown({clientX:120,clientY:90,pointerId:1});
 handlers.pointermove({clientX:170,clientY:110});
 handlers.pointerup({});
 assert.notEqual(viewer.state.yaw,before);
 handlers.wheel({deltaY:-80,preventDefault(){}});
 assert.ok(viewer.state.zoom>1);
 viewer.setOptions({wire:true,body:false});
 assert.ok(viewer.state.wire&&!viewer.state.body);
 viewer.reset();
 assert.equal(viewer.state.zoom,1);
});
test("HoodLab is present in gallery, forms and web app script dependencies",()=>{
 const html=fs.readFileSync(path.join(__dirname,"index.html"),"utf8");
 const app=fs.readFileSync(path.join(__dirname,"app.js"),"utf8");
 assert.ok(garments.types.some(t=>t[0]==="hood"));
 assert.ok(html.includes('<option value="hood">'));
 for(const label of ["hoodVariant","hoodProfile","hoodRuleReport",
   "garmentCanvas","garmentWire","garmentBody","garmentObj"]){
  assert.ok(html.includes('id="'+label+'"'),label);
 }
 assert.ok(html.includes("./hood-system.js"));
 assert.ok(html.includes("./garment-3d.js"));
 assert.ok(app.includes("TextilLabHoodSystem.render"));
 assert.ok(app.includes("garmentRenderer.setModel"));
});

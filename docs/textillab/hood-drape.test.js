"use strict";
const {test}=require("node:test");
const assert=require("node:assert/strict");
const fs=require("node:fs");
const path=require("node:path");
const hood=require("./hood-system.js");
const drape=require("./hood-drape.js");
const gallery=require("./garment-3d.js");

test("the 3D cloth is two genuine patterned panels welded at common center-seam vertices",()=>{
 const draft=hood.draft();
 const model=drape.construct(draft);
 const {grid,centerIds,uv}=model.topology;
 const cloth=model.meshes.at(-1);
 assert.equal(model.metrics.joinedCenterSeam,true);
 assert.equal(model.metrics.centerSeamGapMm,0);
 assert.equal(model.metrics.validForProduction,false);
 assert.equal(model.metrics.approximation,true);
 assert.ok(cloth.vertices.length>900&&cloth.faces.length>1800);
 assert.equal(grid.length,2);
 assert.equal(centerIds.length,37);
 for(let i=0;i<centerIds.length;i++){
  assert.equal(grid[0][i][0],centerIds[i]);
  assert.equal(grid[1][i][0],centerIds[i]);
 }
 for(const triangle of cloth.faces){
  assert.equal(new Set(triangle).size,3);
  assert.ok(triangle.every(idx=>idx>=0&&idx<cloth.vertices.length));
  assert.ok(triangle.every(idx=>uv[idx].every(Number.isFinite)));
 }
 for(const v of cloth.vertices) assert.ok(v.every(Number.isFinite));
 for(const type of ["center","face","neck"]){
  assert.ok(model.patternPanelEdges[type].length>3,type);
 }
 assert.ok(Math.abs(model.metrics.neckSeamMm-
   draft.values.backNeck-draft.values.frontNeck)<.1);
});
test("body height and head / bust / waist parameters actually reshape a 3D human",()=>{
 const a=drape.construct(hood.draft());
 const b=drape.construct(hood.draft({profile:"child"}));
 const wider=drape.construct(hood.draft(),{
  heightMm:1740,shoulderMm:470,chestMm:1080,waistMm:880,hipMm:1070,neckMm:390
 });
 assert.notEqual(a.mannequin.heightMm,b.mannequin.heightMm);
 assert.equal(a.mannequin.heightMm,1680);
 assert.equal(b.mannequin.heightMm,1330);
 assert.equal(wider.mannequin.shoulderMm,470);
 assert.notDeepEqual(a.meshes[2].vertices,wider.meshes[2].vertices);
 assert.ok(a.meshes.some(m=>m.fullBodyOnly));
 assert.ok(a.meshes.some(m=>m.name.includes("Oberschenkel")));
 assert.ok(a.meshes.some(m=>m.name.includes("Unterarm")));
 assert.equal(b.mannequin.profile,"child");
});
test("fabric presets, physical strength and gravity change the actual simulated drape",()=>{
 const draft=hood.draft();
 const jersey=drape.construct(draft,{}, {preset:"jersey",iterations:14});
 const rigid=drape.construct(draft,{}, {preset:"denim",iterations:14});
 assert.notEqual(jersey.material.stiffness,rigid.material.stiffness);
 assert.notEqual(jersey.material.drape,rigid.material.drape);
 assert.notDeepEqual(jersey.meshes.at(-1).vertices,rigid.meshes.at(-1).vertices);
 assert.ok(jersey.metrics.collisionCorrections>=0);
 assert.ok(jersey.metrics.rmsEdgeStrainPct>=0);
 assert.ok(rigid.metrics.rmsEdgeStrainPct>=0);
 const none=drape.construct(draft,{}, {preset:"cotton",drape:0,iterations:10});
 const gravity=drape.construct(draft,{}, {preset:"cotton",drape:1.5,iterations:10});
 assert.notDeepEqual(none.meshes.at(-1).vertices,gravity.meshes.at(-1).vertices);
});
test("no 3D sewing claims if stitching geometry is missing or invalid",()=>{
 const d=hood.draft();
 assert.throws(()=>drape.construct({parts:[{outline:[[0,0],[1,2]]}]}),/Schnittkurve|Schnittkanten/);
 const broken=structuredClone(d);
 broken.parts[0].edges.face[0][0]+=5;
 assert.throws(()=>drape.construct(broken),/nicht verbunden/);
 const mismatch=structuredClone(d);
 mismatch.checks[0].status="fail";
 assert.throws(()=>drape.construct(mismatch),/Nahtlängen/);
 assert.throws(()=>drape.construct(d,{heightMm:200}),/Körpermaß/);
 assert.throws(()=>drape.construct(d,{}, {stiffness:5}),/Stoffparameter/);
});
test("new topology can export OBJ with shared center seam and preserved 3D viewer API",()=>{
 const mockCtx=new Proxy({createLinearGradient:()=>({addColorStop(){}})},
  {get(obj,key){return key in obj?obj[key]:(typeof key==="string"?()=>{}:undefined)}});
 const handlers={};
 const canvas={getContext:()=>mockCtx,
  getBoundingClientRect:()=>({width:460,height:500}),
  addEventListener:(key,fn)=>{handlers[key]=fn},setPointerCapture(){}};
 const viewer=gallery.createRenderer(canvas),d=hood.draft();
 const metrics=viewer.setModel("hood",{...d.model,hoodDraft:d,mannequin:{heightMm:1700},
  fabric:{preset:"jersey",iterations:12}});
 assert.equal(metrics.centerSeamGapMm,0);
 assert.equal(viewer.state.report.joinedCenterSeam,true);
 assert.equal(viewer.state.clothMaterial.preset,"jersey");
 viewer.setOptions({fullBody:true});
 assert.equal(viewer.state.fullBody,true);
 viewer.setOptions({fullBody:false,wire:true,body:false});
 assert.equal(viewer.state.body,false);
 assert.ok(viewer.obj().includes("o Kapuze"));
 assert.ok(viewer.obj().includes("f "));
});
test("the editor exposes material settings, full-body and fit diagnostics",()=>{
 const html=fs.readFileSync(path.join(__dirname,"index.html"),"utf8");
 const app=fs.readFileSync(path.join(__dirname,"app.js"),"utf8");
 assert.ok(html.includes("./hood-drape.js"));
 assert.ok(html.indexOf("./hood-drape.js")<html.indexOf("./garment-3d.js"));
 for(const id of ["fitHeight","fitShoulder","fitChest","fitWaist","fitHip","fitNeck",
   "fabricPreset","fabricStiffness","fabricBending","fabricDrape",
   "fabricClearance","hoodDrapeReport","garmentFullBody"]){
  assert.ok(html.includes('id="'+id+'"'),id);
  assert.ok(app.includes(id),id);
 }
 const ids=[...html.matchAll(/\bid="([^"]+)"/g)].map(x=>x[1]);
 assert.equal(new Set(ids).size,ids.length,"All IDs must be unique");
});

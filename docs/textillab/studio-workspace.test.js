"use strict";
const {test}=require("node:test");
const assert=require("node:assert/strict");
const fs=require("node:fs");
const path=require("node:path");
const ui=require("./studio-workspace.js");
const pants=require("./pants-system.js");
const top=require("./bodice-v2.js");
const html=fs.readFileSync(path.join(__dirname,"index.html"),"utf8");

test("gallery offers every supported pattern type and clear workflow destinations",()=>{
 assert.deepEqual(ui.types.map(x=>x[0]),
   ["top","jacket","trousers","skirt","glove","beanie","hat","mesh"]);
 for(const id of ui.types.map(x=>x[0]))
   assert.ok(html.includes('<option value="'+id+'">'),id);
 for(const stage of ["template","measure","construction","check"])
   assert.ok(html.includes('data-step="'+stage+'"'),stage);
 assert.ok(html.includes('id="templateStrip"'));
});
test("context-sensitive body measurements expose correct relevant fields",()=>{
 assert.deepEqual(ui.coreFields("trousers"),["waist","hip","inseam"]);
 assert.ok(ui.coreFields("top").includes("shoulder"));
 assert.ok(ui.coreFields("glove").includes("handLength"));
 assert.equal(ui.coreFields("mesh").length,0);
});
test("pants and bodice waist-to-hip controls have separate unique IDs",()=>{
 const ids=[...html.matchAll(/\bid="([^"]+)"/g)].map(match=>match[1]);
 assert.equal(ids.length,new Set(ids).size,"All DOM IDs must be unique");
 for(const id of ["m-waistToHip","m-pantsWaistToHip","m-outseam",
  "zoomIn","zoomOut","zoomFit","focusDrawing","selectedPieceTitle"]){
   assert.ok(ids.includes(id),id);
 }
});
test("garment plot emits selectable piece identifiers matching cards",()=>{
 const pantSvg=pants.render().svg;
 const topSvg=top.render().svg;
 for(const code of ["VH","HH"])
  assert.ok(pantSvg.includes('data-piece-id="'+code+'"'));
 for(const code of ["VT","RT","AR"])
  assert.ok(topSvg.includes('data-piece-id="'+code+'"'));
 assert.equal(ui.partCode("trousers",0),"VH");
 assert.equal(ui.partCode("trousers",1),"HH");
 assert.equal(ui.partCode("top",2),"AR");
});
test("source comparison links to official public pattern not a pretend copy",()=>{
 assert.ok(html.includes("https://www.muellerundsohn.com/allgemein/schnittmuster-grundschnitt-hose-und-bundfaltenhose/"));
 assert.ok(html.includes("noch nicht als Konstruktionsregeln übernommen"));
 assert.ok(html.includes("studio-workspace.css"));
 assert.ok(html.includes("studio-workspace.js"));
});

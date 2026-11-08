"use strict";
const {test}=require("node:test");
const assert=require("node:assert/strict");
const pants=require("./pants-system.js");
const registry=require("./ruleset-registry.js");

test("front and back pants are different curved pieces with defined landmarks",()=>{
 const d=pants.draft();
 assert.equal(d.pieces.length,2);
 const [f,b]=d.pieces;
 assert.equal(f.count,2);
 assert.equal(b.count,2);
 assert.ok(f.outline.length>100);
 assert.ok(b.svgPath.split(" C ").length>=6);
 assert.ok(b.darts.length===3);
 assert.ok(b.seams.rise>f.seams.rise);
 assert.notDeepEqual(f.outline,b.outline);
 for(const id of ["WC","WS","HS","HC","CR","RS","KS","KI","AS","AI"]) {
   assert.ok(f.marks[id],id);
 }
});
test("cut pattern renders in SolidCraft style with labeled construction heights",()=>{
 const d=pants.render();
 assert.equal(d.pieces,4);
 assert.equal(d.parts.length,2);
 assert.ok(d.svg.includes('data-pants-system="textillab.pants.experimental.1"'));
 assert.ok(d.svg.includes("VH")&&d.svg.includes("HH"));
 assert.ok(d.svg.includes("Sch")&&d.svg.includes("Hü")&&d.svg.includes("Kn"));
 assert.ok(d.svg.includes("FDL"));
 assert.ok(d.svg.includes("#14202e"));
 assert.ok(d.cutArea>d.sewnArea);
 assert.ok(d.mass>0);
 assert.ok(!d.svg.includes("NaN")&&!d.svg.includes("Infinity"));
});
test("body measurements change geometry, not merely output scaling",()=>{
 const orig=pants.render();
 for(const [key,value] of Object.entries({
   waist:92,hip:115,inseam:81,outseam:110,
   waistToKnee:62,thigh:68,knee:54,hem:48,
   crotchExtension:2
 })) {
   const altered=pants.render({[key]:value});
   assert.notEqual(altered.svg,orig.svg,key);
 }
 const back=pants.draft({dartIntake:3}).pieces[1];
 assert.notDeepEqual(back.darts,pants.draft().pieces[1].darts);
});
test("seam checks and provenance are transparent",()=>{
 const d=pants.draft();
 assert.equal(d.rules.length,10);
 assert.ok(d.rules.every(r=>r.id&&r.formula&&r.status));
 assert.ok(d.checks.sideDifferenceMm<5);
 assert.ok(d.checks.inseamDifferenceMm<9);
 assert.equal(d.rules.find(r=>r.id==="H03").value,280);
 assert.ok(d.warnings.some(w=>w.includes("Probeteil")));
});
test("reject implausible garment proportions",()=>{
 assert.throws(()=>pants.render({outseam:98}),/Leibhöhe/);
 assert.throws(()=>pants.render({waistToKnee:25}),/Reihenfolge/);
 assert.throws(()=>pants.render({waist:180,hip:85}),/Verhältnis/);
 assert.throws(()=>pants.render({hipEase:-5}),/Zugabe/);
 assert.throws(()=>pants.render({},-2),/Nahtzugabe/);
});
test("reference comparison requires exact printed source and matching body measurements",()=>{
 const d=pants.render();
 const book={title:"Eigene Kontrollkonstruktion",edition:"Probe 1",pages:"12"};
 const sample={schema:"textillab-reference/1",system:"independent-prototype",
  garment:"trousers",book,
  inputCm:{waist:84,hip:102,inseam:80,outseam:108,crotchDepth:28},
  resultsMm:{sideFront:d.parts[0].seams.side,inseamBack:d.parts[1].seams.inseam}};
 const a=registry.compare(sample,d);
 assert.equal(a.compared,2);
 assert.ok(a.differences.every(x=>x.absDifferenceMm===0));
 assert.throws(()=>registry.compare({...sample,inputCm:{...sample.inputCm,hip:110}},d),/abweichende/);
 assert.throws(()=>registry.compare({...sample,book:{title:"",edition:"",pages:""}},d),/Buchtitel/);
 assert.ok(registry.rulesets.find(x=>x.id==="muller-und-sohn").status.includes("awaiting"));
});


test("HTML number inputs are strings and must still render numeric pants dimensions",()=>{
 const strings=Object.fromEntries(
   Object.entries(pants.defaults).map(([name,value])=>[name,String(value)])
 );
 const parsed=pants.draft(strings);
 assert.equal(typeof parsed.bodyValues.outseam,"number");
 assert.equal(typeof parsed.bodyValues.knee,"number");
 assert.equal(parsed.bodyValues.outseam,108);
 assert.equal(parsed.bodyValues.knee,48);
 const svg=pants.render(strings).svg;
 assert.equal(svg,pants.render().svg);
 assert.match(svg,/SL 108\.0 cm/);
 assert.match(svg,/Kn 24\.0 cm/);
 assert.ok(!svg.includes("NaN")&&!svg.includes("Infinity"));
 const modified=pants.render({...strings,outseam:"110",inseam:"82"}).svg;
 assert.match(modified,/SL 110\.0 cm/);
 assert.notEqual(modified,svg);
 assert.throws(()=>pants.render({...strings,outseam:"abc"}),/Ungültiges Hosenmaß/);
});

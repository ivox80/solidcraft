/* TextilLab verified-source comparison contract.
 * This does not contain formulas or images from proprietary books.
 * Users may enter their own control dimensions from lawful personal copies.
 */
(function(root){
"use strict";
const RULESETS=Object.freeze([
 {id:"textillab.pants.experimental.1",label:"TextilLab Experimental Pants",
  status:"implemented",copyright:"Independent original prototype",source:"docs/TEXTILLAB_PANTS_RULES_V11_DE.md"},
 {id:"muller-und-sohn",label:"M. Müller & Sohn",status:"awaiting-rule-transcription",
  copyright:"Proprietary textbooks: user-owned copies do not imply redistribution rights",
  source:"Provide exact book title, edition, page and rule verification before implementation"},
 {id:"aldrich",label:"Metric Pattern Cutting",status:"not-implemented",
  copyright:"Published text/diagrams not included",source:"Bibliographic reference only"},
 {id:"free-sewing",label:"FreeSewing",status:"architecture-reference",
  copyright:"Respect original code licenses and attribution",source:"https://freesewing.dev/"}
]);
const KEYS=Object.freeze({
 sideFront:"Vorderhose Seitennaht",
 sideBack:"Hinterhose Seitennaht",
 inseamFront:"Vorderhose Innenbein",
 inseamBack:"Hinterhose Innenbein",
 waistFront:"Vorderhose Taille",
 waistBack:"Hinterhose Taille",
 crotchFront:"Vorderhose Schrittkurve",
 crotchBack:"Hinterhose Schrittkurve"
});
function measurementsFrom(draft) {
 return {
  waist:draft.bodyValues.waist,hip:draft.bodyValues.hip,
  inseam:draft.bodyValues.inseam,outseam:draft.bodyValues.outseam,
  crotchDepth:draft.bodyValues.crotchDepth
 };
}
function actual(draft) {
 const [front,back]=draft.parts||draft.pieces;
 return {
  sideFront:front.seams.side,sideBack:back.seams.side,
  inseamFront:front.seams.inseam,inseamBack:back.seams.inseam,
  waistFront:front.seams.waist,waistBack:back.seams.waist,
  crotchFront:front.seams.rise,crotchBack:back.seams.rise
 };
}
function compare(reference,draft) {
 if(!reference||reference.schema!=="textillab-reference/1"||reference.garment!=="trousers")
   throw Error("Ungültiges Hosen-Referenzblatt (Schema v1).");
 if(!reference.system || reference.system==="textillab.pants.experimental.1")
   throw Error("Referenz muss von einem getrennten, überprüfbaren Schnittsystem stammen.");
 if(!reference.book?.title||!reference.book?.edition||!reference.book?.pages)
   throw Error("Buchtitel, Ausgabe und Seitenangabe fehlen.");
 if(!reference.inputCm || !reference.resultsMm || typeof reference.resultsMm!=="object")
   throw Error("Körpermaße und Kontrollwerte fehlen.");
 const expected=measurementsFrom(draft);
 const mismatched=[];
 for(const [key,value]of Object.entries(expected)) {
   const actualVal=Number(reference.inputCm[key]);
   if(!Number.isFinite(actualVal)||Math.abs(actualVal-value)>0.5)mismatched.push(key);
 }
 if(mismatched.length)
   throw Error("Referenz verwendet abweichende Körpermaße: "+mismatched.join(", "));
 const ours=actual(draft),diffs=[];
 for(const [key,value]of Object.entries(reference.resultsMm)){
   if(!(key in KEYS)||value==null)continue;
   const n=Number(value);
   if(!Number.isFinite(n)||n<0||n>3000)throw Error("Ungültiges Referenzmaß: "+key);
   diffs.push({id:key,label:KEYS[key],referenceMm:n,textillabMm:ours[key],
     differenceMm:ours[key]-n,absDifferenceMm:Math.abs(ours[key]-n)});
 }
 if(!diffs.length)throw Error("Keine ausgefüllten Referenzmessungen vorhanden.");
 return {system:reference.system,book:reference.book,compared:diffs.length,
   measurementsMatched:true,differences:diffs};
}
const API={rulesets:RULESETS,keys:KEYS,compare,measurementsFrom,actual};
if(typeof module!=="undefined"&&module.exports)module.exports=API;
root.TextilLabRulesetRegistry=API;
})(typeof globalThis!=="undefined"?globalThis:this);

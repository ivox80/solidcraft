//! Native CPU inference for the V0.4 1,624-parameter profile-ranking MLP.
//! The neural estimate proposes candidates only. Always check any selected
//! geometry with the mechanical solver before treating it as optimized.
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct TinyProfileRanker {
    #[serde(rename="W1")] pub w1:Vec<Vec<f64>>,
    #[serde(rename="b1")] pub b1:Vec<f64>,
    #[serde(rename="W2")] pub w2:Vec<Vec<f64>>,
    #[serde(rename="b2")] pub b2:Vec<f64>,
    #[serde(rename="W3")] pub w3:Vec<Vec<f64>>,
    #[serde(rename="b3")] pub b3:Vec<f64>,
    pub names:Vec<String>,
    pub mean:Vec<f64>,
    pub scale:Vec<f64>,
}
impl TinyProfileRanker {
    pub fn from_json(json:&str)->Result<Self,String> {
        let w:Self=serde_json::from_str(json).map_err(|e|e.to_string())?;
        if w.mean.len()!=29 || w.scale.len()!=29 ||
            w.w1.len()!=29 || w.w1.iter().any(|a|a.len()!=32) || w.b1.len()!=32 ||
            w.w2.len()!=32 || w.w2.iter().any(|a|a.len()!=16) || w.b2.len()!=16 ||
            w.w3.len()!=16 || w.w3.iter().any(|a|a.len()!=8) || w.b3.len()!=8 ||
            w.names.len()!=8 || w.scale.iter().any(|&x|x<=0.0 || !x.is_finite()) {
            return Err("unexpected TinyProfileRanker weight shape".into());
        }
        Ok(w)
    }
    pub fn bundled()->Result<Self,String> {
        Self::from_json(include_str!("../models/tiny_profile_ranker_v04.json"))
    }
    /// Reproduce the Python V0.4 inference: 29 -> 32 (tanh) -> 16 (tanh) -> 8.
    pub fn predict(&self,x:&[f64;29])->[f64;8] {
        let mut z=[0.0_f64;29];
        for i in 0..29 {z[i]=((x[i]-self.mean[i])/self.scale[i]).clamp(-6.0,6.0);}
        let h1=dense(&z,&self.w1,&self.b1,true);
        let h2=dense(&h1,&self.w2,&self.b2,true);
        let scores=dense(&h2,&self.w3,&self.b3,false);
        let mut out=[0.0;8];out.copy_from_slice(&scores);out
    }
    /// Ascending scores: candidates with the lowest predicted compliance first.
    pub fn rank(&self,x:&[f64;29])->Vec<(&str,f64)> {
        let scores=self.predict(x);
        let mut result:Vec<_>=self.names.iter().enumerate().map(|(i,n)|(n.as_str(),scores[i])).collect();
        result.sort_by(|a,b|a.1.total_cmp(&b.1));result
    }
}
fn dense(x:&[f64],weight:&[Vec<f64>],bias:&[f64],activation:bool)->Vec<f64>{
    let mut y=bias.to_vec();
    for (i,&xi) in x.iter().enumerate(){for (j,yj) in y.iter_mut().enumerate(){*yj+=xi*weight[i][j];}}
    if activation {for v in &mut y {*v=v.tanh();}}
    y
}
/// Match `tiny_profile_ranker.py::force_features` exactly.
/// Input: member end forces Fx,Fy,Fz,Mx,My,Mz in the member local basis.
pub fn force_features(force:[f64;6],length:f64,area:f64)->Result<[f64;29],String>{
    if length<=0.0||area<=0.0||!length.is_finite()||!area.is_finite()||
        force.iter().any(|f|!f.is_finite()) {return Err("invalid load, length or area".into());}
    let mut v=[0.0;6];
    for i in 0..6{v[i]=force[i]*if i<3 {1.0}else{1.0/length};}
    let mag=v.iter().map(|a|a*a).sum::<f64>().sqrt().max(1e-15);
    for a in &mut v {*a/=mag;}
    let mut x=[0.0;29];x[..6].copy_from_slice(&v);
    let mut index=6;
    for i in 0..6{for j in i..6{x[index]=v[i]*v[j];index+=1;}}
    x[27]=length.ln();x[28]=area.ln();
    Ok(x)
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn weights_and_feature_layout(){
        let net=TinyProfileRanker::bundled().unwrap();
        let x=force_features([0.0,100.0,80.0,0.0,12.0,0.0],1.5,0.018).unwrap();
        let scores=net.predict(&x);
        assert!(scores.iter().all(|s|s.is_finite()));
        // Golden values from the original Python V0.4 NumPy model.
        let python_scores=[-0.055535323634,-0.075372069905,0.362269519104,
            -0.077786938120,0.400474495869,-0.041986493922,
            -0.075672564477,-0.438387508161];
        for i in 0..8 {assert!((scores[i]-python_scores[i]).abs()<1e-8);}
        assert_eq!(net.rank(&x).len(),8);
        assert!((x[27]-1.5_f64.ln()).abs()<1e-12);
    }
}
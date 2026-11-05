use solidcraft_skeleton_ai::{generate_mesh, SkeletonDocument};
use std::{env, fs, path::Path};
fn main() {
    if let Err(e)=run(){eprintln!("skeleton-ai: {e}");std::process::exit(1);}
}
fn run()->Result<(),String>{
    let mut input=String::new();let mut output=String::from("skeleton.obj");
    let mut resolution=64_usize;let mut blend=0.12_f64;let mut junction=0.35_f64;
    let mut args=env::args().skip(1);
    while let Some(arg)=args.next(){
        let val=args.next().ok_or_else(||format!("missing value for {arg}"))?;
        match arg.as_str(){
            "--input"=>input=val,
            "--output"=>output=val,
            "--resolution"=>resolution=val.parse().map_err(|_|"invalid resolution")?,
            "--blend"=>blend=val.parse().map_err(|_|"invalid blend")?,
            "--junction"=>junction=val.parse().map_err(|_|"invalid junction")?,
            _=>return Err(format!("unknown option {arg}")),
        }
    }
    if input.is_empty(){return Err("Usage: skeleton-ai --input file.json --output model.obj|model.stl [--resolution 64] [--blend 0.12] [--junction 0.35]".into());}
    let contents=fs::read_to_string(&input).map_err(|e|e.to_string())?;
    let doc:SkeletonDocument=serde_json::from_str(&contents).map_err(|e|e.to_string())?;
    let skeleton=doc.to_skeleton(blend,junction)?;
    let mesh=generate_mesh(&skeleton,resolution,8_000_000)?;
    if !mesh.edge_incidence_watertight(){return Err("non-manifold mesh: try a higher resolution or adjust the profiles".into());}
    let bytes=if Path::new(&output).extension().and_then(|e|e.to_str())==Some("stl") {mesh.to_binary_stl()}else {mesh.to_obj().into_bytes()};
    fs::write(&output,bytes).map_err(|e|e.to_string())?;
    println!("Saved {output}: {} vertices, {} triangles; watertight={}",mesh.vertices.len(),mesh.faces.len(),mesh.edge_incidence_watertight());
    Ok(())
}
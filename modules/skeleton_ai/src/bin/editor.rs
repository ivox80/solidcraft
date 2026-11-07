//! Standalone localhost editor for the SolidCraft Skeleton AI Rust geometry crate.
//! The actual SolidCraft host application is not available in this repository;
//! this server/editor is the integration-ready UI prototype, not a host plugin.
use serde::{Deserialize, Serialize};
use solidcraft_skeleton_ai::{generate_mesh, SkeletonDocument};
use std::{env, fs, io::{Read, Write}, net::{TcpListener, TcpStream}, path::PathBuf, sync::{Arc, Mutex}, time::Duration};

const HTML: &str = include_str!("../../assets/editor.html");
const MAX_BODY: usize = 2 * 1024 * 1024;
const MAX_HEADER: usize = 16 * 1024;

#[derive(Clone)]
struct State { document: SkeletonDocument, input: PathBuf, blend: f64, junction: f64 }

#[derive(Serialize)]
struct MeshPayload { vertices: Vec<[f64;3]>, faces: Vec<[u32;3]>, watertight: bool, triangles: usize }

#[derive(Deserialize)]
struct Resolution { resolution: Option<usize> }

#[derive(Debug)]
struct Request { method: String, uri: String, headers: Vec<(String,String)>, body: Vec<u8> }
impl Request {
    fn header(&self, key: &str) -> Option<&str> {
        self.headers.iter().find(|(k,_)| k.eq_ignore_ascii_case(key)).map(|(_,v)|v.as_str())
    }
}

fn read_request(stream: &mut TcpStream) -> Result<Request, String> {
    let mut buf: Vec<u8> = Vec::new(); let mut chunk = [0u8;4096];
    let end = loop {
        if let Some(i)=buf.windows(4).position(|w|w==b"\r\n\r\n") { break i+4; }
        if buf.len()>MAX_HEADER {return Err("HTTP headers too large".into());}
        let n=stream.read(&mut chunk).map_err(|e|e.to_string())?;
        if n==0 {return Err("connection ended before headers".into());}
        buf.extend_from_slice(&chunk[..n]);
    };
    if end>MAX_HEADER{return Err("HTTP headers too large".into());}
    let head=std::str::from_utf8(&buf[..end]).map_err(|_|"invalid HTTP header encoding")?;
    let mut lines=head.split("\r\n");
    let first=lines.next().ok_or("missing request line")?;
    let tokens:Vec<&str>=first.split_whitespace().collect();
    if tokens.len()!=3 || !tokens[2].starts_with("HTTP/1.") {return Err("invalid request line".into());}
    let headers:Vec<(String,String)>=lines.filter_map(|line|line.split_once(':').map(|(k,v)|(k.trim().to_string(),v.trim().to_string()))).collect();
    let len=headers.iter().find(|(k,_)|k.eq_ignore_ascii_case("content-length"))
        .map(|(_,v)|v.parse::<usize>().map_err(|_|"invalid content length")).transpose()?.unwrap_or(0);
    if len>MAX_BODY{return Err("request body too large".into());}
    let mut body=buf[end..].to_vec();
    if body.len()>len {body.truncate(len);}
    while body.len()<len {
        let n=stream.read(&mut chunk).map_err(|e|e.to_string())?;
        if n==0{return Err("incomplete request body".into());}
        body.extend_from_slice(&chunk[..n.min(len-body.len())]);
    }
    Ok(Request{method:tokens[0].into(),uri:tokens[1].into(),headers,body})
}

fn respond(stream: &mut TcpStream, code:u16, mime:&str, body:&[u8], filename:Option<&str>) -> std::io::Result<()> {
    let reason=match code {200=>"OK",400=>"Bad Request",403=>"Forbidden",404=>"Not Found",405=>"Method Not Allowed",413=>"Payload Too Large",422=>"Unprocessable Entity",500=>"Internal Server Error",_=>"Error"};
    let disposition=filename.map(|f| format!("Content-Disposition: attachment; filename=\"{f}\"\r\n")).unwrap_or_default();
    let head=format!("HTTP/1.1 {code} {reason}\r\nContent-Type: {mime}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nX-Frame-Options: DENY\r\n{disposition}Connection: close\r\n\r\n",body.len());
    stream.write_all(head.as_bytes())?;stream.write_all(body)
}
fn json_error(message:&str)->Vec<u8>{serde_json::to_vec(&serde_json::json!({"error":message})).unwrap_or_default()}
fn generate(state:&State,res:usize)->Result<solidcraft_skeleton_ai::Mesh,String> {
    if !(16..=80).contains(&res){return Err("Auflösung muss 16–80 sein".into());}
    let skeleton=state.document.to_skeleton(state.blend,state.junction)?;
    generate_mesh(&skeleton,res,2_000_000)
}
fn guarded_write(req:&Request,port:u16)->bool {
    if req.header("x-solidcraft-editor")!=Some("1") || !req.header("content-type").unwrap_or("").starts_with("application/json") {return false;}
    let expected=[format!("http://127.0.0.1:{port}"),format!("http://localhost:{port}")];
    req.header("origin").is_none_or(|origin|expected.iter().any(|e|e==origin))
}
fn parse_resolution(uri:&str,default:usize)->usize {
    uri.split_once('?').and_then(|(_,query)|query.split('&').find_map(|part|part.strip_prefix("resolution=")?.parse::<usize>().ok())).unwrap_or(default)
}
fn handle(req:Request,shared:&Arc<Mutex<State>>,stream:&mut TcpStream,port:u16) -> std::io::Result<()> {
    let path=req.uri.split('?').next().unwrap_or("/");
    let failure=|code,msg:String,stream:&mut TcpStream|respond(stream,code,"application/json; charset=utf-8",&json_error(&msg),None);
    match (req.method.as_str(),path) {
        ("GET","/") | ("GET","/editor") => respond(stream,200,"text/html; charset=utf-8",HTML.as_bytes(),None),
        ("GET","/api/health")=>respond(stream,200,"application/json",br#"{"ok":true,"geometry":"Rust SDF","editor":"localhost"}"#,None),
        ("GET","/api/state") => {
            let state=shared.lock().unwrap();
            let data=serde_json::to_vec(&state.document).unwrap_or_default();
            respond(stream,200,"application/json",&data,None)
        }
        ("POST","/api/state") => {
            if !guarded_write(&req,port){return failure(403,"Editor-Header/Origin fehlt".into(),stream);}
            let parsed:Result<SkeletonDocument,_>=serde_json::from_slice(&req.body);
            let doc=match parsed{Ok(d)=>d,Err(e)=>return failure(400,format!("Ungültiges JSON: {e}"),stream)};
            let (blend,junction)={let state=shared.lock().unwrap();(state.blend,state.junction)};
            if let Err(e)=doc.to_skeleton(blend,junction){return failure(422,e,stream);}
            shared.lock().unwrap().document=doc;
            respond(stream,200,"application/json",br#"{"ok":true}"#,None)
        }
        ("GET","/api/mesh") => {
            let res=parse_resolution(&req.uri,34);
            let state=shared.lock().unwrap().clone();
            match generate(&state,res){
                Ok(mesh)=>{
                    let payload=MeshPayload{triangles:mesh.faces.len(),watertight:mesh.edge_incidence_watertight(),vertices:mesh.vertices,faces:mesh.faces};
                    match serde_json::to_vec(&payload){Ok(data)=>respond(stream,200,"application/json",&data,None),Err(e)=>failure(500,e.to_string(),stream)}
                },
                Err(e)=>failure(422,e,stream)
            }
        }
        ("GET","/api/download") => {
            let res=parse_resolution(&req.uri,64);
            let kind=req.uri.split_once('?').map(|(_,s)|s.split('&').any(|x|x=="format=stl")).unwrap_or(false);
            let state=shared.lock().unwrap().clone();
            match generate(&state,res){
                Ok(mesh) if !mesh.edge_incidence_watertight()=>failure(422,"Netz ist nicht geschlossen; bitte Profil/Auflösung prüfen".into(),stream),
                Ok(mesh)=>if kind { respond(stream,200,"model/stl",&mesh.to_binary_stl(),Some("skeleton.stl")) }
                            else { respond(stream,200,"text/plain; charset=utf-8",mesh.to_obj().as_bytes(),Some("skeleton.obj")) },
                Err(e)=>failure(422,e,stream),
            }
        }
        ("POST","/api/save")=>{
            if !guarded_write(&req,port){return failure(403,"Editor-Header/Origin fehlt".into(),stream);}
            let state=shared.lock().unwrap().clone();
            let filename=state.input.file_stem().and_then(|s|s.to_str()).unwrap_or("skeleton");
            let destination=state.input.with_file_name(format!("{filename}.edited.json"));
            let data=serde_json::to_vec_pretty(&state.document).unwrap_or_default();
            match fs::write(&destination,data){
                Ok(_)=>{let p=serde_json::json!({"ok":true,"path":destination.display().to_string()});respond(stream,200,"application/json",p.to_string().as_bytes(),None)},
                Err(e)=>failure(500,e.to_string(),stream)
            }
        }
        ("OPTIONS",_)=>failure(403,"Cross-origin requests are not enabled".into(),stream),
        ("GET",_)=>failure(404,"Unbekannter Pfad".into(),stream),
        _=>failure(405,"Methode nicht erlaubt".into(),stream),
    }
}
fn run()->Result<(),String>{
    let mut input=PathBuf::from("examples/two_sided_profiles.json");
    let mut port=8765_u16;let mut check=false;
    let mut args=env::args().skip(1);
    while let Some(arg)=args.next(){match arg.as_str(){
        "--input"=>input=PathBuf::from(args.next().ok_or("missing --input value")?),
        "--port"=>port=args.next().ok_or("missing --port value")?.parse().map_err(|_|"invalid port")?,
        "--check"=>check=true,
        _=>return Err(format!("unbekannte Option {arg}"))
    }}
    let doc:SkeletonDocument=serde_json::from_str(&fs::read_to_string(&input).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
    doc.to_skeleton(0.12,0.35)?;
    let state=State{document:doc,input,blend:0.12,junction:0.35};
    if check {
        let mesh=generate(&state,32)?;
        println!("Skeleton editor: check OK, {} faces, closed={}",mesh.faces.len(),mesh.edge_incidence_watertight());
        return Ok(());
    }
    let listener=TcpListener::bind(("127.0.0.1",port)).map_err(|e|e.to_string())?;
    println!("SolidCraft Skeleton Editor: http://127.0.0.1:{port}");
    println!("Open the address in your browser; Ctrl+C to stop.");
    let shared=Arc::new(Mutex::new(state));
    for connection in listener.incoming(){
        match connection {
            Ok(mut stream)=>{
                let _=stream.set_read_timeout(Some(Duration::from_secs(15)));
                let _=stream.set_write_timeout(Some(Duration::from_secs(30)));
                match read_request(&mut stream) {
                    Ok(req)=>{if let Err(e)=handle(req,&shared,&mut stream,port){eprintln!("response: {e}");}},
                    Err(e)=>{let _=respond(&mut stream,400,"application/json",&json_error(&e),None);}
                }
            },
            Err(e)=>eprintln!("connection: {e}")
        }
    }
    Ok(())
}
fn main(){if let Err(e)=run(){eprintln!("skeleton-editor: {e}");std::process::exit(1);}}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn example_stations_can_be_changed_at_both_ends(){
        let doc:SkeletonDocument=serde_json::from_str(include_str!("../../examples/two_sided_profiles.json")).unwrap();
        let mut other=doc.clone();
        other.section_stations.get_mut("0").unwrap().stations[0].width=Some(0.25);
        assert_ne!(doc.section_stations["0"].stations[0].width,other.section_stations["0"].stations[0].width);
        assert_eq!(doc.section_stations["0"].stations[2].width,other.section_stations["0"].stations[2].width);
        other.to_skeleton(0.12,0.35).unwrap();
    }
    #[test]
    fn rejects_oversize_grid(){
        let doc:SkeletonDocument=serde_json::from_str(include_str!("../../examples/two_sided_profiles.json")).unwrap();
        let state=State{document:doc,input:PathBuf::from("test"),blend:0.12,junction:0.35};
        assert!(generate(&state,81).is_err());
    }
    #[test]
    fn query_parameters_resolution(){
        assert_eq!(parse_resolution("/api/mesh?resolution=45",32),45);
        assert_eq!(parse_resolution("/api/mesh?format=stl&resolution=55",32),55);
        assert_eq!(parse_resolution("/api/mesh?junk=yes",32),32);
    }
}
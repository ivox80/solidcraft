//! TextilLab Studio local-only web UI backed by Rust geometry calculations.
//! Start with: cargo run --release --bin textillab-studio
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;
use textillab::obj_import::{parse_obj, parse_seam};
use textillab::studio_patterns::make_cut_preview;

const HOST: &str = "127.0.0.1:27840";
const MAX_BODY: usize = 4 * 1024 * 1024;

fn percent_decode(input: &str) -> Result<String, String> {
    let bytes = input.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' => {
                if i + 2 >= bytes.len() {
                    return Err("Truncated percent encoding".into());
                }
                let digit = |b: u8| -> Option<u8> {
                    match b {
                        b'0'..=b'9' => Some(b - b'0'),
                        b'A'..=b'F' => Some(b - b'A' + 10),
                        b'a'..=b'f' => Some(b - b'a' + 10),
                        _ => None,
                    }
                };
                let hi = digit(bytes[i + 1]).ok_or("Invalid percent encoding")?;
                let lo = digit(bytes[i + 2]).ok_or("Invalid percent encoding")?;
                decoded.push(hi * 16 + lo);
                i += 3;
            }
            b'+' => {
                decoded.push(b' ');
                i += 1;
            }
            other => {
                decoded.push(other);
                i += 1;
            }
        }
    }
    String::from_utf8(decoded).map_err(|_| "Invalid UTF-8 in form data".into())
}

fn parse_form(body: &str) -> Result<BTreeMap<String, String>, String> {
    let mut form = BTreeMap::new();
    for pair in body.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (key, value) = pair
            .split_once('=')
            .ok_or("Malformed form parameter")?;
        form.insert(percent_decode(key)?, percent_decode(value)?);
    }
    Ok(form)
}

fn generate_from_form(body: &str) -> Result<String, String> {
    let fields = parse_form(body)?;
    let obj = fields.get("obj").ok_or("OBJ model missing")?;
    let mut mesh = parse_obj(obj)?;
    if mesh.faces.len() > 10_000 {
        return Err("Studio preview supports up to 10,000 triangles".into());
    }
    let factor = match fields.get("unit").map(String::as_str).unwrap_or("mm") {
        "mm" => 1.0,
        "cm" => 10.0,
        "m" => 1000.0,
        _ => return Err("Unknown length unit".into()),
    };
    for vertex in &mut mesh.vertices {
        vertex.x *= factor;
        vertex.y *= factor;
        vertex.z *= factor;
    }
    mesh.validate()?;
    if let Some(seams) = fields.get("seams") {
        for seam in seams.split(',').filter(|value| !value.trim().is_empty()) {
            let edge = parse_seam(seam, &mesh)?;
            mesh.seams.insert(edge);
        }
    }
    let allowance: f64 = fields
        .get("allowance")
        .map(String::as_str)
        .unwrap_or("0")
        .parse()
        .map_err(|_| "Invalid seam allowance")?;
    let density: f64 = fields
        .get("density")
        .map(String::as_str)
        .unwrap_or("210")
        .parse()
        .map_err(|_| "Invalid material density")?;
    Ok(make_cut_preview(&mesh, allowance, density)?.svg)
}

fn response(status: &str, kind: &str, body: &str) -> Vec<u8> {
    let headers = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {kind}\r\nContent-Length: {}\r\nConnection: close\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nContent-Security-Policy: default-src 'self'; script-src 'self'; style-src 'self'; connect-src 'self'; img-src 'self' blob: data:\r\n\r\n",
        body.len()
    );
    [headers.as_bytes(), body.as_bytes()].concat()
}

fn read_request(stream: &mut TcpStream) -> Result<(String, String, String), String> {
    let mut buf = Vec::new();
    let mut chunk = [0; 8192];
    let header_end = loop {
        let n = stream.read(&mut chunk).map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("Client closed connection".into());
        }
        buf.extend_from_slice(&chunk[..n]);
        if let Some(index) = buf.windows(4).position(|v| v == b"\r\n\r\n") {
            break index + 4;
        }
        if buf.len() > 16_384 {
            return Err("HTTP headers too large".into());
        }
    };
    let header = String::from_utf8(buf[..header_end].to_vec())
        .map_err(|_| "Invalid request headers")?;
    let first = header.lines().next().ok_or("Request line missing")?;
    let mut parts = first.split_whitespace();
    let method = parts.next().ok_or("Method missing")?.to_owned();
    let path = parts.next().ok_or("Path missing")?.to_owned();
    if !matches!(method.as_str(), "GET" | "POST") {
        return Err("Unsupported HTTP method".into());
    }
    let mut content_length = 0_usize;
    let mut origin = None;
    let mut host = None;
    for line in header.lines().skip(1) {
        if let Some((k, v)) = line.split_once(':') {
            match k.trim().to_ascii_lowercase().as_str() {
                "content-length" => {
                    content_length = v
                        .trim()
                        .parse()
                        .map_err(|_| "Invalid Content-Length")?;
                }
                "origin" => origin = Some(v.trim().to_string()),
                "host" => host = Some(v.trim().to_string()),
                _ => {}
            }
        }
    }
    if host.as_deref() != Some(HOST) {
        return Err("Invalid Host header".into());
    }
    if origin.is_some() && origin.as_deref() != Some("http://127.0.0.1:27840") {
        return Err("Cross-origin request refused".into());
    }
    if content_length > MAX_BODY {
        return Err("Model too large (4 MB request limit)".into());
    }
    let target_len = header_end + content_length;
    if buf.len() > target_len {
        buf.truncate(target_len);
    }
    while buf.len() < target_len {
        let n = stream.read(&mut chunk).map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("Incomplete HTTP request".into());
        }
        let remaining = target_len - buf.len();
        buf.extend_from_slice(&chunk[..n.min(remaining)]);
    }
    let body = String::from_utf8(buf[header_end..].to_vec())
        .map_err(|_| "Invalid UTF-8 request body")?;
    Ok((method, path, body))
}

fn serve(mut stream: TcpStream) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));
    let result = read_request(&mut stream);
    let response_bytes = match result {
        Ok((method, path, body)) => match (method.as_str(), path.as_str()) {
            ("GET", "/") => response("200 OK", "text/html; charset=utf-8", include_str!("../../studio/index.html")),
            ("GET", "/app.js") => response("200 OK", "text/javascript; charset=utf-8", include_str!("../../studio/app.js")),
            ("GET", "/style.css") => response("200 OK", "text/css; charset=utf-8", include_str!("../../studio/style.css")),
            ("GET", "/example.obj") => response("200 OK", "text/plain; charset=utf-8", include_str!("../../examples/bent-panel.obj")),
            ("POST", "/api/unfold") => match generate_from_form(&body) {
                Ok(svg) => response("200 OK", "image/svg+xml; charset=utf-8", &svg),
                Err(error) => response("422 Unprocessable Content", "text/plain; charset=utf-8", &error),
            },
            _ => response("404 Not Found", "text/plain; charset=utf-8", "Not found"),
        },
        Err(error) => response("400 Bad Request", "text/plain; charset=utf-8", &error),
    };
    let _ = stream.write_all(&response_bytes);
}

fn main() -> Result<(), String> {
    let listener = TcpListener::bind(HOST).map_err(|e| format!("Cannot start Studio: {e}"))?;
    println!("TextilLab Studio: http://{HOST}");
    println!("This is a local prototype. Press Ctrl+C to stop.");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => serve(stream),
            Err(error) => eprintln!("Connection error: {error}"),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_encoding_and_form_validation() {
        assert_eq!(percent_decode("v+0+0+0%0A").unwrap(), "v 0 0 0\n");
        assert!(percent_decode("%ZZ").is_err());
    }

    #[test]
    fn studio_api_unfolds_example() {
        let sample = include_str!("../../examples/bent-panel.obj");
        let body = format!(
            "obj={}&seams=1%3A3&allowance=6&density=210&unit=mm",
            sample.replace('\n', "%0A").replace(' ', "+")
        );
        let svg = generate_from_form(&body).unwrap();
        assert!(svg.contains("data-pieces=\"2\""));
        assert!(svg.contains("stroke-dasharray"));
        assert!(svg.contains("data-mass="));
    }

    #[test]
    fn reject_invalid_inputs() {
        assert!(generate_from_form("obj=invalid").is_err());
        assert!(generate_from_form("obj=v+0+0+0%0Av+1+0+0%0Av+0+1+0%0Af+1+2+3&density=-10").is_err());
    }
}

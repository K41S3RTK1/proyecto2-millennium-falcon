use crate::{
    camera::Camera,
    png,
    render::{self, Settings},
    scene::Scene,
};
use std::{
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    time::Duration,
};
const HTML: &str = include_str!("../web/index.html");
fn response(
    s: &mut TcpStream,
    status: &str,
    kind: &str,
    data: &[u8],
    extra: &str,
) -> io::Result<()> {
    write!(
        s,
        "HTTP/1.1 {status}\r\nContent-Type: {kind}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n{extra}\r\n",
        data.len()
    )?;
    s.write_all(data)
}
fn num(query: &str, key: &str, default: f32, min: f32, max: f32) -> f32 {
    query
        .split('&')
        .filter_map(|s| s.split_once('='))
        .find(|(k, _)| *k == key)
        .and_then(|(_, v)| v.parse::<f32>().ok())
        .filter(|v| v.is_finite())
        .unwrap_or(default)
        .clamp(min, max)
}
fn request(s: &mut TcpStream, scene: &Scene) -> io::Result<()> {
    s.set_read_timeout(Some(Duration::from_secs(20)))?;
    s.set_write_timeout(Some(Duration::from_secs(60)))?;
    let mut bytes = Vec::new();
    let end;
    loop {
        let mut buf = [0u8; 8192];
        let n = s.read(&mut buf)?;
        if n == 0 {
            return Ok(());
        }
        bytes.extend_from_slice(&buf[..n]);
        if let Some(i) = bytes.windows(4).position(|v| v == b"\r\n\r\n") {
            end = i + 4;
            break;
        }
        if bytes.len() > 16384 {
            return response(
                s,
                "431 Request Header Fields Too Large",
                "text/plain",
                b"Header too large",
                "",
            );
        }
    }
    let header = String::from_utf8_lossy(&bytes[..end]).into_owned();
    let mut start = header.lines().next().unwrap_or("").split_whitespace();
    let method = start.next().unwrap_or("");
    let target = start.next().unwrap_or("/");
    let (path, query) = target.split_once('?').unwrap_or((target, ""));
    match (method, path) {
        ("GET", "/") => response(s, "200 OK", "text/html; charset=utf-8", HTML.as_bytes(), ""),
        ("GET", "/health") => response(
            s,
            "200 OK",
            "application/json",
            format!(
                "{{\"blocks\":{},\"materials\":{},\"dependencies\":1}}",
                scene.blocks.len(),
                scene.materials.len()
            )
            .as_bytes(),
            "",
        ),
        ("GET", "/frame") => {
            let mut camera = Camera::default();
            camera.yaw = num(query, "yaw", camera.yaw, -3600., 3600.);
            camera.pitch = num(query, "pitch", camera.pitch, -15., 80.);
            camera.distance = num(query, "distance", camera.distance, 3., 48.);
            camera.target.x = num(query, "tx", camera.target.x, -10., 10.);
            camera.target.y = num(query, "ty", camera.target.y, -1., 7.);
            camera.target.z = num(query, "tz", camera.target.z, -12., 12.);
            let cfg = Settings {
                width: num(query, "width", 800., 160., 1600.) as usize,
                height: num(query, "height", 520., 100., 1100.) as usize,
                quality: num(query, "quality", 1., 0., 2.) as u8,
                reflections: num(query, "reflections", 1., 0., 1.) > 0.,
                refractions: num(query, "refractions", 1., 0., 1.) > 0.,
                space: num(query, "space", 0., 0., 1.) > 0.,
                skybox: num(query, "skybox", 1., 0., 1.) > 0.,
                ..Settings::default()
            };
            let frame = render::render(scene, camera, cfg);
            let data = png::encode(frame.width, frame.height, &frame.pixels);
            response(
                s,
                "200 OK",
                "image/png",
                &data,
                &format!("X-Render-Ms: {}\r\n", frame.elapsed.as_millis()),
            )
        }
        ("POST", "/recording") => {
            let length = header
                .lines()
                .find_map(|l| {
                    l.split_once(':')
                        .filter(|(k, _)| k.eq_ignore_ascii_case("content-length"))
                        .and_then(|(_, v)| v.trim().parse::<usize>().ok())
                })
                .unwrap_or(0);
            if !(4..=100_000_000).contains(&length) {
                return response(
                    s,
                    "413 Content Too Large",
                    "text/plain",
                    b"Invalid video size",
                    "",
                );
            }
            let mut body = bytes[end..].to_vec();
            body.truncate(length);
            while body.len() < length {
                let mut buf = [0u8; 65536];
                let n = s.read(&mut buf[..(length - body.len()).min(65536)])?;
                if n == 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "Video incompleto",
                    ));
                }
                body.extend_from_slice(&buf[..n]);
            }
            if !body.starts_with(&[0x1a, 0x45, 0xdf, 0xa3]) {
                return response(s, "400 Bad Request", "text/plain", b"Expected WebM", "");
            }
            std::fs::create_dir_all("renders")?;
            std::fs::write("renders/diorama.webm", body)?;
            response(s, "200 OK", "text/plain", b"renders/diorama.webm", "")
        }
        ("GET", "/video") => match std::fs::read("renders/diorama.webm") {
            Ok(data) => response(s, "200 OK", "video/webm", &data, ""),
            Err(_) => response(
                s,
                "404 Not Found",
                "text/plain",
                b"Primero graba el recorrido",
                "",
            ),
        },
        _ => response(s, "404 Not Found", "text/plain", b"Not found", ""),
    }
}
pub fn serve(scene: Scene, port: u16) -> io::Result<()> {
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    println!(
        "Visor: http://127.0.0.1:{port}\nArrastra para rotar; rueda para zoom. Ctrl+C termina el servidor."
    );
    for connection in listener.incoming() {
        match connection {
            Ok(mut s) => {
                if let Err(e) = request(&mut s, &scene) {
                    eprintln!("Petición: {e}");
                }
            }
            Err(e) => eprintln!("Conexión: {e}"),
        }
    }
    Ok(())
}

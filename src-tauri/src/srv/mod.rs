use std::{fs, io, net::{Ipv4Addr, SocketAddrV4}, sync::{atomic::AtomicBool, Arc}};

use axum::{body::Body, extract::{Path, State}, http::{header, HeaderValue, StatusCode}, response::{IntoResponse, Response}, routing, Router};
use crate::{id::Id, rt, workspace::WSLock, zipext::ZipExt};

#[derive(Clone)]
pub struct Server {
    pub port: u16,
    running: Arc<AtomicBool>
}
impl Server {
    pub fn new() -> Option<Self> {
        select_port().map(|port| Self { port, running: Arc::new(AtomicBool::new(false)) })
    }
    pub fn run(self, ws: WSLock) {
        std::thread::spawn(move || {
            if self.running.swap(true, std::sync::atomic::Ordering::Relaxed) {
                return;
            }
            if let Err(e) = rt::block_on(run_server(self.port, ws)) {
                eprintln!("Server error: {e}");
            }
            self.running.store(false, std::sync::atomic::Ordering::Relaxed);
        });
    }
}

fn select_port() -> Option<u16> {
    let mut port = 9267;
    while port < 0x8000 {
        let ip = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port);
        if std::net::TcpListener::bind(ip)
            .and_then(|x| x.local_addr())
            .map_or(false, |x| x.port() == port) {
            return Some(port);
        }
        port += 1;
    }
    None
}

async fn get_raw_data(
    Path((id, path)): Path<(Id, String)>,
    State(ws): State<WSLock>,
) -> Response {
    let mut hm = header::HeaderMap::new();
    hm.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"));
    let (status, body) = match get_raw(&ws, id, &path) {
        Some(Ok(data)) => {
            hm.insert(header::CACHE_CONTROL, HeaderValue::from_static("max-age=31536000"));
            (StatusCode::OK, Body::from(data))
        }
        None => (StatusCode::NOT_FOUND, Body::empty()),
        Some(Err(e)) => {
            eprintln!("{e}");
            (StatusCode::INTERNAL_SERVER_ERROR, Body::empty())
        }
    };
    (status, hm, body).into_response()
}

pub async fn run_server(port: u16, ws: WSLock) -> anyhow::Result<()> {
    let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let app = Router::new()
        .route("/raw/:id/*path", routing::get(get_raw_data))
        .with_state(ws);
    Ok(axum::serve(listener, app).await?)
}

fn get_raw(ws: &WSLock, id: Id, path: &str) -> Option<anyhow::Result<Vec<u8>>> {
    let bp = ws.locking(|ws| ws.entry_path(id)).ok()?;
    let file = match fs::File::open(bp) {
        Ok(x) => x,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            return None;
        }
        Err(e) => { return Some(Err(e.into())) }
    };
    let mut zip = match zip::ZipArchive::new(io::BufReader::new(file)) {
        Ok(x) => x,
        Err(e) => { return Some(Err(e.into())) }
    };
    zip.read_mem(path)
}
use std::{fs, io, net::{Ipv4Addr, SocketAddrV4}, ops::RangeInclusive, sync::{atomic::AtomicBool, Arc}};

use axum::{body::Body, extract::{Path, State}, http::{header, HeaderMap, HeaderValue, StatusCode}, response::{IntoResponse, Response}, routing, Router};
use crate::{id::Id, rt, workspace::DirWS};

#[derive(Clone)]
pub struct Server {
    pub port: u16,
    running: Arc<AtomicBool>
}
impl Server {
    pub fn new() -> Option<Self> {
        select_port().map(|port| Self { port, running: Arc::new(AtomicBool::new(false)) })
    }
    pub fn run(self, ws: DirWS) {
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
    headers: HeaderMap,
    State(ws): State<DirWS>,
) -> Response {
    let mut hm = header::HeaderMap::new();
    hm.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"));
    let (status, body) = match get_raw(&ws, id, &path) {
        Some(Ok(data)) => {
            let dlen = data.len();
            hm.insert(header::CACHE_CONTROL, HeaderValue::from_static("max-age=31536000"));
            if let Some(range) = headers.get(header::RANGE).and_then(|hr| {
                parse_range(hr, dlen as u64)
                    .inspect_err(|e| eprintln!("{e}"))
                    .ok()
            }) {
                hm.insert(header::CONTENT_RANGE, HeaderValue::from_str(&format!("bytes {}-{}/{}", range.start(), range.end(), dlen)).unwrap());
                let data = data[range].to_vec();
                (StatusCode::PARTIAL_CONTENT, Body::from(data))
            } else {
                (StatusCode::OK, Body::from(data))
            }
        }
        None => (StatusCode::NOT_FOUND, Body::empty()),
        Some(Err(e)) => {
            eprintln!("{e}");
            (StatusCode::INTERNAL_SERVER_ERROR, Body::empty())
        }
    };
    (status, hm, body).into_response()
}

fn parse_range(hv: &HeaderValue, size: u64) -> anyhow::Result<RangeInclusive<usize>> {
    let range = hv.to_str()?;
    let pr = http_range_header::parse_range_header(range)?;
    let vr = pr.validate(size)?;
    let (start, end) = vr[0].clone().into_inner();
    Ok((start.try_into()?)..=(end.try_into()?))
}

pub async fn run_server(port: u16, ws: DirWS) -> anyhow::Result<()> {
    let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let app = Router::new()
        .route("/raw/:id/*path", routing::get(get_raw_data))
        .with_state(ws);
    Ok(axum::serve(listener, app).await?)
}

fn get_raw(ws: &DirWS, id: Id, path: &str) -> Option<anyhow::Result<Vec<u8>>> {
    let mods = ws.mods_read();
    let (bp, fm) = mods.get(&id).and_then(|fi| {
        fi.filemap().map(|fm| (fi.path.clone(), fm))
    })?;
    drop(mods);
    let file = match fs::File::open(bp) {
        Ok(x) => x,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            return None;
        }
        Err(e) => { return Some(Err(e.into())) }
    };
    let fe = fm.get(path)?;
    Some(fe.vec_from(&mut io::BufReader::new(file)))
}
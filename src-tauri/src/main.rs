// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod rt;
mod extract;
mod workspace;
mod ext;
mod jvm;
mod mc;
mod slice;
mod jclass;
mod imp;
mod id;
mod loader;
mod auth;

use std::{borrow::Cow, collections::HashMap, fs::File, io, sync::Arc, time::Instant};
use id::Id;
use tauri::{command, generate_context, generate_handler, http::Response, Manager, State};
use workspace::{Gatherer, WSLock, WSMode};

use crate::workspace::AllGather;

#[command]
async fn auth(app: tauri::AppHandle, state: State<'_, auth::GithubClient>) -> Result<bool, ()> {
    let r = state.authorize(|c, u| Ok(app.emit("authcode", (c, u))?)).await.map_err(|e| eprintln!("Error in auth: {} / {:?}", e, e));
    if r.is_ok() {
        let ghc = state.inner().clone();
        rt::spawn(async move {
            if let Ok(uinfo) = ghc.user_info().await.map_err(|e| eprintln!("Error in user_info: {} / {:?}", e, e)) {
                if let Err(e) = app.emit("auth", uinfo) {
                    eprintln!("Emit auth error: {e}");
                }
            }
        });
    }
    
    Ok(r.is_ok())
}

#[command]
async fn logout(app: tauri::AppHandle, state: State<'_, auth::GithubClient>) -> Result<(), ()> {
    state.remove_token();
    rt::spawn(async move {
        if let Err(e) = app.emit("auth", None::<(Box<str>, Box<str>, u8)>) {
            eprintln!("Emit auth error: {e}");
        }
    });
    Ok(())
}

#[command]
async fn load(app: tauri::AppHandle, state: State<'_, WSLock>) -> Result<(), ()> {
    state.clone().locking(|ws| {
        if ws.is_empty() {
            if let Err(e) = app.emit("ws-open", true) {
                eprintln!("Opening workspace error: {e}");
            }
        }
        Ok(())
    }).map_err(|e| eprintln!("Error in load: {e}"))
}

#[command]
async fn mod_dirs(app: tauri::AppHandle, state: State<'_, WSLock>, kind: imp::ReqModDirs) -> Result<imp::RespModDirs, ()> {
    match kind {
        imp::ReqModDirs::List => {
            let mut v = imp::all_minecraft_dirs();
            v.retain(|p| imp::get_mods_dir(p).is_some());
            Ok(imp::RespModDirs::Listed(v))
        }
        imp::ReqModDirs::Select(dir) => {
            let astate = Arc::clone(&state.0);
            rt::spawn(async move {
                let Some(mdir) = imp::get_mods_dir(&dir) else { return; };
                if let Err(e) = astate.lock().unwrap().prepare(mdir) {
                    eprintln!("Opening workspace error: {e}");
                }
                if let Err(e) = app.emit("ws-open", true) {
                    eprintln!("Opening workspace error: {e}");
                }
            });
            Ok(imp::RespModDirs::Selected)
        }
    }
}

#[command]
async fn workspace(app: tauri::AppHandle, state: State<'_, WSLock>, open: bool) -> Result<(), ()> {
    let astate = Arc::clone(&state.0);
    rt::spawn(async move {
        if open {
            let Some(dir) = rfd::AsyncFileDialog::new().pick_folder().await else { return };
            if let Err(e) = astate.lock().unwrap().prepare(dir.into()) {
                eprintln!("Opening workspace error: {e}");
            }
            if let Err(e) = app.emit("ws-open", true) {
                eprintln!("Opening workspace error: {e}");
            }
        } else {
            astate.lock().unwrap().reset();
            if let Err(e) = app.emit("ws-open", false) {
                eprintln!("Closing workspace error: {e}");
            }
        }
    });
    Ok(())
}

#[command]
async fn ws_files(state: State<'_, WSLock>) -> Result<Vec<(Id, String, u64)>, ()> {
    state.mods().and_then(|afe| {
        let mut x = afe.read().map_err(|_| anyhow::anyhow!("fe read error"))?.iter()
            .map(|(id, fe)| (*id, fe.name(), fe.size()))
            .collect::<Vec<_>>();
        x.sort_by_cached_key(|x| x.1.to_lowercase());
        Ok(x)
    }).map_err(|e| eprintln!("Error in ws_files: {e}"))
}

#[command]
async fn ws_namespaces(state: State<'_, WSLock>) -> Result<Vec<Box<str>>, ()> {
    state.locking(|ws| Ok(ws.namespace_keys())).map_err(|e| eprintln!("Error in ws_namespaces: {e}"))
}

fn ws_item<T: Send + Sync + 'static>(state: State<'_, WSLock>, id: Id, gfn: Gatherer<T>) -> anyhow::Result<Arc<T>> {
    state.mods().and_then(|afe| afe.gather_by_id(id, gfn))
}

#[command]
async fn ws_show(state: State<'_, WSLock>, id: Id) -> Result<(), ()> {
    state.mods().and_then(|afe| {
        let fe = &*afe.read().map_err(|_| anyhow::anyhow!("fe read error"))?;
        let path = match fe.get(&id) {
            Some(fi) => fi.path.clone(),
            None => anyhow::bail!("file path not found"),
        };
        Ok(opener::reveal(path)?)
    }).map_err(|e| eprintln!("Error in ws_show: {e}"))
}

#[command]
async fn ws_name(state: State<'_, WSLock>, id: Id) -> Result<String, ()> {
    state.mods().and_then(|afe| {
        let fe = &*afe.read().map_err(|_| anyhow::anyhow!("fe read error"))?;
        let Some(fe) = fe.get(&id) else { anyhow::bail!("file not found") };
        Ok(fe.name())
    }).map_err(|e| eprintln!("Error in ws_name: {e}"))
}

#[command]
fn ws_mod_data(state: State<'_, WSLock>, id: Id) -> Option<Arc<loader::ModTypeData>> {
    ws_item(state, id, workspace::gather_mod_data).inspect_err(|e| eprintln!("Error in ws_mod_data: {e}")).ok()
}
#[command]
fn ws_str_index(state: State<'_, WSLock>, id: Id) -> Option<Arc<jvm::StrIndexMapped>> {
    ws_item(state, id, workspace::gather_str_index).inspect_err(|e| eprintln!("Error in ws_str_index: {e}")).ok()
}
#[command]
fn ws_file_type_sizes(state: State<'_, WSLock>, mode: WSMode) -> Option<Arc<extract::ModFileTypeSizes>> {
    state.mods().and_then(|afe| {
        mode.gather_from_entries(&afe, workspace::gather_file_type_sizes)
    }).inspect_err(|e| eprintln!("Error in ws_file_type_sizes: {e}")).ok()
}

#[command]
fn ws_content_sizes(state: State<'_, WSLock>, mode: WSMode) -> Option<Arc<extract::ModContentSizes>> {
    state.mods().and_then(|afe| {
        mode.gather_from_entries(&afe, workspace::gather_content_sizes)
    }).inspect_err(|e| eprintln!("Error in ws_content_sizes: {e}")).ok()
}
#[command]
async fn ws_inheritance(state: State<'_, WSLock>, mode: WSMode) -> Result<Arc<ext::Inheritance>, ()> {
    let now = Instant::now();
    let x = state.mods().and_then(|afe| {
        match mode {
            WSMode::Generic(force) => {
                let fe = &*afe.gather_with(force, workspace::gather_inheritance)?;
                Ok(Arc::new(fe.values().filter_map(workspace::FileInfo::get).collect()))
            }
            WSMode::Specific(id) => afe.gather_by_id(id, workspace::gather_inheritance)
        }
    }).map_err(|e| eprintln!("Error in ws_inheritance: {e}"));
    println!("gather_inheritance took {:?}", now.elapsed());
    x
}
#[command]
async fn ws_complexity(state: State<'_, WSLock>, mode: WSMode) -> Result<Arc<jvm::Complexity>, ()> {
    let now = Instant::now();
    let x = state.mods().and_then(|afe| {
        match mode {
            WSMode::Generic(force) => {
                let fe = &*afe.gather_with(force, workspace::gather_complexity)?;
                Ok(Arc::new(fe.values().filter_map(workspace::FileInfo::get).collect()))
            }
            WSMode::Specific(id) => afe.gather_by_id(id, workspace::gather_complexity)
        }
    }).map_err(|e| eprintln!("Error in ws_complexity: {e}"));
    println!("gather_complexity took {:?}", now.elapsed());
    x
}
#[command]
async fn ws_tags(state: State<'_, WSLock>, mode: WSMode) -> Result<Arc<extract::TagsList>, ()> {
    let now = Instant::now();
    let x = state.mods().and_then(|afe| {
        match mode {
            WSMode::Generic(force) => {
                let fe = &*afe.gather_with(force, workspace::gather_tags)?;
                Ok(Arc::new(fe.values().filter_map(workspace::FileInfo::get).collect()))
            }
            WSMode::Specific(id) => afe.gather_by_id(id, workspace::gather_tags)
        }
    }).map_err(|e| eprintln!("Error in ws_tags: {e}"));
    println!("gather_tags took {:?}", now.elapsed());
    x
}
#[command]
async fn ws_recipes(state: State<'_, WSLock>, mode: WSMode) -> Result<Arc<extract::RecipeTypeMap>, ()> {
    let now = Instant::now();
    let x = state.mods().and_then(|afe| {
        match mode {
            WSMode::Generic(force) => {
                let fe = &*afe.gather_with(force, workspace::gather_recipes)?;
                Ok(Arc::new(fe.values().filter_map(workspace::FileInfo::get).collect()))
            }
            WSMode::Specific(id) => afe.gather_by_id(id, workspace::gather_recipes)
        }
    }).map_err(|e| eprintln!("Error in ws_recipes: {e}"));
    println!("gather_recipes took {:?}", now.elapsed());
    x
}

#[command]
async fn ws_mod_entries(state: State<'_, WSLock>, id: Id) -> Result<Arc<jvm::ModEntries>, ()> {
    state.mods()
        .and_then(|afe| afe.gather_by_id(id, workspace::gather_mod_entries))
        .map_err(|e| eprintln!("Error in ws_mod_entry: {e}"))
}

#[command]
fn ws_mod_playable(state: State<'_, WSLock>, id: Id) -> Result<Arc<extract::PlayableFiles>, ()> {
    state.mods()
        .and_then(|afe| afe.gather_by_id(id, workspace::gather_playable))
        .map_err(|e| eprintln!("Error in ws_mod_playable: {e}"))
}

#[command]
fn dbg_parse_times() -> HashMap<Box<str>, f64> {
    let m = jvm::PARSE_TIMES.lock().unwrap();
    m.iter().map(|(k, v)| (k.clone(), v.as_secs_f64())).collect()
}

fn main() {
    tauri::Builder::default()
        .manage(workspace::WSLock::new())
        .manage(auth::GithubClient::setup().expect("Failed to setup github client"))
        .invoke_handler(generate_handler![
            auth, logout, load, mod_dirs, workspace, ws_files, ws_namespaces, ws_show, ws_name, ws_mod_data, ws_str_index, ws_file_type_sizes, ws_content_sizes, ws_inheritance, ws_complexity, ws_tags, ws_mod_entries, ws_recipes, ws_mod_playable, dbg_parse_times
        ])
        .register_asynchronous_uri_scheme_protocol("raw", |app, req, resp| {
            let now = std::time::Instant::now();
            let ws = app.state::<WSLock>().inner().clone();
            rt::spawn(async move {
                let rb = Response::builder();
                resp.respond(match get_raw(&ws, req.uri().path()) {
                    Some(Ok(data)) => rb.header("Content-Length", data.len()).body(Cow::Owned(data)),
                    None => rb.status(404).body(Cow::Borrowed(&[][..])),
                    Some(Err(e)) => {
                        eprintln!("{e}");
                        rb.status(500).body(Cow::Borrowed(&[][..]))
                    }
                }.unwrap());
                println!("Fetched in {:?} -> {}", now.elapsed(), req.uri().path());
            });
        })
        .run(generate_context!())
        .expect("error while running tauri application");
}

#[inline]
fn get_raw(ws: &WSLock, uri_path: &str) -> Option<anyhow::Result<Vec<u8>>> {
    let (s_id, path) = uri_path[1..].split_once('/')?;
    let id = s_id.parse().ok()?;
    let bp = ws.locking(|ws| ws.entry_path(id)).ok()?;
    let file = match File::open(bp) {
        Ok(x) => x,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return None;
        }
        Err(e) => { return Some(Err(e.into())) }
    };
    let mut zip = match zip::ZipArchive::new(io::BufReader::new(file)) {
        Ok(x) => x,
        Err(e) => { return Some(Err(e.into())) }
    };
    extract::get_raw_data(&mut zip, path).map(Ok)
}
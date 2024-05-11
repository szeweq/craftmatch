// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod manifest;
mod extract;
mod workspace;
mod ext;
mod jvm;
mod mc;
mod slice;
mod jclass;
mod imp;

use std::{borrow::Cow, collections::HashMap, sync::Arc, time::Instant};
use tauri::{async_runtime, command, generate_context, generate_handler, http::Response, Manager, State};
use uuid::Uuid;
use workspace::{WSLock, WSMode};
use slice::BinSearchExt;

use crate::workspace::AllGather;

#[command]
async fn load(app: tauri::AppHandle, state: State<'_, WSLock>) -> Result<(), ()> {
    state.clone().locking(|ws| {
        if &*ws.dir_path != std::path::Path::new("") {
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
            let astate = state.0.clone();
            async_runtime::spawn(async move {
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
async fn open_workspace(app: tauri::AppHandle, state: State<'_, WSLock>) -> Result<(), ()> {
    let astate = state.0.clone();
    async_runtime::spawn(async move {
        let Some(dir) = rfd::AsyncFileDialog::new().pick_folder().await else { return };
        if let Err(e) = astate.lock().unwrap().prepare(dir.into()) {
            eprintln!("Opening workspace error: {e}");
        }
        if let Err(e) = app.emit("ws-open", true) {
            eprintln!("Opening workspace error: {e}");
        }
    });
    Ok(())
}

#[command]
async fn close_workspace(app: tauri::AppHandle, state: State<'_, WSLock>) -> Result<(), ()> {
    let astate = state.0.clone();
    async_runtime::spawn(async move {
        astate.lock().unwrap().reset();
        if let Err(e) = app.emit("ws-open", false) {
            eprintln!("Closing workspace error: {e}");
        }
    });
    Ok(())
}

#[command]
async fn ws_files(state: State<'_, WSLock>) -> Result<Vec<(uuid::Uuid, String)>, ()> {
    let x = state.file_entries().and_then(|afe| {
        let mut x = afe.read().map_err(|_| anyhow::anyhow!("fe read error"))?.iter()
            .map(|fe| (fe.id, fe.name()))
            .collect::<Vec<_>>();
        x.sort_by_cached_key(|x| x.1.to_lowercase());
        Ok(x)
    });
    x.map_err(|e| eprintln!("Error in ws_files: {e}"))
}

#[command]
async fn ws_name(state: State<'_, WSLock>, id: uuid::Uuid) -> Result<String, ()> {
    state.file_entries().and_then(|afe| {
        let fe = &*afe.read().map_err(|_| anyhow::anyhow!("fe read error"))?;
        fe.binsearch_key_map(&id, |fe| fe.id, |fe| Ok(fe.name()))
    }).map_err(|e| eprintln!("Error in ws_name: {e}"))
}

#[command]
fn ws_mod_data(state: State<'_, WSLock>, id: uuid::Uuid) -> Option<Arc<manifest::ModTypeData>> {
    state.file_entries().and_then(|afe| {
        afe.gather_by_id(id, workspace::gather_mod_data)
    }).inspect_err(|e| eprintln!("Error in ws_mod_data: {e}")).ok()
}
#[command]
fn ws_str_index(state: State<'_, WSLock>, id: uuid::Uuid) -> Option<Arc<jvm::StrIndexMapped>> {
    state.file_entries().and_then(|afe| {
        afe.gather_by_id(id, workspace::gather_str_index)
    }).inspect_err(|e| eprintln!("Error in ws_str_index: {e}")).ok()
}

#[command]
fn ws_content_sizes(state: State<'_, WSLock>, mode: WSMode) -> Option<Arc<extract::ModContentSizes>> {
    state.file_entries().and_then(|afe| {
        match mode {
            WSMode::Generic(force) => {
                afe.gather_with(force, workspace::gather_content_sizes)?;
                let fe = &*afe.read().map_err(|_| anyhow::anyhow!("fe read error"))?;
                Ok(Arc::new(fe.iter().filter_map(workspace::FileInfo::get).collect()))
            }
            WSMode::Specific(id) => afe.gather_by_id(id, workspace::gather_content_sizes)
        }
    }).inspect_err(|e| eprintln!("Error in ws_content_sizes: {e}")).ok()
}
#[command]
async fn ws_inheritance(state: State<'_, WSLock>, mode: WSMode) -> Result<Arc<ext::Inheritance>, ()> {
    let now = Instant::now();
    let x = state.file_entries().and_then(|afe| {
        match mode {
            WSMode::Generic(force) => {
                afe.gather_with(force, workspace::gather_inheritance)?;
                let fe = &*afe.read().map_err(|_| anyhow::anyhow!("fe read error"))?;
                Ok(Arc::new(fe.iter().filter_map(workspace::FileInfo::get).collect()))
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
    let x = state.file_entries().and_then(|afe| {
        match mode {
            WSMode::Generic(force) => {
                afe.gather_with(force, workspace::gather_complexity)?;
                let fe = &*afe.read().map_err(|_| anyhow::anyhow!("fe read error"))?;
                Ok(Arc::new(fe.iter().filter_map(workspace::FileInfo::get).collect()))
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
    let x = state.file_entries().and_then(|afe| {
        match mode {
            WSMode::Generic(force) => {
                afe.gather_with(force, workspace::gather_tags)?;
                let fe = &*afe.read().map_err(|_| anyhow::anyhow!("fe read error"))?;
                Ok(Arc::new(fe.iter().filter_map(workspace::FileInfo::get).collect()))
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
    let x = state.file_entries().and_then(|afe| {
        match mode {
            WSMode::Generic(force) => {
                afe.gather_with(force, workspace::gather_recipes)?;
                let fe = &*afe.read().map_err(|_| anyhow::anyhow!("fe read error"))?;
                Ok(Arc::new(fe.iter().filter_map(workspace::FileInfo::get).collect()))
            }
            WSMode::Specific(id) => afe.gather_by_id(id, workspace::gather_recipes)
        }
    }).map_err(|e| eprintln!("Error in ws_recipes: {e}"));
    println!("gather_recipes took {:?}", now.elapsed());
    x
}

#[command]
async fn ws_mod_entries(state: State<'_, WSLock>, id: Uuid) -> Result<Arc<jvm::ModEntries>, ()> {
    state.file_entries()
        .and_then(|afe| afe.gather_by_id(id, workspace::gather_mod_entries))
        .map_err(|e| eprintln!("Error in ws_mod_entry: {e}"))
}

#[command]
fn dbg_parse_times() -> HashMap<Box<str>, f64> {
    let m = jvm::PARSE_TIMES.lock().unwrap();
    m.iter().map(|(k, v)| (k.clone(), v.as_secs_f64())).collect()
}

fn main() {
    tauri::Builder::default()
        .manage(workspace::WSLock::new())
        .invoke_handler(generate_handler![
            load, mod_dirs, open_workspace, close_workspace, ws_files, ws_name, ws_mod_data, ws_str_index, ws_content_sizes, ws_inheritance, ws_complexity, ws_tags, ws_mod_entries, ws_recipes, dbg_parse_times
        ])
        .register_asynchronous_uri_scheme_protocol("raw", |app, req, resp| {
            let now = std::time::Instant::now();
            let ws = app.state::<WSLock>().inner().clone();
            async_runtime::spawn(async move {
                let rb = Response::builder();
                resp.respond(match get_img(&ws, req.uri().path()) {
                    Some(Ok(data)) => rb.header("Content-Length", data.len()).body(Cow::Owned(data)),
                    None => rb.status(404).body(Cow::Borrowed(&[][..])),
                    Some(Err(e)) => {
                        eprintln!("{e}");
                        rb.status(500).body(Cow::Borrowed(&[][..]))
                    }
                }.unwrap());
                println!("Fetching image took {:?} -> {}", now.elapsed(), req.uri().path());
            });
        })
        .run(generate_context!())
        .expect("error while running tauri application");
}

#[inline]
fn get_img(ws: &WSLock, uri_path: &str) -> Option<anyhow::Result<Vec<u8>>> {
    let (s_id, path) = uri_path[1..].split_once('/')?;
    let id = Uuid::try_parse(s_id).ok()?;
    let f = ws.locking(|ws| ws.entry_path(id)).ok()?;
    let mut zip = match ext::zip_open(f) {
        Ok(x) => x,
        Err(e) => { return Some(Err(e)) }
    };
    extract::get_img_data(&mut zip, path).map(Ok)
}
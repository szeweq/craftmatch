// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod rt;
mod extract;
mod workspace;
mod ext;
mod jvm;
mod mc;
mod slice;
mod imp;
mod id;
mod loader;
mod srv;
mod zipext;

use std::{collections::HashMap, sync::Arc, time::Instant};
use id::Id;
use tauri::{command, generate_context, generate_handler, Emitter, Listener, Manager, State};
use workspace::{Gatherer, WSLock, WSMode};

use crate::workspace::AllGather;

#[command]
async fn auth(app: tauri::AppHandle, state: State<'_, cm_auth::GithubClient>) -> Result<bool, ()> {
    let r = state.authorize(|c, u| Ok(app.emit("authcode", (c, u))?)).await.map_err(|e| eprintln!("Error in auth: {} / {:?}", e, e));
    if r.is_ok() {
        let ghc = state.inner().clone();
        rt::spawn(async move {
            if let Ok(uinfo) = ghc.user_info().await.map_err(|e| eprintln!("Error in user_info: {} / {:?}", e, e)) {
                emit_auth(app, Some(uinfo));
            }
        });
    }
    
    Ok(r.is_ok())
}

#[command]
async fn logout(app: tauri::AppHandle, state: State<'_, cm_auth::GithubClient>) -> Result<(), ()> {
    state.remove_token();
    rt::spawn(async move { emit_auth(app, None); });
    Ok(())
}

fn emit_auth(app: tauri::AppHandle, info: Option<(Box<str>, Box<str>, u8)>) {
    if let Err(e) = app.emit("auth", info) {
        eprintln!("Emit auth error: {e}");
    }
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
                let r = astate.lock().unwrap().prepare(mdir);
                if let Err(e) = r {
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
            let r = astate.lock().unwrap().prepare(dir.into());
            if let Err(e) = r {
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
    state.locking(|ws| ws.entry_path(id))
        .and_then(|path| Ok(opener::reveal(path)?))
        .map_err(|e| eprintln!("Error in ws_show: {e}"))
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
fn ws_mod_errors(state: State<'_, WSLock>, id: Id) -> Result<Vec<workspace::FileError>, ()> {
    state.mods().and_then(|afe| {
        let fe = &*afe.read().map_err(|_| anyhow::anyhow!("fe read error"))?;
        let Some(fe) = fe.get(&id) else { anyhow::bail!("file not found") };
        Ok(fe.errors.clone())
    }).map_err(|e| eprintln!("Error in ws_name: {e}"))
}

#[command]
fn ws_file_type_sizes(state: State<'_, WSLock>, mode: WSMode) -> Option<Arc<extract::ModFileTypeSizes>> {
    state.mods().and_then(|afe| {
        mode.gather_from_entries(&afe, workspace::gather_file_type_sizes)
    }).inspect_err(|e| eprintln!("Error in ws_file_type_sizes: {e}")).ok()
}

#[command]
fn ws_dep_map(state: State<'_, WSLock>, mode: WSMode) -> Option<Arc<loader::DepMapIndexed>> {
    state.mods().and_then(|afe| {
        mode.gather_from_entries(&afe, workspace::gather_dep_map)
    }).map(|x| Arc::new(x.as_ref().into())).inspect_err(|e| eprintln!("Error in ws_dep_map: {e}")).ok()
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

#[command]
fn srv_port(state: State<'_, srv::Server>) -> u16 {
    state.inner().port
}

fn main() {
    tauri::Builder::default()
        .manage(workspace::WSLock::new())
        .manage(srv::Server::new().expect("Failed to setup server"))
        .manage(cm_auth::GithubClient::setup().expect("Failed to setup github client"))
        .setup(|app| {
            let wapp = app.handle().clone();
            let wss = app.state::<WSLock>().inner().clone();
            app.listen("load", move |_| {
                let ws = wapp.state::<WSLock>().inner().clone();
                if let Err(e) = ws.locking(|ws| {
                    if ws.is_empty() {
                        if let Err(e) = wapp.emit("ws-open", true) {
                            eprintln!("Opening workspace error: {e}");
                        }
                    }
                    Ok(())
                }) { eprintln!("Error in load: {e}"); }
            });
            let server = app.state::<srv::Server>().inner().clone();
            server.run(wss);
            Ok(())
        })
        .invoke_handler(generate_handler![
            auth, logout, mod_dirs, workspace,
            ws_files, ws_namespaces, ws_show, ws_name, ws_mod_data, ws_dep_map, ws_str_index, ws_mod_errors, ws_file_type_sizes, ws_content_sizes, ws_inheritance, ws_complexity, ws_tags, ws_mod_entries, ws_recipes, ws_mod_playable,
            dbg_parse_times, srv_port
        ])
        .run(generate_context!())
        .expect("error while running tauri application");
}

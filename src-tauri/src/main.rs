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
mod err;

use core::str;
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use err::SafeResult;
use id::Id;
use tauri::{command, generate_context, generate_handler, Emitter, Listener, Manager, State};
use workspace::{AllGather, DirWS, Gatherer, WSMode};

#[command]
async fn auth(app: tauri::AppHandle, state: State<'_, cm_auth::GithubClient>) -> SafeResult<bool> {
    let r = state.authorize(|c, u| Ok(app.emit("authcode", (c, u))?)).await.map_err(|e| eprintln!("Error in auth: {e} / {e:?}"));
    if r.is_ok() {
        let ghc = state.inner().clone();
        rt::spawn(async move {
            if let Ok(uinfo) = ghc.user_info().await.map_err(|e| eprintln!("Error in user_info: {e} / {e:?}")) {
                emit_auth(app, Some(uinfo));
            }
        });
    }
    
    Ok(r.is_ok())
}

#[command]
async fn logout(app: tauri::AppHandle, state: State<'_, cm_auth::GithubClient>) -> SafeResult<()> {
    state.remove_token().await;
    emit_auth(app, None);
    Ok(())
}

fn emit_auth(app: tauri::AppHandle, info: Option<(Box<str>, Box<str>, u8)>) {
    if let Err(e) = app.emit("auth", info) {
        eprintln!("Emit auth error: {e}");
    }
}

fn emit_ws_open(app: tauri::AppHandle, dws: &DirWS, dir_path: PathBuf) {
    if let Err(e) = dws.prepare(dir_path) {
        eprintln!("Opening workspace error: {e}");
    }
    if let Err(e) = app.emit("ws-open", true) {
        eprintln!("Opening workspace error: {e}");
    }
}

#[command]
fn dirs(app: tauri::AppHandle, state: State<'_, DirWS>, kind: imp::ReqModDirs) -> imp::RespModDirs {
    match kind {
        imp::ReqModDirs::List => {
            let mut v = imp::all_minecraft_dirs();
            v.retain(|p| imp::get_mods_dir(p).is_some());
            imp::RespModDirs::Listed(v)
        }
        imp::ReqModDirs::Select(dir) => {
            let dws = state.inner().clone();
            rt::spawn(async move {
                let Some(mdir) = imp::get_mods_dir(&dir) else { return; };
                emit_ws_open(app, &dws, mdir);
            });
            imp::RespModDirs::Selected
        }
    }
}

#[command]
fn workspace(app: tauri::AppHandle, state: State<'_, DirWS>, open: bool) {
    let dws = state.inner().clone();
    rt::spawn(async move {
        if open {
            let Some(dir) = rfd::AsyncFileDialog::new().pick_folder().await else { return };
            emit_ws_open(app, &dws, dir.into());
        } else {
            dws.reset();
            if let Err(e) = app.emit("ws-open", false) {
                eprintln!("Closing workspace error: {e}");
            }
        }
    });
}

#[command]
async fn ws_files(state: State<'_, DirWS>) -> Result<Vec<(Id, String, u64)>, ()> {
    let mods = state.mods_read();
    let v = mods.iter()
        .map(|(id, fe)| (*id, fe.name(), fe.size()))
        .collect::<Vec<_>>();
    drop(mods);
    Ok(v)
}

#[command]
fn ws_namespaces(state: State<'_, DirWS>) -> Vec<Box<str>> {
    state.namespace_keys()
}

fn ws_item<T: Send + Sync + 'static>(state: State<'_, DirWS>, id: Id, gfn: Gatherer<T>) -> anyhow::Result<Arc<T>> {
    state.mods().gather_by_id(id, gfn)
}

#[command]
fn ws_show(state: State<'_, DirWS>, id: Id) -> bool {
    state.entry_path(id)
        .and_then(|path| Ok(opener::reveal(path)?))
        .map_err(|e| eprintln!("Error in ws_show: {e}"))
        .is_ok()
}

#[command]
async fn ws_name(state: State<'_, DirWS>, id: Id) -> Result<String, ()> {
    let mods = state.mods_read();
    Ok(mods.get(&id).map_or_else(|| {
        eprintln!("Error in ws_name: file not found");
        String::new()
    }, |fe| fe.name()))
}

#[command]
fn ws_mod_data(state: State<'_, DirWS>, id: Id) -> Option<Arc<loader::ModTypeData>> {
    ws_item(state, id, workspace::gather_mod_data).inspect_err(|e| eprintln!("Error in ws_mod_data: {e}")).ok()
}
#[command]
fn ws_str_index(state: State<'_, DirWS>, id: Id) -> Option<Arc<jvm::StrIndexMapped>> {
    ws_item(state, id, workspace::gather_str_index).inspect_err(|e| eprintln!("Error in ws_str_index: {e}")).ok()
}
#[command]
fn ws_mod_errors(state: State<'_, DirWS>, id: Id) -> Vec<workspace::FileError> {
    state.mods_read().get(&id).map_or_else(|| {
        eprintln!("Error in ws_mod_errors: file not found");
        vec![]
    }, |fe| fe.errors.clone())
}

#[command]
fn ws_file_type_sizes(state: State<'_, DirWS>, mode: WSMode) -> Option<Arc<extract::ModFileTypeSizes>> {
    let mods = state.mods();
    mode.gather_from_entries(mods, workspace::gather_file_type_sizes)
        .inspect_err(|e| eprintln!("Error in ws_file_type_sizes: {e}")).ok()
}

#[command]
fn ws_dep_map(state: State<'_, DirWS>, mode: WSMode) -> Option<Arc<loader::DepMapIndexed>> {
    let mods = state.mods();
    mode.gather_from_entries(mods, workspace::gather_dep_map)
        .map(|x| Arc::new(x.as_ref().into()))
        .inspect_err(|e| eprintln!("Error in ws_dep_map: {e}")).ok()
}
#[command]
fn ws_content_sizes(state: State<'_, DirWS>, mode: WSMode) -> Option<Arc<extract::ModContentSizes>> {
    mode.gather_from_entries(state.mods(), workspace::gather_content_sizes)
        .inspect_err(|e| eprintln!("Error in ws_content_sizes: {e}")).ok()
}
#[command]
async fn ws_inheritance(state: State<'_, DirWS>, mode: WSMode) -> Result<Arc<ext::Inheritance>, ()> {
    mode.gather_from_entries(state.mods(), workspace::gather_inheritance)
        .map_err(|e| eprintln!("Error in ws_inheritance: {e}"))
}
#[command]
async fn ws_complexity(state: State<'_, DirWS>, mode: WSMode) -> Result<Arc<jvm::Complexity>, ()> {
    mode.gather_from_entries(state.mods(), workspace::gather_complexity)
        .map_err(|e| eprintln!("Error in ws_complexity: {e}"))
}
#[command]
async fn ws_tags(state: State<'_, DirWS>, mode: WSMode) -> Result<Arc<extract::TagsList>, ()> {
    mode.gather_from_entries(state.mods(), workspace::gather_tags)
        .map_err(|e| eprintln!("Error in ws_tags: {e}"))
}
#[command]
async fn ws_recipes(state: State<'_, DirWS>, mode: WSMode) -> Result<Arc<extract::RecipeTypeMap>, ()> {
    mode.gather_from_entries(state.mods(), workspace::gather_recipes)
        .map_err(|e| eprintln!("Error in ws_recipes: {e}"))
}

#[command]
async fn ws_mod_entries(state: State<'_, DirWS>, id: Id) -> Result<Arc<jvm::ModEntries>, ()> {
    state.mods().gather_by_id(id, workspace::gather_mod_entries)
        .map_err(|e| eprintln!("Error in ws_mod_entry: {e}"))
}

#[command]
fn ws_mod_playable(state: State<'_, DirWS>, id: Id) -> Result<Arc<extract::PlayableFiles>, ()> {
    state.mods().gather_by_id(id, workspace::gather_playable)
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
        .manage(workspace::DirWS::new())
        .manage(srv::Server::new().expect("Failed to setup server"))
        .manage(cm_auth::GithubClient::setup().expect("Failed to setup github client"))
        .setup(|app| {
            let wapp = app.handle().clone();
            let wss = app.state::<DirWS>().inner().clone();
            app.listen("load", move |_| {
                let ws = wapp.state::<DirWS>().inner().clone();
                if ws.is_empty() {
                    if let Err(e) = wapp.emit("ws-open", true) {
                        eprintln!("Opening workspace error: {e}");
                    }
                }
            });
            let server = app.state::<srv::Server>().inner().clone();
            server.run(wss);
            Ok(())
        })
        .invoke_handler(generate_handler![
            auth, logout, dirs, workspace,
            ws_files, ws_namespaces, ws_show, ws_name, ws_mod_data, ws_dep_map, ws_str_index, ws_mod_errors, ws_file_type_sizes, ws_content_sizes, ws_inheritance, ws_complexity, ws_tags, ws_mod_entries, ws_recipes, ws_mod_playable,
            dbg_parse_times, srv_port
        ])
        .run(generate_context!())
        .expect("error while running tauri application");
}

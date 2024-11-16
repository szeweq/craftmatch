use std::{fs, path::{Path, PathBuf}, sync::Arc};

use indexmap::IndexMap;
use parking_lot::{RwLock, RwLockReadGuard};
use rayon::prelude::*;

use crate::{ext, id::Id, loader::ModTypeData};

use super::{gather_mod_data, FileInfo, LockMap, Namespaces};

fn id_from_time(path: &Path) -> anyhow::Result<Id> {
    let time = fs::metadata(path)?.modified()?;
    let d = time.duration_since(std::time::UNIX_EPOCH)?;
    Ok(Id::new(d))
}

#[derive(Clone)]
pub struct DirWS {
    dir_path: Arc<RwLock<Box<Path>>>,
    mod_entries: LockMap<FileInfo>,
    filemaps: LockMap<Arc<cm_zipext::FileMap>>,
    namespaces: Namespaces
}
impl DirWS {
    pub fn new() -> Self {
        Self {
            dir_path: Arc::new(RwLock::new(Box::from(Path::new("")))),
            mod_entries: Arc::new(RwLock::new(IndexMap::new())),
            filemaps: Arc::new(RwLock::new(IndexMap::new())),
            namespaces: Arc::new(RwLock::new(IndexMap::new())),
        }
    }
    pub const fn mods(&self) -> &LockMap<FileInfo> {
        &self.mod_entries
    }
    pub fn mods_read(&self) -> RwLockReadGuard<IndexMap<Id, FileInfo>> {
        self.mod_entries.read()
    }
    pub fn reset(&self) {
        *self.dir_path.write() = Box::from(Path::new(""));
        *self.mod_entries.write() = IndexMap::new();
        *self.filemaps.write() = IndexMap::new();
        *self.namespaces.write() = IndexMap::new();
    }
    pub fn is_empty(&self) -> bool {
        &**self.dir_path.read() != Path::new("")
    }
    pub fn prepare(&self, dir_path: PathBuf) -> anyhow::Result<()> {
        *self.dir_path.write() = dir_path.into_boxed_path();
        let rdir = fs::read_dir(&*self.dir_path.read())?;
        let mut jars = rdir.filter_map(|entry| {
            let entry = entry.ok()?;
            if entry.path().is_dir() { return None; }
            if ext::Extension::Jar.matches(&entry.file_name()) {
                let id = id_from_time(&entry.path()).map_err(|e| {
                    eprintln!("{}: {}", entry.path().display(), e);
                }).ok()?;
                let fi = FileInfo::new(entry.path());
                Some((id, fi))
            } else {
                None
            }
        }).collect::<IndexMap<_, _>>();
        jars.sort_unstable_keys();

        let fmaps = jars.par_iter_mut().filter_map(|(id, fi)| {
            let fm = cm_zipext::FileMap::from_zip_read_seek(fi.file_mem().ok()?).ok()?;
            let afm = Arc::new(fm);
            fi.filemap = Arc::downgrade(&afm);
            Some((*id, afm))
        }).collect::<IndexMap<_, _>>();

        let ns = jars.par_iter_mut()
            .filter_map(|(id, fi)| Some((*id, fi.get_or_gather(gather_mod_data).ok()?)))
            .flat_map(|(id, md)| (match &*md {
                ModTypeData::Fabric(d) => d.par_iter(),
                ModTypeData::Forge(d) | ModTypeData::Neoforge(d) => d.par_iter(),
            }).map(|d| (Box::from(d.slug()), id)).collect::<Vec<_>>())
            .collect::<IndexMap<_, _>>();

        *self.mod_entries.write() = jars;
        *self.filemaps.write() = fmaps;
        *self.namespaces.write() = ns;
        Ok(())
    }
    pub fn entry_path(&self, id: Id) -> anyhow::Result<Box<Path>> {
        let fe = self.mod_entries.read();
        let Some(fi) = fe.get(&id) else { anyhow::bail!("file not found") };
        let p = fi.path.clone();
        drop(fe);
        Ok(p)
    }
    pub fn namespace_keys(&self) -> Vec<Box<str>> {
        self.namespaces.read().keys().cloned().collect()
    }
}
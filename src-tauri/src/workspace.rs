use std::{fs::File, io::{self, BufReader}, path::{Path, PathBuf}, sync::{Arc, Mutex, RwLock}};

use indexmap::IndexMap;
use rayon::iter::ParallelIterator;
use serde::Deserialize;
use state::TypeMap;
use uuid::Uuid;

use crate::{ext, extract, jvm, manifest};

#[derive(Clone)]
pub struct WSLock(pub Arc<Mutex<DirWS>>);
impl WSLock {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(DirWS::new())))
    }
    #[inline]
    pub fn locking<T>(&self, f: impl FnOnce(&DirWS) -> anyhow::Result<T>) -> anyhow::Result<T> {
        self.0.lock().map_or_else(|_| Err(anyhow::anyhow!("WSLock poisoned")), |ws| {
            f(&ws)
        })
    }
    pub fn mods(&self) -> anyhow::Result<WSFiles> {
        self.locking(|ws| Ok(ws.mod_entries.clone()))
    }
}

type WSFiles = Arc<RwLock<IndexMap<Uuid, FileInfo>>>;

pub struct DirWS {
    pub dir_path: Box<Path>,
    mod_entries: WSFiles,
}
impl DirWS {
    pub fn new() -> Self {
        Self { dir_path: Box::from(Path::new("")), mod_entries: Arc::new(RwLock::new(IndexMap::new())) }
    }
    pub fn reset(&mut self) {
        *self = Self::new();
    }
    pub fn prepare(&mut self, dir_path: PathBuf) -> anyhow::Result<()> {
        self.dir_path = dir_path.into_boxed_path();
        let rdir = std::fs::read_dir(&self.dir_path)?;
        let mut jars = rdir.filter_map(|entry| {
            let entry = entry.ok()?;
            if entry.path().is_dir() { return None; }
            if ext::Extension::Jar.matches(&entry.file_name()) {
                let id = id_from_time(&entry.path()).inspect_err(|e| {
                    eprintln!("{}: {}", entry.path().display(), e);
                }).ok()?;
                Some((id, FileInfo::new(entry.path())))
            } else {
                None
            }
        }).collect::<IndexMap<_, _>>();
        jars.sort_unstable_keys();
        *self.mod_entries.write().unwrap() = jars;
        Ok(())
    }
    pub fn entry_path(&self, id: Uuid) -> anyhow::Result<Box<Path>> {
        let fe = self.mod_entries.read().map_err(|_| anyhow::anyhow!("fe read error"))?;
        let Some(fi) = fe.get(&id) else { anyhow::bail!("file not found") };
        let p = fi.path.clone();
        drop(fe);
        Ok(p)
    }
}

pub trait AllGather {
    fn gather_with<T: Send + Sync + 'static>(&self, force: bool, gfn: Gatherer<T>) -> anyhow::Result<()>;
    fn gather_by_id<T: Send + Sync + 'static>(&self, id: Uuid, gfn: Gatherer<T>) -> anyhow::Result<Arc<T>>;
}
impl AllGather for WSFiles {
    fn gather_with<T: Send + Sync + 'static>(&self, force: bool, gfn: Gatherer<T>) -> anyhow::Result<()> {
        self.write().map_err(|_| anyhow::anyhow!("Error in gather_with"))?.par_values_mut().for_each(|file_entry| {
            if let Err(e) = file_entry.gather(gfn, force) {
                eprintln!("{}: {}", file_entry.path.display(), e);
            }
        });
        Ok(())
    }
    fn gather_by_id<T: Send + Sync + 'static>(&self, id: Uuid, gfn: Gatherer<T>) -> anyhow::Result<Arc<T>> {
        let fe = &mut *self.write().map_err(|_| anyhow::anyhow!("fe write error"))?;
        let Some(fi) = fe.get_mut(&id) else { anyhow::bail!("file not found") };
        fi.get_or_gather(gfn)
    }
}

fn id_from_time(path: &Path) -> anyhow::Result<Uuid> {
    let time = std::fs::metadata(path)?.modified()?;
    let d = time.duration_since(std::time::UNIX_EPOCH)?;
    let (seconds, nanos) = (d.as_secs(), d.subsec_nanos());
    Ok(Uuid::new_v7(uuid::Timestamp::from_unix(uuid::timestamp::context::NoContext, seconds, nanos)))
}

pub struct FileInfo {
    //pub id: Uuid,
    pub path: Box<Path>,
    datamap: TypeMap![Send + Sync]
}
impl FileInfo {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path: path.into_boxed_path(),
            datamap: <TypeMap![Send + Sync]>::new()
        }
    }
    pub fn name(&self) -> String {
        self.path.file_name().unwrap().to_string_lossy().to_string()
    }
    pub fn size(&self) -> u64 {
        std::fs::metadata(&self.path).map_or(0, |md| md.len())
    }
    fn open<RS: io::Read + io::Seek>(reader: io::Result<RS>) -> anyhow::Result<zip::ZipArchive<RS>> {
        Ok(zip::ZipArchive::new(reader?)?)
    }
    pub fn open_buf(&self) -> anyhow::Result<zip::ZipArchive<BufReader<File>>> {
        Self::open(File::open(&self.path).map(BufReader::new))
    }
    pub fn open_mem(&self) -> anyhow::Result<zip::ZipArchive<io::Cursor<Vec<u8>>>> {
        Self::open(std::fs::read(&self.path).map(io::Cursor::new))
    }
    pub fn get<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        self.datamap.try_get::<Arc<T>>().cloned()
    }
    // pub fn has<T: Send + Sync + 'static>(&self) -> bool {
    //     self.datamap.try_get::<Arc<T>>().is_some()
    // }
    #[inline]
    fn gather<T: Send + Sync + 'static>(&mut self, gatherer: Gatherer<T>, force: bool) -> anyhow::Result<()> {
        if force || self.datamap.try_get::<Arc<T>>().is_none() {
            self.datamap.set(Arc::new(gatherer(self)?));
        }
        Ok(())
    }
    pub fn get_or_gather<T: Send + Sync + 'static>(&mut self, gatherer: Gatherer<T>) -> anyhow::Result<Arc<T>> {
        self.gather(gatherer, false)?;
        self.get().ok_or_else(|| anyhow::anyhow!("No data"))
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum WSMode {
    Generic(bool),
    Specific(Uuid),
}
impl WSMode {
    pub fn gather_from_entries<T: Send + Sync + FromIterator<Arc<T>> + 'static>(self, entries: &WSFiles, gfn: Gatherer<T>) -> anyhow::Result<Arc<T>> {
        match self {
            Self::Generic(force) => {
                entries.gather_with(force, gfn)?;
                let fe = &*entries.read().map_err(|_| anyhow::anyhow!("fe read error"))?;
                Ok(Arc::new(fe.values().filter_map(FileInfo::get::<T>).collect()))
            }
            Self::Specific(id) => entries.gather_by_id(id, gfn)
        }
    }
}

pub type Gatherer<T> = fn(&FileInfo) -> anyhow::Result<T>;

pub fn gather_mod_data(fi: &FileInfo) -> anyhow::Result<manifest::ModTypeData> {
    manifest::extract_mod_info(&mut fi.open_buf()?)
}
pub fn gather_file_type_sizes(fi: &FileInfo) -> anyhow::Result<extract::ModFileTypeSizes> {
    extract::compute_file_type_sizes(&mut fi.open_mem()?)
}
pub fn gather_content_sizes(fi: &FileInfo) -> anyhow::Result<extract::ModContentSizes> {
    extract::compute_mod_content_sizes(&mut fi.open_mem()?)
}
pub fn gather_inheritance(fi: &FileInfo) -> anyhow::Result<ext::Inheritance> {
    jvm::gather_inheritance_v2(fi.open_mem()?)
}
pub fn gather_complexity(fi: &FileInfo) -> anyhow::Result<jvm::Complexity> {
    jvm::gather_complexity(fi.open_mem()?)
}
pub fn gather_tags(fi: &FileInfo) -> anyhow::Result<extract::TagsList> {
    extract::gather_tags(fi.open_mem()?)
}
pub fn gather_str_index(fi: &FileInfo) -> anyhow::Result<jvm::StrIndexMapped> {
    jvm::gather_str_index_v2(fi.open_mem()?)
}
pub fn gather_mod_entries(fi: &FileInfo) -> anyhow::Result<jvm::ModEntries> {
    let Some(moddata) = fi.get::<manifest::ModTypeData>() else { return Err(anyhow::anyhow!("No moddata")) };
    manifest::extract_mod_entries(&mut fi.open_mem()?, moddata.as_ref())
}
pub fn gather_recipes(fi: &FileInfo) -> anyhow::Result<extract::RecipeTypeMap> {
    extract::gather_recipes(&mut fi.open_mem()?)
}
pub fn gather_playable(fi: &FileInfo) -> anyhow::Result<extract::PlayableFiles> {
    extract::gather_playable_files(&fi.open_buf()?)
}
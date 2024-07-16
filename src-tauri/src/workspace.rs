use std::{any::type_name, fs::{self, File}, io::{self, BufReader}, path::{Path, PathBuf}, sync::{Arc, Mutex, RwLock, RwLockReadGuard}, time};

use indexmap::IndexMap;
use rayon::iter::{IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator};
use serde::Deserialize;
use state::TypeMap;

use crate::{ext, extract, id::Id, jvm, loader::{self, ModTypeData}, zipext};

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
        self.locking(|ws| Ok(Arc::clone(&ws.mod_entries)))
    }
}

type WSFiles = Arc<RwLock<IndexMap<Id, FileInfo>>>;
type Namespaces = Arc<RwLock<IndexMap<Box<str>, Id>>>;

pub struct DirWS {
    dir_path: Box<Path>,
    mod_entries: WSFiles,
    namespaces: Namespaces
}
impl DirWS {
    pub fn new() -> Self {
        Self {
            dir_path: Box::from(Path::new("")),
            mod_entries: Arc::new(RwLock::new(IndexMap::new())),
            namespaces: Arc::new(RwLock::new(IndexMap::new())),
        }
    }
    pub fn reset(&mut self) {
        *self = Self::new();
    }
    pub fn is_empty(&self) -> bool {
        &*self.dir_path != Path::new("")
    }
    pub fn prepare(&mut self, dir_path: PathBuf) -> anyhow::Result<()> {
        self.dir_path = dir_path.into_boxed_path();
        let rdir = fs::read_dir(&self.dir_path)?;
        let mut jars = rdir.filter_map(|entry| {
            let entry = entry.ok()?;
            if entry.path().is_dir() { return None; }
            if ext::Extension::Jar.matches(&entry.file_name()) {
                let id = id_from_time(&entry.path()).map_err(|e| {
                    eprintln!("{}: {}", entry.path().display(), e);
                }).ok()?;
                let mut fi = FileInfo::new(entry.path());
                fi.gather(gather_filemap, false).map_err(|e| eprintln!("Invalid filemap: {}", e)).ok()?;
                Some((id, fi))
            } else {
                None
            }
        }).collect::<IndexMap<_, _>>();
        jars.sort_unstable_keys();

        let ns = jars.par_iter_mut()
            .filter_map(|(id, fi)| Some((*id, fi.get_or_gather(gather_mod_data).ok()?)))
            .flat_map(|(id, md)| (match &*md {
                ModTypeData::Fabric(d) => d.par_iter(),
                ModTypeData::Forge(d) => d.par_iter(),
            }).map(|d| (Box::from(d.slug()), id)).collect::<Vec<_>>())
            .collect::<IndexMap<_, _>>();

        *self.mod_entries.write().unwrap() = jars;
        *self.namespaces.write().unwrap() = ns;
        Ok(())
    }
    pub fn entry_path(&self, id: Id) -> anyhow::Result<Box<Path>> {
        let fe = self.mod_entries.read().map_err(|_| anyhow::anyhow!("fe read error"))?;
        let Some(fi) = fe.get(&id) else { anyhow::bail!("file not found") };
        let p = fi.path.clone();
        drop(fe);
        Ok(p)
    }
    pub fn namespace_keys(&self) -> Vec<Box<str>> {
        self.namespaces.read().unwrap().keys().cloned().collect()
    }
}

pub trait AllGather {
    fn gather_with<T: Send + Sync + 'static>(&self, force: bool, gfn: Gatherer<T>) -> anyhow::Result<RwLockReadGuard<'_, IndexMap<Id, FileInfo>>>;
    fn gather_by_id<T: Send + Sync + 'static>(&self, id: Id, gfn: Gatherer<T>) -> anyhow::Result<Arc<T>>;
}
impl AllGather for WSFiles {
    fn gather_with<T: Send + Sync + 'static>(&self, force: bool, gfn: Gatherer<T>) -> anyhow::Result<RwLockReadGuard<'_, IndexMap<Id, FileInfo>>> {
        self.write().map_err(|_| anyhow::anyhow!("Error in gather_with"))?.par_values_mut().for_each(|file_entry| {
            if let Err(e) = file_entry.gather(gfn, force) {
                eprintln!("{}: {}", file_entry.path.display(), e);
            }
        });
        self.read().map_err(|_| anyhow::anyhow!("WSFiles read error"))
    }
    fn gather_by_id<T: Send + Sync + 'static>(&self, id: Id, gfn: Gatherer<T>) -> anyhow::Result<Arc<T>> {
        let fe = &mut *self.write().map_err(|_| anyhow::anyhow!("fe write error"))?;
        let Some(fi) = fe.get_mut(&id) else { anyhow::bail!("file not found") };
        fi.get_or_gather(gfn)
    }
}

fn id_from_time(path: &Path) -> anyhow::Result<Id> {
    let time = fs::metadata(path)?.modified()?;
    let d = time.duration_since(time::UNIX_EPOCH)?;
    Ok(Id::new(d))
}

pub type FileError = (f64, &'static str, Box<str>);

fn now_seconds() -> f64 {
    time::SystemTime::now().duration_since(time::UNIX_EPOCH).map_or(0.0, |d| d.as_secs_f64())
}

pub struct FileInfo {
    //pub id: Uuid,
    pub path: Box<Path>,
    pub errors: Vec<FileError>,
    datamap: TypeMap![Send + Sync]
}
impl FileInfo {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path: path.into_boxed_path(),
            errors: Vec::new(),
            datamap: <TypeMap![Send + Sync]>::new()
        }
    }
    pub fn name(&self) -> String {
        self.path.file_name().map_or(self.path.as_os_str(), |name| name).to_string_lossy().to_string()
    }
    pub fn size(&self) -> u64 {
        fs::metadata(&self.path).map_or(0, |md| md.len())
    }
    pub fn file_buf(&self) -> io::Result<BufReader<File>> {
        File::open(&self.path).map(BufReader::new)
    }
    pub fn file_mem(&self) -> io::Result<io::Cursor<Vec<u8>>> {
        fs::read(&self.path).map(io::Cursor::new)
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
            let item = gatherer(self);
            if let Err(e) = &item {
                self.errors.push((now_seconds(), type_name::<T>(), e.to_string().into_boxed_str()));
            }
            self.datamap.set(Arc::new(item?));
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
    Specific(Id),
}
impl WSMode {
    pub fn gather_from_entries<T: Send + Sync + FromIterator<Arc<T>> + 'static>(self, entries: &WSFiles, gfn: Gatherer<T>) -> anyhow::Result<Arc<T>> {
        match self {
            Self::Generic(force) => {
                let fe = &*entries.gather_with(force, gfn)?;
                Ok(Arc::new(fe.values().filter_map(FileInfo::get::<T>).collect()))
            }
            Self::Specific(id) => entries.gather_by_id(id, gfn)
        }
    }
}

pub type Gatherer<T> = fn(&FileInfo) -> anyhow::Result<T>;

fn get_file_map(fi: &FileInfo) -> anyhow::Result<Arc<zipext::FileMap>> {
    fi.get().ok_or_else(|| anyhow::anyhow!("No file map"))
}

pub fn gather_mod_data(fi: &FileInfo) -> anyhow::Result<loader::ModTypeData> {
    let fm = get_file_map(fi)?;
    loader::extract_mod_info(&fm, &mut fi.file_buf()?)
}
pub fn gather_dep_map(fi: &FileInfo) -> anyhow::Result<loader::DepMap> {
    let fm = get_file_map(fi)?;
    loader::extract_dep_map(&fm, &mut fi.file_buf()?)
}
pub fn gather_file_type_sizes(fi: &FileInfo) -> anyhow::Result<extract::ModFileTypeSizes> {
    let fm = get_file_map(fi)?;
    extract::compute_file_type_sizes(&fm)
}
pub fn gather_content_sizes(fi: &FileInfo) -> anyhow::Result<extract::ModContentSizes> {
    let fm = get_file_map(fi)?;
    extract::compute_mod_content_sizes(&fm)
}
pub fn gather_inheritance(fi: &FileInfo) -> anyhow::Result<ext::Inheritance> {
    let fm = get_file_map(fi)?;
    jvm::gather_inheritance_v2(&fm, &mut fi.file_mem()?)
}
pub fn gather_complexity(fi: &FileInfo) -> anyhow::Result<jvm::Complexity> {
    let fm = get_file_map(fi)?;
    jvm::gather_complexity(&fm, &mut fi.file_mem()?)
}
pub fn gather_tags(fi: &FileInfo) -> anyhow::Result<extract::TagsList> {
    let fm = get_file_map(fi)?;
    extract::gather_tags(&fm, &mut fi.file_mem()?)
}
pub fn gather_str_index(fi: &FileInfo) -> anyhow::Result<jvm::StrIndexMapped> {
    let fm = get_file_map(fi)?;
    jvm::gather_str_index_v2(&fm, &mut fi.file_mem()?)
}
pub fn gather_mod_entries(fi: &FileInfo) -> anyhow::Result<jvm::ModEntries> {
    let fm = get_file_map(fi)?;
    let Some(moddata) = fi.get::<loader::ModTypeData>() else { return Err(anyhow::anyhow!("No moddata")) };
    loader::extract_mod_entries(&fm, moddata.as_ref(), &mut fi.file_mem()?)
}
pub fn gather_recipes(fi: &FileInfo) -> anyhow::Result<extract::RecipeTypeMap> {
    let fm = get_file_map(fi)?;
    extract::gather_recipes(&fm, &mut fi.file_mem()?)
}
pub fn gather_playable(fi: &FileInfo) -> anyhow::Result<extract::PlayableFiles> {
    let fm = get_file_map(fi)?;
    Ok(extract::gather_playable_files(&fm))
}
pub fn gather_filemap(fi: &FileInfo) -> anyhow::Result<zipext::FileMap> {
    use zipext::ZipExt;
    zip::ZipArchive::new(fi.file_mem()?)?.file_map()
}
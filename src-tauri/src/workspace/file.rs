use std::{any::type_name, fs, io::{self, BufReader}, path::{Path, PathBuf}, sync::{Arc, Weak}, time};

use state::TypeMap;

use super::Gatherer;

pub type FileError = (f64, &'static str, Box<str>);

fn now_seconds() -> f64 {
    time::SystemTime::now().duration_since(time::UNIX_EPOCH).map_or(0.0, |d| d.as_secs_f64())
}

pub struct FileInfo {
    pub(super) filemap: Weak<cm_zipext::FileMap>,
    pub path: Box<Path>,
    pub errors: Vec<FileError>,
    datamap: TypeMap![Send + Sync]
}
impl FileInfo {
    pub fn new(path: PathBuf) -> Self {
        Self {
            filemap: Weak::new(),
            path: path.into_boxed_path(),
            errors: Vec::new(),
            datamap: <TypeMap![Send + Sync]>::new()
        }
    }
    pub fn filemap(&self) -> Option<Arc<cm_zipext::FileMap>> {
        self.filemap.upgrade()
    }
    pub fn name(&self) -> String {
        self.path.file_name().map_or(self.path.as_os_str(), |name| name).to_string_lossy().to_string()
    }
    pub fn size(&self) -> u64 {
        fs::metadata(&self.path).map_or(0, |md| md.len())
    }
    pub fn file_buf(&self) -> io::Result<BufReader<fs::File>> {
        fs::File::open(&self.path).map(BufReader::new)
    }
    pub fn file_mem(&self) -> io::Result<io::Cursor<Vec<u8>>> {
        fs::read(&self.path).map(io::Cursor::new)
    }
    pub fn get<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        self.datamap.try_get::<Arc<T>>().cloned()
    }
    #[inline]
    pub(super) fn gather<T: Send + Sync + 'static>(&mut self, gatherer: Gatherer<T>, force: bool) -> anyhow::Result<()> {
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
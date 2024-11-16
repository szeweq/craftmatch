mod dir;
mod file;
mod gather;

use std::sync::Arc;

pub use dir::*;
pub use file::*;
pub use gather::*;

use indexmap::IndexMap;
use parking_lot::RwLock;
use serde::Deserialize;

use crate::id::Id;

pub type LockMap<V> = Arc<RwLock<IndexMap<Id, V>>>;
pub type Namespaces = Arc<RwLock<IndexMap<Box<str>, Id>>>;

#[derive(Deserialize)]
#[serde(untagged)]
pub enum WSMode {
    Generic(bool),
    Specific(Id),
}
impl WSMode {
    pub fn gather_from_entries<T: Send + Sync + FromIterator<Arc<T>> + 'static>(self, entries: &LockMap<FileInfo>, gfn: Gatherer<T>) -> anyhow::Result<Arc<T>> {
        match self {
            Self::Generic(force) => {
                let fe = &*entries.gather_with(force, gfn);
                Ok(Arc::new(fe.values().filter_map(FileInfo::get::<T>).collect()))
            }
            Self::Specific(id) => entries.gather_by_id(id, gfn)
        }
    }
}